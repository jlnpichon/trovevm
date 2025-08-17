use std::{collections::HashMap, time::SystemTime};

use trove_core::{
    CompiledFunction, RuntimeError, Value,
    function::{Callable, CompiledFunctionKind, ExecContext},
};

pub type NativeFn = fn(&[Value], &mut dyn ExecContext) -> Result<Value, RuntimeError>;

#[derive(Debug, Clone)]
pub struct Native {
    pub func: NativeFn,
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
}

pub fn install_natives(globals: &mut HashMap<String, Value>) {
    define_native(globals, "clock", 0, clock);
}

fn define_native(globals: &mut HashMap<String, Value>, name: &str, arity: usize, func: NativeFn) {
    let native = Native { func };
    let value = Value::Function(CompiledFunction {
        kind: CompiledFunctionKind::Native(Box::new(native)),
        name: name.to_string(),
        arity,
    });

    globals.insert(name.into(), value);
}

fn clock(_: &[Value], _: &mut dyn ExecContext) -> Result<Value, RuntimeError> {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => Ok(Value::Number(n.as_secs() as f64)),
        Err(_) => todo!(),
    }
}
