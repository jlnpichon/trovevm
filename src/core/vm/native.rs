use std::{collections::HashMap, time::SystemTime};

use super::{RuntimeError, VM, Value, function::CompiledFunction};

pub type Native = fn(&mut VM, &[Value]) -> Result<Value, RuntimeError>;

pub fn install_natives(globals: &mut HashMap<String, Value>) {
    define_native(globals, "clock", 0, clock);
}

fn define_native(globals: &mut HashMap<String, Value>, name: &str, arity: usize, ptr: Native) {
    let value = Value::Function(CompiledFunction {
        kind: super::function::CompiledFunctionKind::Native(ptr),
        name: name.to_string(),
        arity,
    });

    globals.insert(name.into(), value);
}

fn clock(_: &mut VM, _: &[Value]) -> Result<Value, RuntimeError> {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => Ok(Value::Number(n.as_secs() as f64)),
        Err(_) => todo!(),
    }
}
