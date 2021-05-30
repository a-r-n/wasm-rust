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
        if op_0.t != op_1.t || op_0.t != self.result_type {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.result_type {
            PrimitiveType::I32 => {
                let val_0 = unsafe { op_0.v.i32 };
                let val_1 = unsafe { op_1.v.i32 };

                type SignedT = i32;
                type UnsignedT = u32;
                let calc = match self.op_type {
                    IBinOpType::Add => val_0.wrapping_add(val_1),
                    IBinOpType::Sub => val_0.wrapping_sub(val_1),
                    IBinOpType::Mul => val_0.wrapping_mul(val_1),
                    IBinOpType::Div(Signedness::Signed) => match val_0.checked_div(val_1) {
                        // checked_div will catch division by zero and TYPE_MIN / -1
                        Some(v) => v,
                        None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                    },
                    IBinOpType::Div(Signedness::Unsigned) => {
                        match (val_0 as UnsignedT).checked_div(val_1 as UnsignedT) {
                            Some(v) => v as SignedT,
                            None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                        }
                    }
                    IBinOpType::Rem(Signedness::Signed) => match val_0.checked_rem(val_1) {
                        Some(v) => v,
                        None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                    },
                    IBinOpType::Rem(Signedness::Unsigned) => {
                        match (val_0 as UnsignedT).checked_rem(val_1 as UnsignedT) {
                            Some(v) => v as SignedT,
                            None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                        }
                    }
                    IBinOpType::And => (val_0 & val_1),
                    IBinOpType::Or => (val_0 | val_1),
                    IBinOpType::Xor => (val_0 ^ val_1),
                    // shifts are modular in val_1, ie. shifting by 34 == shifting by 2
                    IBinOpType::Shl => val_0.wrapping_shl(val_1 as u32),
                    IBinOpType::Shr(Signedness::Signed) => val_0.wrapping_shr(val_1 as u32),
                    IBinOpType::Shr(Signedness::Unsigned) => {
                        (val_0 as UnsignedT).wrapping_shr(val_1 as u32) as SignedT
                    }
                    IBinOpType::Rotl => val_0.rotate_left(val_1 as u32),
                    IBinOpType::Rotr => val_0.rotate_right(val_1 as u32),
                };

                Value::from_explicit_type(self.result_type, calc as u64)
            }
            PrimitiveType::I64 => {
                let val_0 = unsafe { op_0.v.i64 };
                let val_1 = unsafe { op_1.v.i64 };

                type SignedT = i64;
                type UnsignedT = u64;
                let calc = match self.op_type {
                    IBinOpType::Add => val_0.wrapping_add(val_1),
                    IBinOpType::Sub => val_0.wrapping_sub(val_1),
                    IBinOpType::Mul => val_0.wrapping_mul(val_1),
                    IBinOpType::Div(Signedness::Signed) => match val_0.checked_div(val_1) {
                        // checked_div will catch division by zero and TYPE_MIN / -1
                        Some(v) => v,
                        None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                    },
                    IBinOpType::Div(Signedness::Unsigned) => {
                        match (val_0 as UnsignedT).checked_div(val_1 as UnsignedT) {
                            Some(v) => v as SignedT,
                            None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                        }
                    }
                    IBinOpType::Rem(Signedness::Signed) => match val_0.checked_rem(val_1) {
                        Some(v) => v,
                        None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                    },
                    IBinOpType::Rem(Signedness::Unsigned) => {
                        match (val_0 as UnsignedT).checked_rem(val_1 as UnsignedT) {
                            Some(v) => v as SignedT,
                            None => return Ok(ControlInfo::Trap(Trap::UndefinedDivision)),
                        }
                    }
                    IBinOpType::And => (val_0 & val_1),
                    IBinOpType::Or => (val_0 | val_1),
                    IBinOpType::Xor => (val_0 ^ val_1),
                    // shifts are modular in val_1, ie. shifting by 34 == shifting by 2
                    IBinOpType::Shl => val_0.wrapping_shl(val_1 as u32),
                    IBinOpType::Shr(Signedness::Signed) => val_0.wrapping_shr(val_1 as u32),
                    IBinOpType::Shr(Signedness::Unsigned) => {
                        (val_0 as UnsignedT).wrapping_shr(val_1 as u32) as SignedT
                    }
                    IBinOpType::Rotl => val_0.rotate_left(val_1 as u32),
                    IBinOpType::Rotr => val_0.rotate_right(val_1 as u32),
                };

                Value::from_explicit_type(self.result_type, calc as u64)
            }
            _ => unreachable!(),
        };

        stack.push_value(result);
        log::debug!("Pushed {}", result);

        Ok(ControlInfo::None)
    }
}

pub enum FBinOpType {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    CopySign,
}

pub struct FBinOp {
    result_type: PrimitiveType,
    op_type: FBinOpType,
}

impl FBinOp {
    pub fn new(result_type: PrimitiveType, op_type: FBinOpType) -> Self {
        Self {
            result_type,
            op_type,
        }
    }
}

impl Instruction for FBinOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        unimplemented!()
    }
}

pub enum RelOpType {
    Eq,
    Neq,
    Lt(Signedness),
    Gt(Signedness),
    Le(Signedness),
    Ge(Signedness),
}

pub struct RelOp {
    result_type: PrimitiveType,
    op_type: RelOpType,
}

impl RelOp {
    pub fn new(result_type: PrimitiveType, op_type: RelOpType) -> Self {
        Self {
            result_type,
            op_type,
        }
    }
}

impl Instruction for RelOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        unimplemented!()
    }
}

pub struct ITestOpEqz {
    result_type: PrimitiveType,
}

impl ITestOpEqz {
    pub fn new(result_type: PrimitiveType) -> Self {
        Self { result_type }
    }
}

impl Instruction for ITestOpEqz {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        unimplemented!()
    }
}

pub enum IUnOpType {
    Clz,
    Ctz,
    Popcnt,
}

pub struct IUnOp {
    result_type: PrimitiveType,
    op_type: IUnOpType,
}

impl IUnOp {
    pub fn new(result_type: PrimitiveType, op_type: IUnOpType) -> Self {
        Self {
            result_type,
            op_type,
        }
    }
}

impl Instruction for IUnOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        unimplemented!()
    }
}

pub enum FUnOpType {
    Abs,
    Neg,
    Sqrt,
    Ceil,
    Floor,
    Trunc,
    Nearest,
}

pub struct FUnOp {
    result_type: PrimitiveType,
    op_type: FUnOpType,
}

impl FUnOp {
    pub fn new(result_type: PrimitiveType, op_type: FUnOpType) -> Self {
        Self {
            result_type,
            op_type,
        }
    }
}

impl Instruction for FUnOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        unimplemented!()
    }
}

pub enum CvtOpType {
    Wrap,
    Extend(Signedness),
    Trunc(Signedness),
    TruncSat(Signedness),
    Convert(Signedness),
    Demote,
    Promote,
    Reinterpret,
}

pub struct CvtOp {
    source_type: PrimitiveType,
    result_type: PrimitiveType,
    op_type: CvtOpType,
}

impl CvtOp {
    pub fn new(source_type: PrimitiveType, result_type: PrimitiveType, op_type: CvtOpType) -> Self {
        Self {
            source_type,
            result_type,
            op_type,
        }
    }
}

impl Instruction for CvtOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error> {
        unimplemented!()
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
