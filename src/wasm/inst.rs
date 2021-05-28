use super::*;

use std::num::Wrapping;

pub struct Const {
    value: Value,
}

impl Const {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl Instruction for Const {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
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

        let result = match self.result_type {
            PrimitiveType::I32 => {
                let val_0 = Wrapping(unsafe { op_0.v.i32 });
                let val_1 = Wrapping(unsafe { op_1.v.i32 });

                let calc = match self.op_type {
                    IBinOpType::Add => (val_0 + val_1).0,
                    IBinOpType::Sub => (val_0 - val_1).0,
                    IBinOpType::Mul => (val_0 * val_1).0,
                    IBinOpType::Div(_) => todo!(),
                    IBinOpType::Rem(_) => todo!(),
                    IBinOpType::And => (val_0 & val_1).0,
                    IBinOpType::Or => (val_0 | val_1).0,
                    IBinOpType::Xor => (val_0 ^ val_1).0,
                    IBinOpType::Shl => val_0.0 << val_1.0,
                    IBinOpType::Shr(_) => todo!(),
                    IBinOpType::Rotl => val_0.0.rotate_left(val_1.0 as u32),
                    IBinOpType::Rotr => val_0.0.rotate_right(val_1.0 as u32),
                };

                println!("{}", calc);

                Value::from_explicit_type(self.result_type, calc as u64)
            }
            PrimitiveType::I64 => {
                let val_0 = Wrapping(unsafe { op_0.v.i64 });
                let val_1 = Wrapping(unsafe { op_1.v.i64 });

                let calc = match self.op_type {
                    IBinOpType::Add => (val_0 + val_1).0,
                    IBinOpType::Sub => (val_0 - val_1).0,
                    IBinOpType::Mul => (val_0 * val_1).0,
                    IBinOpType::Div(_) => todo!(),
                    IBinOpType::Rem(_) => todo!(),
                    IBinOpType::And => (val_0 & val_1).0,
                    IBinOpType::Or => (val_0 | val_1).0,
                    IBinOpType::Xor => (val_0 ^ val_1).0,
                    IBinOpType::Shl => val_0.0 << val_1.0,
                    IBinOpType::Shr(_) => todo!(),
                    IBinOpType::Rotl => val_0.0.rotate_left(val_1.0 as u32),
                    IBinOpType::Rotr => val_0.0.rotate_right(val_1.0 as u32),
                };

                Value::from_explicit_type(self.result_type, calc as u64)
            }
            _ => {
                panic!("IBinOp result type not an integer type");
            }
        };

        stack.push_value(result);
        log::debug!("Pushed {}", result);

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
        stack.push_value(locals[self.index]);
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
        locals[self.index] = *stack.fetch_value(0)?;
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
        match memory.read(self.result_type, self.load_bitwidth, address) {
            Some(s) => {
                stack.push_value(s);
                Ok(ControlInfo::None)
            }
            None => Ok(ControlInfo::Trap(Trap::MemoryOutOfBounds)),
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
        let value = unsafe { stack.pop_value()?.v.i64 } as u64;
        let address = u32::try_from(stack.pop_value()?)? as u64 + self.offset as u64;
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
