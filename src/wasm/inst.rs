use super::*;

use std::ops::Neg;

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
        _: &Vec<Function>,
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
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op_1 = stack.pop_value()?;
        let op_0 = stack.pop_value()?;
        if !((op_0.t, op_1.t) == (op_1.t, self.result_type)) {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.result_type {
            PrimitiveType::I32 => {
                let val_0 = op_0.as_i32_unchecked();
                let val_1 = op_1.as_i32_unchecked();

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
                    IBinOpType::Rem(Signedness::Signed) => {
                        if val_1 == 0 {
                            return Ok(ControlInfo::Trap(Trap::UndefinedDivision));
                        } else {
                            val_0.wrapping_rem(val_1)
                        }
                    }
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
                let val_0 = op_0.as_i64_unchecked();
                let val_1 = op_1.as_i64_unchecked();

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
                    IBinOpType::Rem(Signedness::Signed) => {
                        if val_1 == 0 {
                            return Ok(ControlInfo::Trap(Trap::UndefinedDivision));
                        } else {
                            val_0.wrapping_rem(val_1)
                        }
                    }
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
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op_1 = stack.pop_value()?;
        let op_0 = stack.pop_value()?;
        if !((op_0.t, op_1.t) == (op_1.t, self.result_type)) {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.result_type {
            PrimitiveType::F32 => {
                let val_0 = op_0.as_f32_unchecked();
                let val_1 = op_1.as_f32_unchecked();

                let calc = match self.op_type {
                    FBinOpType::Add => val_0 + val_1,
                    FBinOpType::Sub => val_0 - val_1,
                    FBinOpType::Mul => val_0 * val_1,
                    FBinOpType::Div => val_0 / val_1,
                    FBinOpType::Min => {
                        if val_0.eq(&val_1) {
                            f32::from_bits(val_0.to_bits() | val_1.to_bits())
                        } else if val_0 < val_1 {
                            val_0
                        } else if val_0 > val_1 {
                            val_1
                        } else {
                            f32::NAN
                        }
                    }
                    FBinOpType::Max => {
                        if val_0.eq(&val_1) {
                            f32::from_bits(val_0.to_bits() & val_1.to_bits())
                        } else if val_0 > val_1 {
                            val_0
                        } else if val_0 < val_1 {
                            val_1
                        } else {
                            f32::NAN
                        }
                    }
                    FBinOpType::CopySign => val_0.copysign(val_1),
                };

                Value::from(calc)
            }
            PrimitiveType::F64 => {
                let val_0 = op_0.as_f64_unchecked();
                let val_1 = op_1.as_f64_unchecked();

                let calc = match self.op_type {
                    FBinOpType::Add => val_0 + val_1,
                    FBinOpType::Sub => val_0 - val_1,
                    FBinOpType::Mul => val_0 * val_1,
                    FBinOpType::Div => val_0 / val_1,
                    FBinOpType::Min => {
                        if val_0.eq(&val_1) {
                            f64::from_bits(val_0.to_bits() | val_1.to_bits())
                        } else if val_0 < val_1 {
                            val_0
                        } else if val_0 > val_1 {
                            val_1
                        } else {
                            f64::NAN
                        }
                    }
                    FBinOpType::Max => {
                        if val_0.eq(&val_1) {
                            f64::from_bits(val_0.to_bits() & val_1.to_bits())
                        } else if val_0 > val_1 {
                            val_0
                        } else if val_0 < val_1 {
                            val_1
                        } else {
                            f64::NAN
                        }
                    }
                    FBinOpType::CopySign => val_0.copysign(val_1),
                };

                Value::from(calc)
            }
            _ => unreachable!(),
        };

        stack.push_value(result);
        log::debug!("Pushed {}", result);

        Ok(ControlInfo::None)
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
    arg_type: PrimitiveType,
    op_type: RelOpType,
}

impl RelOp {
    pub fn new(arg_type: PrimitiveType, op_type: RelOpType) -> Self {
        Self { arg_type, op_type }
    }
}

impl Instruction for RelOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op_1 = stack.pop_value()?;
        let op_0 = stack.pop_value()?;
        if op_0.t != op_1.t {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.arg_type {
            PrimitiveType::F32 => {
                let val_0 = op_0.as_f32_unchecked();
                let val_1 = op_1.as_f32_unchecked();

                let calc = match self.op_type {
                    RelOpType::Eq => val_0.eq(&val_1),
                    RelOpType::Neq => val_0.eq(&val_1),
                    RelOpType::Lt(Signedness::Signed) => val_0 < val_1,
                    RelOpType::Gt(Signedness::Signed) => val_0 > val_1,
                    RelOpType::Le(Signedness::Signed) => val_0 <= val_1,
                    RelOpType::Ge(Signedness::Signed) => val_0 >= val_1,
                    _ => unreachable!(),
                };

                Value::from_explicit_type(PrimitiveType::I32, calc as u64)
            }
            PrimitiveType::F64 => {
                let val_0 = op_0.as_f64_unchecked();
                let val_1 = op_1.as_f64_unchecked();

                let calc = match self.op_type {
                    RelOpType::Eq => val_0.eq(&val_1),
                    RelOpType::Neq => val_0.eq(&val_1),
                    RelOpType::Lt(Signedness::Signed) => val_0 < val_1,
                    RelOpType::Gt(Signedness::Signed) => val_0 > val_1,
                    RelOpType::Le(Signedness::Signed) => val_0 <= val_1,
                    RelOpType::Ge(Signedness::Signed) => val_0 >= val_1,
                    _ => unreachable!(),
                };

                Value::from_explicit_type(PrimitiveType::I32, calc as u64)
            }
            PrimitiveType::I32 => {
                let val_0 = op_0.as_i32_unchecked();
                let val_1 = op_1.as_i32_unchecked();

                type UnsignedT = u32;
                let calc = match self.op_type {
                    RelOpType::Eq => val_0 == val_1,
                    RelOpType::Neq => val_0 != val_1,
                    RelOpType::Lt(Signedness::Signed) => val_0 < val_1,
                    RelOpType::Gt(Signedness::Signed) => val_0 > val_1,
                    RelOpType::Le(Signedness::Signed) => val_0 <= val_1,
                    RelOpType::Ge(Signedness::Signed) => val_0 >= val_1,
                    RelOpType::Lt(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) < (val_1 as UnsignedT)
                    }
                    RelOpType::Gt(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) > (val_1 as UnsignedT)
                    }
                    RelOpType::Le(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) <= (val_1 as UnsignedT)
                    }
                    RelOpType::Ge(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) >= (val_1 as UnsignedT)
                    }
                };

                Value::from_explicit_type(PrimitiveType::I32, calc as u64)
            }
            PrimitiveType::I64 => {
                let val_0 = op_0.as_i64_unchecked();
                let val_1 = op_1.as_i64_unchecked();

                type UnsignedT = u64;
                let calc = match self.op_type {
                    RelOpType::Eq => val_0 == val_1,
                    RelOpType::Neq => val_0 != val_1,
                    RelOpType::Lt(Signedness::Signed) => val_0 < val_1,
                    RelOpType::Gt(Signedness::Signed) => val_0 > val_1,
                    RelOpType::Le(Signedness::Signed) => val_0 <= val_1,
                    RelOpType::Ge(Signedness::Signed) => val_0 >= val_1,
                    RelOpType::Lt(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) < (val_1 as UnsignedT)
                    }
                    RelOpType::Gt(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) > (val_1 as UnsignedT)
                    }
                    RelOpType::Le(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) <= (val_1 as UnsignedT)
                    }
                    RelOpType::Ge(Signedness::Unsigned) => {
                        (val_0 as UnsignedT) >= (val_1 as UnsignedT)
                    }
                };

                Value::from_explicit_type(PrimitiveType::I32, calc as u64)
            }
        };

        stack.push_value(result);
        log::debug!("Pushed {}", result);

        Ok(ControlInfo::None)
    }
}

