use std::ops::Index;

use crate::core::ast::Identifier;

use super::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,

    Pop,
    Push(usize),

    DefineGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    GetLocal(usize),
    SetLocal(usize),

    Jump(usize),
    JumpIfFalse(usize),
    JumpBack(usize),

    Return,
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    bytecode: Vec<Opcode>,
    constants: Vec<Value>,
}

impl Program {
    pub fn new() -> Self {
        Program::default()
    }

    pub fn emit_opcode(&mut self, opcode: Opcode) {
        self.bytecode.push(opcode);
    }

    pub fn emit_return(&mut self) {
        self.bytecode.push(Opcode::Return);
    }

    pub fn current_opcode_index(&self) -> usize {
        self.bytecode.len()
    }

    pub fn patch_jump(&mut self, index: usize, destination: usize) {
        let opcode = self.bytecode.get_mut(index).expect("path_jump");
        match opcode {
            Opcode::JumpIfFalse(offset) | Opcode::Jump(offset) => *offset = destination,
            _ => panic!("Expect a jump instruction to patch, got '{:?}'", opcode),
        };
    }

    pub fn emit_null(&mut self) {
        let index = self.define_constant(Value::Null);
        self.bytecode.push(Opcode::Push(index));
    }

    // Constant already exists? Reuse it
    pub fn define_constant(&mut self, value: Value) -> usize {
        if let Some(index) = self.constants.iter().position(|c| *c == value) {
            index
        } else {
            self.constants.push(value);
            self.constants.len() - 1
        }
    }

    pub fn emit_constant(&mut self, value: Value) {
        let index = self.define_constant(value);
        self.emit_opcode(Opcode::Push(index));
    }

    pub fn emit_jump(&mut self, opcode: Opcode) -> usize {
        assert!(matches!(
            opcode,
            Opcode::JumpIfFalse(_) | Opcode::Jump(_) | Opcode::JumpBack(_)
        ));
        self.emit_opcode(opcode);
        self.current_opcode_index() - 1
    }

    pub fn define_global(&mut self, name: &Identifier) {
        let index = self.define_constant(Value::String(name.to_string()));
        self.emit_opcode(Opcode::DefineGlobal(index));
    }

    pub fn len(&self) -> usize {
        self.bytecode.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn constant_get(&self, index: usize) -> Option<&Value> {
        self.constants.get(index)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.bytecode {
            _ => todo!(),
        };
        todo!()
    }

    pub fn as_slices(&self) -> (&[Opcode], &[Value]) {
        (&self.bytecode, &self.constants)
    }
}

impl<const N: usize> From<(&[Opcode; N], Vec<Value>)> for Program {
    fn from((bytecode, constants): (&[Opcode; N], Vec<Value>)) -> Self {
        Self {
            bytecode: bytecode.to_vec(),
            constants,
        }
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Opcode;

    type IntoIter = std::slice::Iter<'a, Opcode>;

    fn into_iter(self) -> Self::IntoIter {
        self.bytecode.iter()
    }
}

impl Index<usize> for Program {
    type Output = Opcode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bytecode[index]
    }
}
