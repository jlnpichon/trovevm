pub mod bytecode;
pub mod value;

use std::collections::HashMap;

// Re-exports
pub use bytecode::Opcode;
pub use bytecode::Program;
pub use value::Value;

use value::Op;

#[derive(Debug)]
pub struct VM {
    stack: Vec<Value>,
    ip: usize,
    globals: HashMap<String, Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("invalid opcode")]
    InvalidOpcode,
    #[error("invalid constant index {0}")]
    InvalidConstantIndex(usize),
    #[error("type mismatch")]
    TypeMismatch,
    #[error("division by zero")]
    DivisionByZero,
    #[error("invalid operation '{0:?}' on string")]
    InvalidStringOperation(Op),
    #[error("undefined variable '{0:?}'")]
    UndefinedVariable(String),
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            ip: 0,
            globals: HashMap::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn stack_top(&self) -> Option<&Value> {
        self.stack.last()
    }

    fn apply_binop(&mut self, op: Op) -> Result<(), RuntimeError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let value = lhs.try_apply(op, Some(&rhs))?;
        self.push(value);
        Ok(())
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        self.ip = 0;

        while self.ip < program.len() {
            let opcode = &program[self.ip];
            match opcode {
                Opcode::Add => self.apply_binop(Op::Add)?,
                Opcode::Sub => self.apply_binop(Op::Sub)?,
                Opcode::Mul => self.apply_binop(Op::Mul)?,
                Opcode::Div => self.apply_binop(Op::Div)?,
                Opcode::Mod => self.apply_binop(Op::Mod)?,
                Opcode::Neg => {
                    let lhs = self.pop()?;
                    let value = lhs.try_apply(Op::Neg, None)?;
                    self.push(value);
                }

                Opcode::Lt => self.apply_binop(Op::Lt)?,
                Opcode::Le => self.apply_binop(Op::Le)?,
                Opcode::Gt => self.apply_binop(Op::Gt)?,
                Opcode::Ge => self.apply_binop(Op::Ge)?,
                Opcode::Eq => self.apply_binop(Op::Eq)?,
                Opcode::Neq => self.apply_binop(Op::Neq)?,

                Opcode::And => self.apply_binop(Op::And)?,
                Opcode::Or => self.apply_binop(Op::Or)?,

                Opcode::Pop => {
                    self.pop()?;
                }
                Opcode::Push(index) => {
                    let value = program
                        .constant_get(*index)
                        .ok_or(RuntimeError::InvalidConstantIndex(*index))?;
                    self.push(value.clone());
                }

                Opcode::DefineGlobal(index) => {
                    let name = program
                        .constant_get(*index)
                        .ok_or(RuntimeError::InvalidConstantIndex(*index))?
                        .as_string()
                        .ok_or(RuntimeError::TypeMismatch)?
                        .clone();
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                Opcode::SetGlobal => {
                    let value = self.pop()?;
                    let name = self
                        .pop()?
                        .as_string()
                        .ok_or(RuntimeError::TypeMismatch)?
                        .clone();
                    if let Some(v) = self.globals.get_mut(&name) {
                        *v = value;
                    } else {
                        return Err(RuntimeError::UndefinedVariable(name));
                    }
                }
                Opcode::GetGlobal(index) => {
                    let name = program
                        .constant_get(*index)
                        .ok_or(RuntimeError::InvalidConstantIndex(*index))?
                        .as_string()
                        .ok_or(RuntimeError::TypeMismatch)?
                        .clone();
                    if let Some(value) = self.globals.get(&name) {
                        self.push(value.clone());
                    } else {
                        return Err(RuntimeError::UndefinedVariable(name));
                    }
                }

                Opcode::Jump(_) => todo!(),
                Opcode::Return => todo!(),
            }
            self.ip += 1;
        }
        Ok(())
    }
}