pub struct ITestOpEqz {
    arg_type: PrimitiveType,
}

impl ITestOpEqz {
    pub fn new(arg_type: PrimitiveType) -> Self {
        Self { arg_type }
    }
}

impl Instruction for ITestOpEqz {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op = stack.pop_value()?;
        if op.t != self.arg_type {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.arg_type {
            PrimitiveType::I32 => {
                let val_0 = op.as_i32_unchecked();
                let calc = val_0 == 0_i32;
                Value::from_explicit_type(PrimitiveType::I32, calc as u64)
            }
            PrimitiveType::I64 => {
                let val_0 = op.as_i64_unchecked();
                let calc = val_0 == 0_i64;
                Value::from_explicit_type(PrimitiveType::I32, calc as u64)
            }
            _ => unreachable!(),
        };

        stack.push_value(result);
        log::debug!("Pushed {}", result);
        Ok(ControlInfo::None)
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
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op = stack.pop_value()?;
        if op.t != self.result_type {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.result_type {
            PrimitiveType::I32 => {
                let val_0 = op.as_i32_unchecked();

                let calc = match self.op_type {
                    IUnOpType::Clz => val_0.leading_zeros(),
                    IUnOpType::Ctz => val_0.trailing_zeros(),
                    IUnOpType::Popcnt => val_0.count_ones(),
                };

                Value::from_explicit_type(self.result_type, calc as u64)
            }
            PrimitiveType::I64 => {
                let val_0 = op.as_i64_unchecked();

                let calc = match self.op_type {
                    IUnOpType::Clz => val_0.leading_zeros(),
                    IUnOpType::Ctz => val_0.trailing_zeros(),
                    IUnOpType::Popcnt => val_0.count_ones(),
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
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op = stack.pop_value()?;
        if op.t != self.result_type {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.result_type {
            PrimitiveType::F32 => {
                let val_0 = op.as_f32_unchecked();

                let calc = match self.op_type {
                    FUnOpType::Abs => val_0.abs(),
                    FUnOpType::Neg => val_0.neg(),
                    FUnOpType::Sqrt => f32::sqrt(val_0),
                    FUnOpType::Ceil => val_0.ceil(),
                    FUnOpType::Floor => val_0.floor(),
                    FUnOpType::Trunc => val_0.trunc(),
                    // bit magic from reference implementation in OCaml
                    FUnOpType::Nearest => {
                        if val_0 == 0.0 || val_0.is_nan() {
                            val_0
                        } else {
                            let u = val_0.ceil();
                            let d = val_0.floor();
                            let um = (val_0 - u).abs();
                            let dm = (val_0 - d).abs();
                            let c = (u / 2.0).floor().eq(&(u / 2.0));
                            if um < dm || um.eq(&dm) && c {
                                u
                            } else {
                                d
                            }
                        }
                    }
                };

                Value::from(calc)
            }
            PrimitiveType::F64 => {
                let val_0 = op.as_f64_unchecked();

                let calc = match self.op_type {
                    FUnOpType::Abs => val_0.abs(),
                    FUnOpType::Neg => val_0.neg(),
                    FUnOpType::Sqrt => f64::sqrt(val_0),
                    FUnOpType::Ceil => val_0.ceil(),
                    FUnOpType::Floor => val_0.floor(),
                    FUnOpType::Trunc => val_0.trunc(),
                    FUnOpType::Nearest => {
                        if val_0 == 0.0 || val_0.is_nan() {
                            val_0
                        } else {
                            let u = val_0.ceil();
                            let d = val_0.floor();
                            let um = (val_0 - u).abs();
                            let dm = (val_0 - d).abs();
                            let c = (u / 2.0).floor().eq(&(u / 2.0));
                            if um < dm || um.eq(&dm) && c {
                                u
                            } else {
                                d
                            }
                        }
                    }
                };

                Value::from(calc)
            }
            _ => unreachable!(),
        };

        stack.push_value(result);
        log::debug!("Pushed {}", result);

        Ok(ControlInfo::None)
    }
}

// variants declared with `PrimitiveType`s as (source, [result])
pub enum CvtOpType {
    Wrap,
    Extend(Signedness),
    Trunc(Signedness, PrimitiveType, PrimitiveType),
    TruncSat(Signedness, PrimitiveType, PrimitiveType),
    Convert(Signedness, PrimitiveType, PrimitiveType),
    Demote,
    Promote,
    Reinterpret(PrimitiveType), // source type
}

pub struct CvtOp {
    op_type: CvtOpType,
}

impl CvtOp {
    pub fn new(op_type: CvtOpType) -> Self {
        Self { op_type }
    }
}

impl Instruction for CvtOp {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let op = stack.pop_value()?;
        let has_correct_type = match self.op_type {
            CvtOpType::Wrap => op.t == PrimitiveType::I32,
            CvtOpType::Extend(_) => op.t == PrimitiveType::I32,
            CvtOpType::Trunc(_, src, _) => op.t == src,
            CvtOpType::TruncSat(_, src, _) => op.t == src,
            CvtOpType::Convert(_, src, _) => op.t == src,
            CvtOpType::Promote => op.t == PrimitiveType::F32,
            CvtOpType::Demote => op.t == PrimitiveType::F64,
            CvtOpType::Reinterpret(src) => op.t == src,
        };
        if !has_correct_type {
            return Err(Error::Misc("Operand type mismatch"));
        }

        let result = match self.op_type {
            CvtOpType::Wrap => {
                Value::from_explicit_type(PrimitiveType::I32, op.as_i64_unchecked() as u64)
            }
            CvtOpType::Extend(Signedness::Signed) => {
                Value::from_explicit_type(PrimitiveType::I64, op.as_i32_unchecked() as i64 as u64)
            }
            CvtOpType::Extend(Signedness::Unsigned) => {
                Value::from_explicit_type(PrimitiveType::I64, op.as_i32_unchecked() as u32 as u64)
            }
            CvtOpType::Trunc(Signedness::Unsigned, src, dst) => Value::from_explicit_type(
                dst,
                match src {
                    PrimitiveType::F32 => op.as_f32_unchecked() as u32 as u64,
                    PrimitiveType::F64 => op.as_f64_unchecked() as u64,
                    _ => unreachable!(),
                },
            ),
            CvtOpType::Trunc(Signedness::Signed, src, dst) => Value::from_explicit_type(
                dst,
                match src {
                    PrimitiveType::F32 => op.as_f32_unchecked() as i32 as u32 as u64,
                    PrimitiveType::F64 => op.as_f64_unchecked() as i64 as u64,
                    _ => unreachable!(),
                },
            ),
            CvtOpType::Convert(Signedness::Unsigned, src, dst) => match (src, dst) {
                (PrimitiveType::I32, PrimitiveType::F32) => {
                    Value::from(op.as_i32_unchecked() as f32)
                }
                (PrimitiveType::I32, PrimitiveType::F64) => {
                    Value::from(op.as_i32_unchecked() as f64)
                }
                (PrimitiveType::I64, PrimitiveType::F32) => {
                    Value::from(op.as_i64_unchecked() as f32)
                }
                (PrimitiveType::I64, PrimitiveType::F64) => {
                    Value::from(op.as_i64_unchecked() as f64)
                }
                _ => unreachable!(),
            },
            CvtOpType::Convert(Signedness::Signed, src, dst) => match (src, dst) {
                (PrimitiveType::I32, PrimitiveType::F32) => {
                    Value::from(op.as_i32_unchecked() as u32 as f32)
                }
                (PrimitiveType::I32, PrimitiveType::F64) => {
                    Value::from(op.as_i32_unchecked() as u32 as f64)
                }
                (PrimitiveType::I64, PrimitiveType::F32) => {
                    Value::from(op.as_i64_unchecked() as u64 as f32)
                }
                (PrimitiveType::I64, PrimitiveType::F64) => {
                    Value::from(op.as_i64_unchecked() as u64 as f64)
                }
                _ => unreachable!(),
            },
            CvtOpType::TruncSat(_, _, _) => unimplemented!(),
            CvtOpType::Promote => Value::from(op.as_f32_unchecked() as f64),
            CvtOpType::Demote => Value::from(op.as_f64_unchecked() as f32),
            CvtOpType::Reinterpret(src) => match src {
                PrimitiveType::I32 => Value {
                    t: PrimitiveType::F32,
                    v: InternalValue::from(op.as_i32_unchecked()),
                },
                PrimitiveType::F32 => Value {
                    t: PrimitiveType::I32,
                    v: InternalValue::from(op.as_f32_unchecked()),
                },
                PrimitiveType::I64 => Value {
                    t: PrimitiveType::F64,
                    v: InternalValue::from(op.as_i64_unchecked()),
                },
                PrimitiveType::F64 => Value {
                    t: PrimitiveType::I64,
                    v: InternalValue::from(op.as_f64_unchecked()),
                },
            },
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
        _: &Vec<Function>,
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
        _: &Vec<Function>,
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
        _: &Vec<Function>,
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
        _: &Vec<Function>,
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
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        //TODO: popped values need to be checked
        let value = stack.pop_value()?.as_i64_unchecked() as u64;
        let address = u32::try_from(stack.pop_value()?)? as u64 + self.offset as u64;
        match memory.write(value, self.bitwidth, address) {
            Some(_) => Ok(ControlInfo::None),
            None => Ok(ControlInfo::Trap(Trap::MemoryOutOfBounds)),
        }
    }
}

pub struct Branch {
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
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        Ok(ControlInfo::Branch(self.branch_index))
    }
}

pub struct BranchIf {
    branch_index: u32,
}

impl BranchIf {
    pub fn new(branch_index: u32) -> Self {
        Self { branch_index }
    }
}

impl Instruction for BranchIf {
    fn execute(
        &self,
        stack: &mut Stack,
        _: &mut Memory,
        _: &mut Vec<Value>,
        _: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        let condition = stack.pop_value()?.as_i64_unchecked() as u64;
        if condition == 0 {
            Ok(ControlInfo::None)
        } else {
            Ok(ControlInfo::Branch(self.branch_index))
        }
    }
}

pub struct Call {
    function_index: usize,
}

impl Call {
    pub fn new(function_index: usize) -> Self {
        Self { function_index }
    }
}

impl Instruction for Call {
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        _: &mut Vec<Value>,
        functions: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        log::debug!("Calling function with index {}", self.function_index);
        let called_function = &functions[self.function_index];
        let mut args = Vec::new();
        for _ in 0..called_function.num_params() {
            args.push(stack.pop_value()?);
        }
        args.reverse();
        stack.push_value(called_function.call(functions, memory, args)?);
        Ok(ControlInfo::None)
    }
}

pub struct Return {}

impl Return {
    pub fn new() -> Self {
        Self {}
    }
}

impl Instruction for Return {
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        _: &mut Vec<Value>,
        functions: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        Ok(ControlInfo::Return)
    }
}

pub enum BlockContinuation {
    Loop,
    Branch,
}

pub struct Block {
    continuation: BlockContinuation,
    instructions: Vec<Box<dyn Instruction>>,
}

impl Block {
    pub fn new(continuation: BlockContinuation, instructions: Vec<Box<dyn Instruction>>) -> Self {
        Self {
            continuation,
            instructions,
        }
    }
}

impl Instruction for Block {
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        locals: &mut Vec<Value>,
        functions: &Vec<Function>,
    ) -> Result<ControlInfo, Error> {
        // This outer loop is being used more as a goto than an actual loop.
        let mut loop_restart;
        loop {
            loop_restart = false;
            for inst in &self.instructions {
                match inst.execute(stack, memory, locals, functions) {
                    // Instruction returned a branch
                    Ok(ControlInfo::Branch(branch_levels)) => {
                        if branch_levels == 0 {
                            // If we are a loop, continue execution from the beginning of our instrucitons.
                            // Otherwise, halt execution and return to our parent block.
                            match self.continuation {
                                BlockContinuation::Loop => {
                                    log::debug!("Branching to loop at depth 0");
                                    loop_restart = true;
                                }
                                BlockContinuation::Branch => {
                                    log::debug!("Branching out of a block with depth 0");
                                    return Ok(ControlInfo::None);
                                }
                            }
                        } else {
                            // Both loops and branches need to pass the control information up to the higher block
                            let new_depth = branch_levels - 1;
                            log::debug!(
                                "Branching out of block from branch depth {} to {}",
                                branch_levels,
                                new_depth
                            );
                            return Ok(ControlInfo::Branch(new_depth));
                        }
                    }
                    Ok(ControlInfo::Return) => {
                        // Unwrap up to the function's call handler
                        log::debug!("Unwrapping return!");
                        return Ok(ControlInfo::Return);
                    }
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                }
                if loop_restart {
                    break;
                }
            }
            // Getting here implies that we need to fall through the block
            if !loop_restart {
                break;
            }
        }
        Ok(ControlInfo::None)
    }
}
