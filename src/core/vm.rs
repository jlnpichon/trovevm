use super::{
    bytecode::Opcode,
    value::{Op, Value},
};

#[derive(Debug)]
pub struct VM {
    stack: Vec<Value>,
    ip: usize,
}

pub struct Program {
    bytecode: Vec<Opcode>,
    constants: Vec<Value>,
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
}

impl Program {
    pub fn new(bytecode: &[Opcode], constants: Vec<Value>) -> Self {
        Self {
            bytecode: bytecode.to_vec(),
            constants,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.bytecode {
            _ => todo!(),
        };
        todo!()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            ip: 0,
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

        while self.ip < program.bytecode.len() {
            let opcode = &program.bytecode[self.ip];
            match opcode {
                Opcode::Add => self.apply_binop(Op::Add)?,
                Opcode::Sub => self.apply_binop(Op::Sub)?,
                Opcode::Mul => self.apply_binop(Op::Mul)?,
                Opcode::Div => self.apply_binop(Op::Div)?,
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
                        .constants
                        .get(*index)
                        .ok_or(RuntimeError::InvalidConstantIndex(*index))?;
                    self.push(value.clone());
                }
                Opcode::Jump(_) => todo!(),
            }
            self.ip += 1;
        }
        Ok(())
    }
}

impl Program {
    fn write(&mut self, opcode: Opcode) {
        todo!()
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Opcode;

    type IntoIter = std::slice::Iter<'a, Opcode>;

    fn into_iter(self) -> Self::IntoIter {
        self.bytecode.iter()
    }
}
