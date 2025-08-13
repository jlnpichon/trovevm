use super::{Program, Value};

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub program: Program,
    pub arity: usize,
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: Box<CompiledFunction>,
    pub slots: Vec<Value>,
}
