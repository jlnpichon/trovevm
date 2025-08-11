use std::{
    ops::{Deref, Index},
    slice::SliceIndex,
};

use super::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Neg,

    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,

    And,
    Or,

    Pop,
    Push(usize),

    Jump(usize),

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

    fn write(&mut self, opcode: Opcode) {
        todo!()
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
