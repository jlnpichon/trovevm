pub mod bytecode;
pub mod function;
pub mod native;
pub mod value;

use function::CallFrame;
use function::CompiledFunction;
use function::CompiledFunctionKind;
use native::install_natives;
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
    #[error("type error: {0}")]
    TypeError(String),
    #[error("division by zero")]
    DivisionByZero,
    #[error("invalid operation '{0:?}' on string")]
    InvalidStringOperation(Op),
    #[error("undefined variable '{0:?}'")]
    UndefinedVariable(String),
    #[error("No call frame in VM")]
    NoCallFrame,
    #[error("ip is out of bounds")]
    OutOfBoundsIp,
    #[error("wrong number of arguments: expected {0}, got {1}")]
    WrongArgCount(usize, usize),
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        install_natives(&mut globals);

        Self {
            stack: Vec::with_capacity(1024),
            globals,
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

    pub fn stack_len(&self) -> Result<usize, RuntimeError> {
        let frame = self.frames.last().ok_or(RuntimeError::NoCallFrame)?;
        Ok(self.stack.len() - frame.base)
    }

    pub fn stack_peek_from_top(&self, offset: usize) -> Result<&Value, RuntimeError> {
        let index = self
            .stack
            .len()
            .checked_sub(1 + offset)
            .ok_or(RuntimeError::StackUnderflow)?;
        self.stack
            .get(index)
            .ok_or(RuntimeError::StackIndexOutOfBound(index))
    }

    pub fn stack_peek(&self, index: usize) -> Result<&Value, RuntimeError> {
        let frame = self.frames.last().ok_or(RuntimeError::NoCallFrame)?;
        self.stack
            .get(frame.base + index + 1)
            .ok_or(RuntimeError::StackIndexOutOfBound(index))
    }

    pub fn stack_set(&mut self, index: usize, value: Value) -> Result<(), RuntimeError> {
        let frame = self.frames.last().ok_or(RuntimeError::NoCallFrame)?;

        if let Some(slot) = self.stack.get_mut(frame.base + index + 1) {
            *slot = value;
            Ok(())
        } else {
            Err(RuntimeError::StackIndexOutOfBound(frame.base + index + 1))
        }
    }

    fn apply_binop(&mut self, op: Op) -> Result<(), RuntimeError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let value = lhs.try_apply(op, Some(&rhs))?;
        self.push(value);
        Ok(())
    }

    pub fn ip(&self) -> Result<usize, RuntimeError> {
        Ok(self.frames.last().ok_or(RuntimeError::NoCallFrame)?.ip)
    }

    pub fn ip_mut(&mut self) -> Result<&mut usize, RuntimeError> {
        Ok(&mut self.frames.last_mut().ok_or(RuntimeError::NoCallFrame)?.ip)
    }

    pub fn program(&self) -> Result<&Program, RuntimeError> {
        Ok(&self
            .frames
            .last()
            .ok_or(RuntimeError::NoCallFrame)?
            .function
            .program())
    }

    fn constant_get(&self, index: usize) -> Result<&Value, RuntimeError> {
        let frame = self.frames.last().ok_or(RuntimeError::NoCallFrame)?;
        frame
            .function
            .program()
            .constant_get(index)
            .ok_or(RuntimeError::InvalidConstantIndex(index))
    }

    fn call(&mut self, function: CompiledFunction, args: usize) -> Result<(), RuntimeError> {
        match function.kind {
            CompiledFunctionKind::Bytecode(_) => {
                self.frames.push(CallFrame {
                    function,
                    ip: 0,
                    base: self.stack.len() - args - 1,
                });
            }
            CompiledFunctionKind::Native(native) => {
                let value = native(self, &[])?;
                for _ in 0..args {
                    self.pop()?;
                }
                self.pop()?; // native function object
                self.push(value);
                *self.ip_mut()? += 1;
            }
        };

        Ok(())
    }

    fn read_opcode(&self) -> Result<Opcode, RuntimeError> {
        let ip = self.ip()?;
        let program = self.program()?;
        if ip > program.len() - 1 {
            return Err(RuntimeError::OutOfBoundsIp);
        }

        Ok(program[ip])
    }

    pub fn run(&mut self, function: CompiledFunction) -> Result<Value, RuntimeError> {
        let function_obj = Value::Function(function.clone());
        self.push(function_obj);

        self.call(function, 0);

        self.run_loop()
    }

    pub fn eval(&mut self, function: CompiledFunction) -> Result<Value, RuntimeError> {
        todo!()
    }

    fn run_loop(&mut self) -> Result<Value, RuntimeError> {
        loop {
            let opcode = self.read_opcode()?;

            trace(self.ip()?, &opcode, &self.stack);

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
                    let value = self.constant_get(index)?;
                    self.push(value.clone());
                }

                Opcode::DefineGlobal(index) => {
                    let name = self
                        .constant_get(index)?
                        .as_string()
                        .ok_or(RuntimeError::TypeError(
                            "DefineGlobal: constant must be string".into(),
                        ))?
                        .clone();
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                Opcode::GetGlobal(index) => {
                    let name = self
                        .constant_get(index)?
                        .as_string()
                        .ok_or(RuntimeError::TypeError(
                            "GetGlobal: constant must be a string".into(),
                        ))?
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
                    let name = self
                        .constant_get(index)?
                        .as_string()
                        .ok_or(RuntimeError::TypeError(
                            "SetGlobal: constant must be a string".into(),
                        ))?
                        .clone();
                    if let Some(v) = self.globals.get_mut(&name) {
                        *v = value;
                    } else {
                        return Err(RuntimeError::UndefinedVariable(name));
                    }
                }
                Opcode::GetLocal(index) => {
                    let value = self.stack_peek(index)?;
                    self.push(value.clone());
                }
                Opcode::SetLocal(index) => {
                    let value = self.stack_top().ok_or(RuntimeError::StackUnderflow)?;
                    self.stack_set(index, value.clone())?;
                }

                Opcode::Jump(offset) => {
                    *self.ip_mut()? += offset;
                    continue;
                }
                Opcode::JumpIfFalse(offset) => {
                    let condition = self.stack_top().ok_or(RuntimeError::StackUnderflow)?;
                    if !condition.is_truthy() {
                        *self.ip_mut()? += offset;
                        continue;
                    }
                }
                Opcode::JumpBack(offset) => {
                    *self.ip_mut()? -= offset;
                    continue;
                }

                Opcode::Call(args_count) => {
                    let function = self
                        .stack_peek_from_top(args_count)?
                        .clone()
                        .to_function()
                        .ok_or(RuntimeError::TypeError(
                            "Can only call function or class".into(),
                        ))?;
                    let arity = function.arity;
                    if arity != args_count {
                        return Err(RuntimeError::WrongArgCount(arity, args_count));
                    }

                    self.call(function, arity)?;

                    // do not increment ip
                    continue;
                }
                Opcode::Return => {
                    let return_value = if self.stack.len() > 1 {
                        self.pop()?
                    } else {
                        Value::Null
                    };

                    if self.frames.len() > 1 {
                        let frame = self.frames.pop().ok_or(RuntimeError::NoCallFrame)?;
                        let pop_count = self.stack.len() - frame.base;
                        for _ in 0..pop_count {
                            self.pop()?;
                        }
                        self.push(return_value);
                    } else {
                        self.pop()?; // main function
                        return Ok(return_value);
                    }
                }
            }
            *self.ip_mut()? += 1;
        }
    }
}

fn trace(ip: usize, opcode: &Opcode, stack: &[Value]) {
    use owo_colors::OwoColorize;

    let op_str = format!("{:?}", opcode).yellow().to_string();
    let stack_str = stack
        .iter()
        .map(|v| format!("{}", v).red().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let ip_str = format!("IP={:02}", ip).bright_blue().to_string();

    trace!("[{}] {:<25} | Stack: [{}]", ip_str, op_str, stack_str);
}
