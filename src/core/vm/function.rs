use super::{Opcode, Program, Value};

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub program: Program,
    pub arity: usize,
}

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
    pub is_function: bool,
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: CompiledFunction,
    pub ip: usize,
    pub base: usize,
}

impl CompiledFunction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn as_slices(&self) -> (&[Opcode], &[Value]) {
        self.program.as_slices()
    }
}

impl Default for CompiledFunction {
    fn default() -> Self {
        Self {
            name: String::from("script"),
            program: Default::default(),
            arity: Default::default(),
        }
    }
}

impl<const N: usize> From<(&[Opcode; N], Vec<Value>)> for CompiledFunction {
    fn from(source: (&[Opcode; N], Vec<Value>)) -> Self {
        Self {
            program: Program::from(source),
            ..Default::default()
        }
    }
}
