use trove_core::{CompiledFunction, ContractEnv};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: CompiledFunction,
    pub ip: usize,
    pub base: usize,
    pub env: Option<ContractEnv>,
}
