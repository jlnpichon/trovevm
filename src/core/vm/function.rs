use super::{Opcode, Program, Value, native::Native};

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub kind: CompiledFunctionKind,
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Clone)]
pub enum CompiledFunctionKind {
    Bytecode(Program),
    Native(Native),
}

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
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
        match &self.kind {
            CompiledFunctionKind::Bytecode(program) => program.as_slices(),
            CompiledFunctionKind::Native(_) => unreachable!(),
        }
    }

    pub fn program(&self) -> &Program {
        match &self.kind {
            CompiledFunctionKind::Bytecode(program) => program,
            CompiledFunctionKind::Native(_) => unreachable!(),
        }
    }

    pub fn program_mut(&mut self) -> &mut Program {
        match self.kind {
            CompiledFunctionKind::Bytecode(ref mut program) => program,
            CompiledFunctionKind::Native(_) => unreachable!(),
        }
    }
}

impl Default for CompiledFunction {
    fn default() -> Self {
        Self {
            kind: CompiledFunctionKind::Bytecode(Default::default()),
            name: String::from("script"),
            arity: Default::default(),
        }
    }
}

impl<const N: usize> From<(&[Opcode; N], Vec<Value>)> for CompiledFunction {
    fn from(source: (&[Opcode; N], Vec<Value>)) -> Self {
        Self {
            kind: CompiledFunctionKind::Bytecode(Program::from(source)),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for CompiledFunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompiledFunctionKind::Bytecode(_) => write!(f, "fn"),
            CompiledFunctionKind::Native(_) => write!(f, "native"),
        }
    }
}
