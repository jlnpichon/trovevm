use std::collections::HashMap;

use tracing::trace;

use trove_core::{
    Address, CompiledFunction, ContractInstance, Opcode, Program, RuntimeError, Value, WorldState,
    function::{Callable, CompiledFunctionKind, ExecContext},
    value::Op,
};

use crate::{function::CallFrame, native::install_natives};

#[derive(Debug)]
pub struct VM {
    stack: Vec<Value>, // TODO: limit
    globals: HashMap<String, Value>,
    natives: HashMap<String, Box<dyn Callable>>,
    frames: Vec<CallFrame>, // TODO: limit
    world_state: WorldState,
}

impl Default for VM {
    fn default() -> Self {
        Self::new(WorldState::default())
    }
}

impl VM {
    pub fn new(world_state: WorldState) -> Self {
        let mut globals = HashMap::new();
        let mut natives = HashMap::new();
        install_natives(&mut globals, &mut natives);

        Self {
            stack: Vec::with_capacity(1024),
            globals,
            natives,
            frames: vec![],
            world_state,
        }
    }

    #[cfg(feature = "tokio-async")]
    pub async fn deploy(
        &mut self,
        sender: Address,
        contract_address: Address,
        args: Vec<Value>,
    ) -> Result<ContractInstance, RuntimeError> {
        let contract = self
            .world_state
            .registry
            .get(&contract_address)
            .ok_or(RuntimeError::ContractNotFound)?
            .clone();

        let instance_address = self.world_state.generate_address();
        let instance = ContractInstance::new(
            instance_address,
            contract.clone(),
            self.world_state.storage.clone(),
        );

        self.world_state
            .storage
            .lock()
            .await
            .init_instance(instance_address);

        if let Some(method) = instance.get_method("init") {
            self.call_method(method, Value::ContractInstance(instance.clone()), args)?;
        }

        Ok(instance)
    }

    #[cfg(not(feature = "tokio-async"))]
    pub fn deploy(
        &mut self,
        sender: Address,
        contract_address: Address,
        args: Vec<Value>,
    ) -> Result<ContractInstance, RuntimeError> {
        let contract = self
            .world_state
            .registry
            .get(&contract_address)
            .ok_or(RuntimeError::ContractNotFound)?;

        let instance_address = self.world_state.generate_address(sender);
        let instance = ContractInstance::new(
            instance_address,
            contract.clone(),
            self.world_state.storage.clone(),
        );

        self.world_state
            .storage
            .lock()
            .init_instance(instance_address);

        if let Some(method) = instance.get_method("init") {
            self.call_method(method, Value::ContractInstance(instance.clone()), args)?;
        }

        Ok(instance)
    }

    fn call_method(
        &mut self,
        method: CompiledFunction,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        self.push(this);
        for arg in args {
            self.push(arg);
        }

        self.call(method.clone(), method.arity)?;
        self.run_loop()
    }

    fn run_transaction(
        &mut self,
        caller: Address,
        instance: &mut ContractInstance,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        self.globals
            .insert("caller".to_string(), Value::String(caller.to_string()));
        /* TODO:
                self.globals
                    .insert("msg_value".to_string(), Value::String());
                self.globals
                    .insert("block_number".to_string(), Value::String());
                self.globals
                    .insert("gas_left".to_string(), Value::String());

        */

        let method = instance
            .get_method(method_name)
            .ok_or(RuntimeError::UndefinedVariable(method_name.to_string()))?;

        self.call_method(method, Value::ContractInstance(instance.clone()), args)
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
        Ok(self
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
            CompiledFunctionKind::Native(name) => {
                let native = self
                    .natives
                    .get(&name)
                    .ok_or(RuntimeError::UndefinedNative(name.to_string()))?
                    .clone();
                let value = native.call(&[], self)?;
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

        self.call(function, 0)?;

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
                Opcode::GetField(index) => {
                    todo!()
                }
                Opcode::SetField(index) => {
                    todo!()
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

impl ExecContext for VM {}

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
