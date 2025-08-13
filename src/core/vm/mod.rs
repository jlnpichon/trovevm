pub mod bytecode;
mod function;
pub mod value;

use function::CallFrame;
use std::collections::HashMap;
use tracing::trace;

// Re-exports
pub use bytecode::Opcode;
pub use bytecode::Program;
pub use value::Value;

use value::Op;

#[derive(Debug)]
pub struct VM {
    stack: Vec<Value>, // TODO: limit
    ip: usize,
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>, // TODO: limit
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("invalid opcode")]
    InvalidOpcode,
    #[error("invalid constant index {0}")]
    InvalidConstantIndex(usize),
    #[error("stack index {0} is out of bound")]
    StackIndexOutOfBound(usize),
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
            frames: vec![],
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

    pub fn stack_peek(&self, index: usize) -> Result<&Value, RuntimeError> {
        self.stack
            .get(index)
            .ok_or(RuntimeError::StackIndexOutOfBound(index))
    }

    pub fn stack_set(&mut self, index: usize, value: Value) -> Result<(), RuntimeError> {
        if let Some(slot) = self.stack.get_mut(index) {
            *slot = value;
            Ok(())
        } else {
            Err(RuntimeError::StackIndexOutOfBound(index))
        }
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

    fn constant_get(&self, index: usize) -> Result<&Value, RuntimeError> {
        let frame = self.frames.last().ok_or(RuntimeError::StackUnderflow)?;
        frame
            .function
            .program
            .constant_get(index)
            .ok_or(RuntimeError::InvalidConstantIndex(index))
    }

    pub fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        self.ip = 0;

        while self.ip < program.len() {
            let opcode = &program[self.ip];

            trace(self.ip, opcode, &self.stack);

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
                Opcode::SetGlobal(index) => {
                    let value = self
                        .stack_top()
                        .ok_or(RuntimeError::StackUnderflow)?
                        .clone();
                    let name = program
                        .constant_get(*index)
                        .ok_or(RuntimeError::InvalidConstantIndex(*index))?
                        .as_string()
                        .ok_or(RuntimeError::TypeMismatch)?
                        .clone();
                    if let Some(v) = self.globals.get_mut(&name) {
                        *v = value;
                    } else {
                        return Err(RuntimeError::UndefinedVariable(name));
                    }
                }
                Opcode::GetLocal(index) => {
                    let value = self.stack_peek(*index)?;
                    self.push(value.clone());
                }
                Opcode::SetLocal(index) => {
                    let value = self.stack_top().ok_or(RuntimeError::StackUnderflow)?;
                    self.stack_set(*index, value.clone())?;
                }

                Opcode::Jump(offset) => {
                    self.ip += offset;
                    continue;
                }
                Opcode::JumpIfFalse(offset) => {
                    let condition = self.stack_top().ok_or(RuntimeError::StackUnderflow)?;
                    if !condition.is_truthy() {
                        self.ip += offset;
                        continue;
                    }
                }
                Opcode::JumpBack(offset) => {
                    self.ip -= offset;
                    continue;
                }
                Opcode::Return => todo!(),
            }
            self.ip += 1;
        }
        Ok(())
    }
}

fn trace(ip: usize, opcode: &Opcode, stack: &[Value]) {
    use owo_colors::OwoColorize;

    let op_str = format!("{:?}", opcode).yellow().to_string();
    let stack_str = stack
        .iter()
        .map(|v| format!("{:?}", v).red().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let ip_str = format!("IP={:02}", ip).bright_blue().to_string();

    trace!("[{}] {:<25} | Stack: [{}]", ip_str, op_str, stack_str);
}
