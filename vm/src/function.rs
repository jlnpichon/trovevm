use trove_core::{CompiledFunction, ContractInstance};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: CompiledFunction,
    pub ip: usize,
    pub base: usize,
    pub instance: Option<ContractInstance>,
}
