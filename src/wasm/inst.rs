use super::*;

use std::num::Wrapping;

pub struct ConstOp {
    value: Value,
}

impl ConstOp {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl Instruction for ConstOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        println!("execute const, pushing {}", self.value);
        stack.push_value(self.value);
        Ok(ControlInfo::None)
    }
}

pub enum Signedness {
    Signed,
    Unsigned,
}

pub enum IBinOpType {
    Add,
    Sub,
    Mul,
    Div(Signedness),
    Rem(Signedness),
    And,
    Or,
    Xor,
    Shl,
    Shr(Signedness),
    Rotl,
    Rotr,
}

pub struct IBinOp {
    result_type: PrimitiveType,
    op_type: IBinOpType,
}

impl IBinOp {
    pub fn new(result_type: PrimitiveType, op_type: IBinOpType) -> Self {
        Self {
            result_type,
            op_type,
        }
    }
}

impl Instruction for IBinOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        let op_1 = stack.pop_value()?;
        let op_0 = stack.pop_value()?;
        if op_0.t != op_1.t {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let val_0 = Wrapping(unsafe { op_0.v.i64 } as u64);
        let val_1 = Wrapping(unsafe { op_1.v.i64 } as u64);

        let result = match self.op_type {
            IBinOpType::Add => Value::from_explicit_type(self.result_type, (val_0 + val_1).0),
            IBinOpType::Sub => Value::from_explicit_type(self.result_type, (val_0 - val_1).0),
            _ => todo!(),
        };

        println!(
            "execute ibinop, operands {} and {}, result {}",
            val_0, val_1, result
        );

        stack.push_value(result);

        Ok(ControlInfo::None)
    }
}

pub struct LocalGet {
    index: usize,
}

impl LocalGet {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl Instruction for LocalGet {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        locals: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        println!("execute localget, pushing {}", locals[self.index]);
        stack.push_value(locals[self.index].clone());
        Ok(ControlInfo::None)
    }
}

pub struct LocalSet {
    index: usize,
}

impl LocalSet {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl Instruction for LocalSet {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        locals: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        println!("execute localset, popping {}", stack.fetch_value(0)?);
        locals[self.index] = stack.pop_value()?;
        Ok(ControlInfo::None)
    }
}

pub struct LocalTee {
    index: usize,
}

impl LocalTee {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl Instruction for LocalTee {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        locals: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        println!("execute localtee, fetching {}", stack.fetch_value(0)?);
        locals[self.index] = stack.fetch_value(0)?.clone();
        Ok(ControlInfo::None)
    }
}

pub struct Load {
    result_type: PrimitiveType,
    load_bitwidth: u8,
    offset: u32,
}

impl Load {
    pub fn new(result_type: PrimitiveType, load_bitwidth: u8, _align: u32, offset: u32) -> Self {
        debug_assert!(load_bitwidth % 8 == 0);
        match result_type {
            PrimitiveType::I32 => {
                debug_assert!(load_bitwidth <= 32);
            }
            PrimitiveType::I64 => {
                debug_assert!(load_bitwidth <= 64);
            }
            PrimitiveType::F32 => {
                debug_assert!(load_bitwidth == 32);
            }
            PrimitiveType::F64 => {
                debug_assert!(load_bitwidth == 64);
            }
        }
        Self {
            result_type,
            load_bitwidth,
            offset,
        }
    }
}

impl Instruction for Load {
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        let address = u32::try_from(stack.pop_value()?)? as u64 + self.offset as u64;
        println!("execute load, reading address {}", address);
        match memory.read(self.result_type, self.load_bitwidth, address) {
            Some(s) => {
                stack.push_value(s);
                Ok(ControlInfo::None)
            }
            None => return Ok(ControlInfo::Trap(Trap::MemoryOutOfBounds)),
        }
    }
}

pub struct Store {
    bitwidth: u8,
    offset: u32,
}

impl Store {
    pub fn new(bitwidth: u8, _align: u32, offset: u32) -> Self {
        Self { bitwidth, offset }
    }
}

impl Instruction for Store {
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        let address = u32::try_from(stack.pop_value()?)? as u64 + self.offset as u64;
        let value = unsafe { stack.pop_value()?.v.i64 } as u64;
        println!("execute store, writing {} to address {}", value, address);
        match memory.write(value, self.bitwidth, address) {
            Some(_) => Ok(ControlInfo::None),
            None => Ok(ControlInfo::Trap(Trap::MemoryOutOfBounds)),
        }
    }
}

struct Branch {
    branch_index: u32,
}

impl Branch {
    pub fn new(branch_index: u32) -> Self {
        Self { branch_index }
    }
}

impl Instruction for Branch {
    fn execute(
        &self,
        _: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        Ok(ControlInfo::Branch(self.branch_index))
    }
}
