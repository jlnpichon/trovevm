use std::{collections::HashMap, time::SystemTime};

use trove_core::{
    CompiledFunction, RuntimeError, Value,
    function::{Callable, CompiledFunctionKind, ExecContext},
    value::SharedValue,
};

pub type NativeFn = fn(&[Value], &mut dyn ExecContext) -> Result<Value, RuntimeError>;

#[derive(Debug, Clone)]
pub struct Native {
    pub func: NativeFn,
    pub name: &'static str,
    pub arity: usize,
}

impl Callable for Native {
    fn call(
        &self,
        args: &[Value],
        context: &mut dyn trove_core::function::ExecContext,
    ) -> Result<Value, RuntimeError> {
        (self.func)(args, context)
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

pub fn install_natives(
    globals: &mut HashMap<String, SharedValue>,
    natives: &mut HashMap<String, Box<dyn Callable>>,
) {
    define_native(globals, natives, "clock", 0, clock);
    define_native(globals, natives, "error", 1, error);
    define_native(globals, natives, "map", 0, map);
}

fn define_native(
    globals: &mut HashMap<String, SharedValue>,
    natives: &mut HashMap<String, Box<dyn Callable>>,
    name: &'static str,
    arity: usize,
    func: NativeFn,
) {
    let native = Native { func, name, arity };
    natives.insert(name.into(), Box::new(native));
    globals.insert(
        name.into(),
        SharedValue::from_function(CompiledFunction {
            kind: CompiledFunctionKind::Native(name.to_string()),
            name: name.to_string(),
            arity,
        }),
    );
}

fn clock(_: &[Value], _: &mut dyn ExecContext) -> Result<Value, RuntimeError> {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => Ok(Value::Number(n.as_secs() as f64)),
        Err(_) => todo!(),
    }
}

fn error(args: &[Value], _: &mut dyn ExecContext) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
            name: "error",
        });
    }

    let message = args[0].as_string().ok_or(RuntimeError::TypeMismatch {
        expected: "String",
        got: args[0].type_name(),
        name: "error",
    })?;

    Err(RuntimeError::ContractError(message.to_string()))
}

fn map(_: &[Value], _: &mut dyn ExecContext) -> Result<Value, RuntimeError> {
    Ok(Value::Map(HashMap::new()))
}
