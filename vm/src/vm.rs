use std::collections::HashMap;

use tracing::trace;

use trove_core::{
    Address, CompiledFunction, ContractEnv, ContractInstance, Opcode, Program, RuntimeError, Value,
    WorldState,
    contract::{CompiledContract, ContractOrInstance, is_builtin_var},
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
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        let mut natives = HashMap::new();

        install_natives(&mut globals, &mut natives);

        Self {
            stack: Vec::with_capacity(1024),
            globals,
            natives,
            frames: vec![],
        }
    }

    pub fn deploy(
        &mut self,
        sender: Address,
        contract: CompiledContract,
        args: Vec<Value>,
        world_state: &mut WorldState,
    ) -> Result<ContractInstance, RuntimeError> {
        let address = world_state.generate_address();
        world_state
            .storage
            .lock()
            .init_instance(sender, address, &contract);

        let instance =
            ContractInstance::new(address, contract.clone(), world_state.storage.clone());

        let env = ContractEnv {
            sender: 0,
            self_address: address,
            instance: instance.clone(),
            value: 0,
            block_number: 0,
            timestamp: 0,
        };

        if let Some(init_method) = instance.get_method("init") {
            self.call_method(
                init_method,
                Value::ContractInstance(instance.clone()),
                args,
                Some(env),
            )?;
        }

        world_state
            .instances
            .insert(instance.address, instance.clone());

        Ok(instance)
    }

    pub fn sandbox_call(
        &mut self,
        target: ContractOrInstance,
        method_name: &str,
        sender: Option<Address>,
        value: Option<u64>,
        args: Vec<Value>,
        init_args: Option<Vec<Value>>,
    ) -> Result<Value, RuntimeError> {
        let (instance, do_init) = match target {
            ContractOrInstance::Contract(contract) => {
                let storage = contract.default_storage();
                (ContractInstance::new(0, contract, storage), true)
            }
            ContractOrInstance::Instance(instance) => (instance, false),
        };

        let method = instance
            .get_method(method_name)
            .ok_or(RuntimeError::UndefinedVariable(method_name.to_string()))?;

        let env = ContractEnv {
            sender: sender.unwrap_or(0),
            self_address: instance.address,
            instance: instance.clone(),
            value: value.unwrap_or(0),
            block_number: 0,
            timestamp: 0,
        };

        if do_init {
            if let Some(init_method) = instance.get_method("init") {
                let args = init_args.unwrap_or_default();
                self.call_method(
                    init_method,
                    Value::ContractInstance(instance.clone()),
                    args,
                    Some(env.clone()),
                )?;
            }
        }

        self.call_method(
            method,
            Value::ContractInstance(instance.clone()),
            args,
            Some(env),
        )
    }

    fn inject_env(&mut self, env: &ContractEnv, sender_balance: f64) {
        let mut injected_globals = self.globals.clone();
        inject_builtins_variables(&mut injected_globals, env, sender_balance);
        self.globals = injected_globals;
    }

    fn prepare_transaction(
        &self,
        env: &ContractEnv,
    ) -> Result<(f64, f64, HashMap<String, Value>), RuntimeError> {
        let storage = env.instance.storage.clone(); // Arc::clone
        let sender_balance = storage
            .lock()
            .get(env.sender, "balance")
            .ok_or(RuntimeError::UndefinedVariable("sender.balance".into()))?
            .as_number()
            .ok_or(RuntimeError::TypeError("balance must be a number".into()))?;

        if sender_balance < (env.value as f64) {
            return Err(RuntimeError::InsufficientFunds);
        }

        let instance_balance = storage
            .lock()
            .get(env.instance.address, "balance")
            .ok_or(RuntimeError::UndefinedVariable("instance.balance".into()))?
            .as_number()
            .ok_or(RuntimeError::TypeError("balance must be a number".into()))?;

        let saved_globals = self.globals.clone();

        Ok((sender_balance, instance_balance, saved_globals))
    }

    fn commit_transaction(
        &self,
        env: &ContractEnv,
        sender_balance: f64,
        instance_balance: f64,
    ) -> Result<(), RuntimeError> {
        let mut storage = env.instance.storage.lock();
        storage.set(
            env.sender,
            "balance",
            Value::Number(sender_balance - env.value as f64),
        );
        storage.set(
            env.instance.address,
            "balance",
            Value::Number(instance_balance + env.value as f64),
        );
        Ok(())
    }

    pub fn call_contract_method(
        &mut self,
        method_name: &str,
        args: Vec<Value>,
        env: ContractEnv,
    ) -> Result<Value, RuntimeError> {
        let (sender_balance, instance_balance, saved_globals) = self.prepare_transaction(&env)?;

        self.inject_env(&env, sender_balance);

        let method = env
            .instance
            .get_method(method_name)
            .ok_or(RuntimeError::UndefinedVariable(method_name.to_string()))?;

        let result = self.call_method(
            method,
            Value::ContractInstance(env.instance.clone()),
            args,
            Some(env.clone()),
        )?;

        self.commit_transaction(&env, sender_balance, instance_balance)?;

        self.globals = saved_globals;

        Ok(result)
    }

    fn call_method(
        &mut self,
        method: CompiledFunction,
        this: Value,
        args: Vec<Value>,
        env: Option<ContractEnv>,
    ) -> Result<Value, RuntimeError> {
        let function_obj = Value::Function(method.clone());
        self.push(function_obj);

        let args_len = args.len();
        self.push(this);
        for arg in args {
            self.push(arg);
        }

        self.call(method, args_len + 1 /* this */, env)?;
        let result = self.run_loop()?;

        self.frames.pop();

        Ok(result)
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

    fn call(
        &mut self,
        function: CompiledFunction,
        args: usize,
        env: Option<ContractEnv>,
    ) -> Result<(), RuntimeError> {
        match function.kind {
            CompiledFunctionKind::Bytecode(_) => {
                self.frames.push(CallFrame {
                    function,
                    ip: 0,
                    base: self.stack.len() - args - 1,
                    env,
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

        self.call(function, 0, None)?;

        self.run_loop()
    }

    pub fn eval(&mut self, function: CompiledFunction) -> Result<Value, RuntimeError> {
        todo!()
    }

    fn push_builtin(&mut self, name: &str) -> Result<(), RuntimeError> {
        let value = self
            .globals
            .get(name)
            .ok_or(RuntimeError::UndefinedBuiltinVariable(name.to_string()))?
            .clone();
        self.push(value);
        Ok(())
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
                    let field_name =
                        self.constant_get(index)?
                            .as_string()
                            .ok_or(RuntimeError::TypeError(
                                "GetField: constant must be a string".into(),
                            ))?;

                    let mut this = self
                        .stack_top()
                        .ok_or(RuntimeError::StackUnderflow)?
                        .as_instance()
                        .ok_or(RuntimeError::TypeError(
                            "'this' must be a contract instance".into(),
                        ))?
                        .clone();

                    if !this.var_exists(field_name) {
                        return Err(RuntimeError::UndefinedVariable(field_name.clone()));
                    }

                    let value = this
                        .get_field(field_name)
                        .ok_or(RuntimeError::UndefinedVariable(field_name.clone()))?;

                    self.push(value);
                }
                Opcode::SetField(index) => {
                    let field_name = self
                        .constant_get(index)?
                        .as_string()
                        .ok_or(RuntimeError::TypeError(
                            "GetField: constant must be a string".into(),
                        ))?
                        .clone();

                    if is_builtin_var(&field_name) {
                        return Err(RuntimeError::ReadOnlyField(field_name.to_string()));
                    }

                    let value = self.pop()?;
                    let mut this = self
                        .stack_top()
                        .ok_or(RuntimeError::StackUnderflow)?
                        .as_instance()
                        .ok_or(RuntimeError::TypeError(
                            "'this' must be a contract instance".into(),
                        ))?
                        .clone();

                    if !this.var_exists(&field_name) {
                        return Err(RuntimeError::UndefinedVariable(field_name.clone()));
                    }

                    this.set_field(&field_name, value.clone());
                    self.push(value);
                }

                Opcode::GetSender => self.push_builtin("msg.sender")?,
                Opcode::GetValue => self.push_builtin("msg.value")?,
                Opcode::GetData => self.push_builtin("msg.data")?,
                Opcode::GetBalance => self.push_builtin("msg.balance")?,
                Opcode::GetBlockNumber => self.push_builtin("block.number")?,
                Opcode::GetBlockTimestamp => self.push_builtin("block.timestamp")?,
                Opcode::GetBlockHash => self.push_builtin("block.hash")?,
                Opcode::GetGasLimit => self.push_builtin("block.gas_limit")?,
                Opcode::GetCoinBase => self.push_builtin("block.coinbase")?,
                Opcode::Balance => {
                    let addr = self.pop()?.as_number().ok_or(RuntimeError::TypeError(
                        "Balance works only on address".to_string(),
                    ))?;

                    let instance = self
                        .frames
                        .last()
                        .and_then(|frame| frame.env.clone())
                        .ok_or(RuntimeError::NoContractInstance)?
                        .instance;

                    let balance = instance.storage.lock().get(addr as Address, "balance");
                    self.push(balance.unwrap_or(Value::Null));
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

                    self.call(
                        function,
                        arity,
                        self.frames.last().and_then(|f| f.env.clone()),
                    )?;

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

fn inject_builtins_variables(
    injected: &mut HashMap<String, Value>,
    env: &ContractEnv,
    sender_balance: f64,
) {
    injected.insert("msg.sender".into(), Value::Number(env.sender as f64));
    injected.insert("msg.balance".into(), Value::Number(sender_balance));
    injected.insert("msg.value".into(), Value::Number(env.value as f64));
    injected.insert(
        "block.number".into(),
        Value::Number(env.block_number as f64),
    );
    injected.insert(
        "block.timestamp".into(),
        Value::Number(env.timestamp as f64),
    );
}
