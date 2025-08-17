use trove_core::CompiledFunction;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: CompiledFunction,
    pub ip: usize,
    pub base: usize,
}
