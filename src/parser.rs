use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use crate::error::Error;
use crate::wasm::inst::*;
use crate::wasm::*;

/// Returns (value, length read)
fn parse_unsigned_leb128(bytes: &[u8]) -> (u64, usize) {
    let mut value = 0;
    let mut offset = 0_usize;
    while bytes[offset] & (1_u8 << 7) != 0 {
        value += ((bytes[offset] & 0b01111111) as u64) << (7 * offset);
        offset += 1;
    }
    value += ((bytes[offset] & 0b01111111) as u64) << (7 * offset);
    offset += 1;

    (value, offset)
}

fn parse_signed_leb128(bytes: &[u8]) -> (i64, usize) {
    let mut value = 0;
    let mut offset = 0_usize;
    while bytes[offset] & (1_u8 << 7) != 0 {
        value += ((bytes[offset] & 0b01111111) as u64) << (7 * offset);
        offset += 1;
    }
    value += ((bytes[offset] & 0b01111111) as u64) << (7 * offset);
    offset += 1;

    // sign extension needed if the highest bit of the parsed number is 1
    if (7 * offset) < 64 && bytes[offset - 1] & 1_u8 << 6 != 0 {
        value |= !0_u64 << (7 * offset);
    }

    (value as i64, offset)
}

struct ByteReader {
    content: Vec<u8>,
    offset: usize,
}

trait CheckedFromU64 {
    fn from(u: u64) -> Result<Self, Error>
    where
        Self: Sized;
}

impl CheckedFromU64 for u64 {
    fn from(u: u64) -> Result<Self, Error> {
        Ok(u)
    }
}

impl CheckedFromU64 for u32 {
    fn from(u: u64) -> Result<Self, Error> {
        match Self::try_from(u) {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::IntSizeViolation),
        }
    }
}

impl CheckedFromU64 for usize {
    fn from(u: u64) -> Result<Self, Error> {
        match Self::try_from(u) {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::IntSizeViolation),
        }
    }
}

impl CheckedFromU64 for i64 {
    fn from(u: u64) -> Result<Self, Error> {
        Ok(u as i64)
    }
}

impl CheckedFromU64 for i32 {
    fn from(u: u64) -> Result<Self, Error> {
        match Self::try_from(u) {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::IntSizeViolation),
        }
    }
}

trait CheckedFromI64 {
    fn from(u: i64) -> Result<Self, Error>
    where
        Self: Sized;
}

impl CheckedFromI64 for i64 {
    fn from(u: i64) -> Result<Self, Error> {
        Ok(u)
    }
}

impl CheckedFromI64 for i32 {
    fn from(u: i64) -> Result<Self, Error> {
        match Self::try_from(u) {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::IntSizeViolation),
        }
    }
}

macro_rules! inst {
    ($x:expr) => {
        Ok(Some(Box::new($x)))
    };
}

impl ByteReader {
    fn new(content: &[u8]) -> Self {
        Self {
            content: Vec::from(content),
            offset: 0,
        }
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let byte = match self.content.get(self.offset) {
            Some(n) => n,
            None => {
                return Err(Error::EndOfData);
            }
        };
        self.offset += 1;
        Ok(*byte)
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        for _ in 0..count {
            bytes.push(self.read_byte()?);
        }
        Ok(bytes)
    }

    fn read_int<I: CheckedFromU64>(&mut self) -> Result<I, Error> {
        let (value, read_bytes) = parse_unsigned_leb128(&self.content[self.offset..]);
        self.offset += read_bytes;
        Ok(I::from(value)?)
    }

    // same as `read_int`, but uses signed leb128 decoding
    fn read_signed_int<I: CheckedFromI64>(&mut self) -> Result<I, Error> {
        let (value, read_bytes) = parse_signed_leb128(&self.content[self.offset..]);
        self.offset += read_bytes;
        Ok(I::from(value)?)
    }

    fn read_f32(&mut self) -> Result<f32, Error> {
        let value = f32::from_le_bytes(
            (&self.content[self.offset..self.offset + 4])
                .try_into()
                .map_err(|_| Error::FloatSizeViolation)?,
        );
        self.offset += 4;
        Ok(value)
    }

    fn read_f64(&mut self) -> Result<f64, Error> {
        let value = f64::from_le_bytes(
            (&self.content[self.offset..self.offset + 8])
                .try_into()
                .map_err(|_| Error::FloatSizeViolation)?,
        );
        self.offset += 8;
        Ok(value)
    }

    fn read_inst(&mut self) -> Result<Option<Box<dyn Instruction>>, Error> {
        let opcode = self.read_byte()?;
        match opcode {
            0x0B => Ok(None),
            // 0x0C => inst!()
            0x20 => inst!(LocalGet::new(self.read_int()?)),
            0x21 => inst!(LocalSet::new(self.read_int()?)),
            0x22 => inst!(LocalTee::new(self.read_int()?)),
            0x28 => inst!(Load::new(
                PrimitiveType::I32,
                32,
                self.read_int()?,
                self.read_int()?
            )),
            0x29 => inst!(Load::new(
                PrimitiveType::I64,
                64,
                self.read_int()?,
                self.read_int()?
            )),
            0x2A => inst!(Load::new(
                PrimitiveType::F32,
                32,
                self.read_int()?,
                self.read_int()?
            )),
            0x2B => inst!(Load::new(
                PrimitiveType::F64,
                64,
                self.read_int()?,
                self.read_int()?
            )),
            0x36 => inst!(Store::new(32, self.read_int()?, self.read_int()?)),
            0x41 => inst!(Const::new(Value::new(self.read_signed_int::<i32>()?))),
            0x42 => inst!(Const::new(Value::new(self.read_signed_int::<i64>()?))),
            0x43 => inst!(Const::new(Value::new(self.read_f32()?))),
            0x44 => inst!(Const::new(Value::new(self.read_f64()?))),
            0x45 => inst!(ITestOpEqz::new(PrimitiveType::I32)),
            0x46 => inst!(RelOp::new(PrimitiveType::I32, RelOpType::Eq)),
            0x47 => inst!(RelOp::new(PrimitiveType::I32, RelOpType::Neq)),
            0x48 => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Lt(Signedness::Signed)
            )),
            0x49 => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Lt(Signedness::Unsigned)
            )),
            0x4A => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Gt(Signedness::Signed)
            )),
            0x4B => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Gt(Signedness::Unsigned)
            )),
            0x4C => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Le(Signedness::Signed)
            )),
            0x4D => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Le(Signedness::Unsigned)
            )),
            0x4E => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Ge(Signedness::Signed)
            )),
            0x4F => inst!(RelOp::new(
                PrimitiveType::I32,
                RelOpType::Ge(Signedness::Unsigned)
            )),
            0x50 => inst!(ITestOpEqz::new(PrimitiveType::I64)),
            0x51 => inst!(RelOp::new(PrimitiveType::I64, RelOpType::Eq)),
            0x52 => inst!(RelOp::new(PrimitiveType::I64, RelOpType::Neq)),
            0x53 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Lt(Signedness::Signed)
            )),
            0x54 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Lt(Signedness::Unsigned)
            )),
            0x55 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Gt(Signedness::Signed)
            )),
            0x56 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Gt(Signedness::Unsigned)
            )),
            0x57 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Le(Signedness::Signed)
            )),
            0x58 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Le(Signedness::Unsigned)
            )),
            0x59 => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Ge(Signedness::Signed)
            )),
            0x5A => inst!(RelOp::new(
                PrimitiveType::I64,
                RelOpType::Ge(Signedness::Unsigned)
            )),
            0x5B => inst!(RelOp::new(PrimitiveType::F32, RelOpType::Eq)),
            0x5C => inst!(RelOp::new(PrimitiveType::F32, RelOpType::Neq)),
            0x5D => inst!(RelOp::new(
                PrimitiveType::F32,
                RelOpType::Lt(Signedness::Signed)
            )),
            0x5E => inst!(RelOp::new(
                PrimitiveType::F32,
                RelOpType::Gt(Signedness::Signed)
            )),
            0x5F => inst!(RelOp::new(
                PrimitiveType::F32,
                RelOpType::Le(Signedness::Signed)
            )),
            0x60 => inst!(RelOp::new(
                PrimitiveType::F32,
                RelOpType::Ge(Signedness::Signed)
            )),
            0x61 => inst!(RelOp::new(PrimitiveType::F64, RelOpType::Eq)),
            0x62 => inst!(RelOp::new(PrimitiveType::F64, RelOpType::Neq)),
            0x63 => inst!(RelOp::new(
                PrimitiveType::F64,
                RelOpType::Lt(Signedness::Signed)
            )),
            0x64 => inst!(RelOp::new(
                PrimitiveType::F64,
                RelOpType::Gt(Signedness::Signed)
            )),
            0x65 => inst!(RelOp::new(
                PrimitiveType::F64,
                RelOpType::Le(Signedness::Signed)
            )),
            0x66 => inst!(RelOp::new(
                PrimitiveType::F64,
                RelOpType::Ge(Signedness::Signed)
            )),
            0x67 => inst!(IUnOp::new(PrimitiveType::I32, IUnOpType::Clz)),
            0x68 => inst!(IUnOp::new(PrimitiveType::I32, IUnOpType::Ctz)),
            0x69 => inst!(IUnOp::new(PrimitiveType::I32, IUnOpType::Popcnt)),
            0x6A => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Add)),
            0x6B => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Sub)),
            0x6C => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Mul)),
            0x6D => inst!(IBinOp::new(
                PrimitiveType::I32,
                IBinOpType::Div(Signedness::Signed)
            )),
            0x6E => inst!(IBinOp::new(
                PrimitiveType::I32,
                IBinOpType::Div(Signedness::Unsigned)
            )),
            0x6F => inst!(IBinOp::new(
                PrimitiveType::I32,
                IBinOpType::Rem(Signedness::Signed)
            )),
            0x70 => inst!(IBinOp::new(
                PrimitiveType::I32,
                IBinOpType::Rem(Signedness::Unsigned)
            )),
            0x71 => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::And)),
            0x72 => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Or)),
            0x73 => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Xor)),
            0x74 => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Shl)),
            0x75 => inst!(IBinOp::new(
                PrimitiveType::I32,
                IBinOpType::Shr(Signedness::Signed)
            )),
            0x76 => inst!(IBinOp::new(
                PrimitiveType::I32,
                IBinOpType::Shr(Signedness::Unsigned)
            )),
            0x77 => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Rotl)),
            0x78 => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Rotr)),

            0x79 => inst!(IUnOp::new(PrimitiveType::I64, IUnOpType::Clz)),
            0x7A => inst!(IUnOp::new(PrimitiveType::I64, IUnOpType::Ctz)),
            0x7B => inst!(IUnOp::new(PrimitiveType::I64, IUnOpType::Popcnt)),
            0x7C => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Add)),
            0x7D => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Sub)),
            0x7E => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Mul)),
            0x7F => inst!(IBinOp::new(
                PrimitiveType::I64,
                IBinOpType::Div(Signedness::Signed)
            )),
            0x80 => inst!(IBinOp::new(
                PrimitiveType::I64,
                IBinOpType::Div(Signedness::Unsigned)
            )),
            0x81 => inst!(IBinOp::new(
                PrimitiveType::I64,
                IBinOpType::Rem(Signedness::Signed)
            )),
            0x82 => inst!(IBinOp::new(
                PrimitiveType::I64,
                IBinOpType::Rem(Signedness::Unsigned)
            )),
            0x83 => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::And)),
            0x84 => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Or)),
            0x85 => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Xor)),
            0x86 => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Shl)),
            0x87 => inst!(IBinOp::new(
                PrimitiveType::I64,
                IBinOpType::Shr(Signedness::Signed)
            )),
            0x88 => inst!(IBinOp::new(
                PrimitiveType::I64,
                IBinOpType::Shr(Signedness::Unsigned)
            )),
            0x89 => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Rotl)),
            0x8A => inst!(IBinOp::new(PrimitiveType::I64, IBinOpType::Rotr)),

            0x8B => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Abs)),
            0x8C => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Neg)),
            0x8D => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Ceil)),
            0x8E => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Floor)),
            0x8F => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Trunc)),
            0x90 => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Nearest)),
            0x91 => inst!(FUnOp::new(PrimitiveType::F32, FUnOpType::Sqrt)),
            0x92 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::Add)),
            0x93 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::Sub)),
            0x94 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::Mul)),
            0x95 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::Div)),
            0x96 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::Min)),
            0x97 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::Max)),
            0x98 => inst!(FBinOp::new(PrimitiveType::F32, FBinOpType::CopySign)),

            0x99 => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Abs)),
            0x9A => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Neg)),
            0x9B => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Ceil)),
            0x9C => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Floor)),
            0x9D => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Trunc)),
            0x9E => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Nearest)),
            0x9F => inst!(FUnOp::new(PrimitiveType::F64, FUnOpType::Sqrt)),
            0xA0 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::Add)),
            0xA1 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::Sub)),
            0xA2 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::Mul)),
            0xA3 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::Div)),
            0xA4 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::Min)),
            0xA5 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::Max)),
            0xA6 => inst!(FBinOp::new(PrimitiveType::F64, FBinOpType::CopySign)),

            0xA7 => inst!(CvtOp::new(
                PrimitiveType::I64,
                PrimitiveType::I32,
                CvtOpType::Wrap,
            )),
            0xA8 => inst!(CvtOp::new(
                PrimitiveType::F32,
                PrimitiveType::I32,
                CvtOpType::Trunc(Signedness::Signed)
            )),
            0xA9 => inst!(CvtOp::new(
                PrimitiveType::F32,
                PrimitiveType::I32,
                CvtOpType::Trunc(Signedness::Unsigned)
            )),
            0xAA => inst!(CvtOp::new(
                PrimitiveType::F64,
                PrimitiveType::I32,
                CvtOpType::Trunc(Signedness::Signed)
            )),
            0xAB => inst!(CvtOp::new(
                PrimitiveType::F64,
                PrimitiveType::I32,
                CvtOpType::Trunc(Signedness::Unsigned)
            )),
            0xAC => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::I64,
                CvtOpType::Extend(Signedness::Signed)
            )),
            0xAD => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::I64,
                CvtOpType::Extend(Signedness::Unsigned)
            )),
            0xAE => inst!(CvtOp::new(
                PrimitiveType::F32,
                PrimitiveType::I64,
                CvtOpType::Trunc(Signedness::Signed)
            )),
            0xAF => inst!(CvtOp::new(
                PrimitiveType::F32,
                PrimitiveType::I64,
                CvtOpType::Trunc(Signedness::Unsigned)
            )),
            0xB0 => inst!(CvtOp::new(
                PrimitiveType::F64,
                PrimitiveType::I64,
                CvtOpType::Trunc(Signedness::Signed)
            )),
            0xB1 => inst!(CvtOp::new(
                PrimitiveType::F64,
                PrimitiveType::I64,
                CvtOpType::Trunc(Signedness::Unsigned)
            )),
            0xB2 => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::F32,
                CvtOpType::Convert(Signedness::Signed)
            )),
            0xB3 => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::F32,
                CvtOpType::Convert(Signedness::Unsigned)
            )),
            0xB4 => inst!(CvtOp::new(
                PrimitiveType::I64,
                PrimitiveType::F32,
                CvtOpType::Convert(Signedness::Signed)
            )),
            0xB5 => inst!(CvtOp::new(
                PrimitiveType::I64,
                PrimitiveType::F32,
                CvtOpType::Convert(Signedness::Unsigned)
            )),

            0xB6 => inst!(CvtOp::new(
                PrimitiveType::F64,
                PrimitiveType::F32,
                CvtOpType::Demote
            )),
            0xB7 => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::F64,
                CvtOpType::Convert(Signedness::Signed)
            )),
            0xB8 => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::F64,
                CvtOpType::Convert(Signedness::Unsigned)
            )),
            0xB9 => inst!(CvtOp::new(
                PrimitiveType::I64,
                PrimitiveType::F64,
                CvtOpType::Convert(Signedness::Signed)
            )),
            0xBA => inst!(CvtOp::new(
                PrimitiveType::I64,
                PrimitiveType::F64,
                CvtOpType::Convert(Signedness::Unsigned)
            )),
            0xBB => inst!(CvtOp::new(
                PrimitiveType::F32,
                PrimitiveType::F64,
                CvtOpType::Promote
            )),

            0xBC => inst!(CvtOp::new(
                PrimitiveType::F32,
                PrimitiveType::I32,
                CvtOpType::Reinterpret
            )),
            0xBD => inst!(CvtOp::new(
                PrimitiveType::F64,
                PrimitiveType::I64,
                CvtOpType::Reinterpret
            )),
            0xBE => inst!(CvtOp::new(
                PrimitiveType::I32,
                PrimitiveType::F32,
                CvtOpType::Reinterpret
            )),
            0xBF => inst!(CvtOp::new(
                PrimitiveType::I64,
                PrimitiveType::F64,
                CvtOpType::Reinterpret
            )),

            0xFC => match self.read_byte()? {
                0x0 => inst!(CvtOp::new(
                    PrimitiveType::F32,
                    PrimitiveType::I32,
                    CvtOpType::TruncSat(Signedness::Signed)
                )),
                0x1 => inst!(CvtOp::new(
                    PrimitiveType::F32,
                    PrimitiveType::I32,
                    CvtOpType::TruncSat(Signedness::Unsigned)
                )),
                0x2 => inst!(CvtOp::new(
                    PrimitiveType::F64,
                    PrimitiveType::I32,
                    CvtOpType::TruncSat(Signedness::Signed)
                )),
                0x3 => inst!(CvtOp::new(
                    PrimitiveType::F64,
                    PrimitiveType::I32,
                    CvtOpType::TruncSat(Signedness::Unsigned)
                )),
                0x4 => inst!(CvtOp::new(
                    PrimitiveType::F32,
                    PrimitiveType::I64,
                    CvtOpType::TruncSat(Signedness::Signed)
                )),
                0x5 => inst!(CvtOp::new(
                    PrimitiveType::F32,
                    PrimitiveType::I64,
                    CvtOpType::TruncSat(Signedness::Unsigned)
                )),
                0x6 => inst!(CvtOp::new(
                    PrimitiveType::F64,
                    PrimitiveType::I64,
                    CvtOpType::TruncSat(Signedness::Signed)
                )),
                0x7 => inst!(CvtOp::new(
                    PrimitiveType::F64,
                    PrimitiveType::I64,
                    CvtOpType::TruncSat(Signedness::Unsigned)
                )),
                x => Err(Error::UnknownSecondaryOpcode(x as u64)),
            },

            x => Err(Error::UnknownOpcode(x as u64)),
        }
    }

    fn read_primitive_type(&mut self) -> Result<PrimitiveType, Error> {
        match self.read_byte()? {
            0x7F => Ok(PrimitiveType::I32),
            0x7E => Ok(PrimitiveType::I64),
            0x7D => Ok(PrimitiveType::F32),
            0x7C => Ok(PrimitiveType::F64),
            _ => Err(Error::UnexpectedData("Expected a number type")),
        }
    }

    fn read_function_type(&mut self) -> Result<FunctionType, Error> {
        if self.read_byte()? != 0x60 {
            return Err(Error::UnexpectedData("Expected function type"));
        }

        let mut param_types = Vec::new();
        let mut result_types = Vec::new();

        let param_len = self.read_int()?;
        for _ in 0..param_len {
            param_types.push(self.read_primitive_type()?);
        }

        let result_len = self.read_int()?;
        for _ in 0..result_len {
            result_types.push(self.read_primitive_type()?);
        }

        Ok(FunctionType::new(param_types, result_types))
    }

    fn read_name(&mut self) -> Result<String, Error> {
        let name_len = self.read_int()?;
        let name = match String::from_utf8(self.read_bytes(name_len)?) {
            Ok(s) => s,
            Err(_) => return Err(Error::UnexpectedData("Expected a valid UTF-8 string")),
        };
        Ok(name)
    }
}

struct ModuleSection {
    section_type: u8,
    content: ByteReader,
}

impl ModuleSection {
    fn new(section_type: u8, content: &[u8]) -> Self {
        /// TODO: make a macro for this
        #[cfg(debug)]
        {
            // for i in 0..content.len() {
            //     print!("{:02X} ", content[i]);
            // }
            // eprintln!();
        }
        ModuleSection {
            section_type,
            content: ByteReader::new(content),
        }
    }

    fn update_module(&mut self, module: &mut Module) -> Result<(), Error> {
        match self.section_type {
            1 => {
                // Type section
                let type_vec_len = self.content.read_int()?;
                for _i in 0..type_vec_len {
                    module.add_function_type(self.content.read_function_type()?);
                }
            }
            3 => {
                // Function section
                let type_index_vec_len = self.content.read_int()?;
                for _ in 0..type_index_vec_len {
                    let type_index = self.content.read_int()?;
                    let function_type = module.get_function_type(type_index);
                    module.add_function(Function::new(function_type))
                }
            }
            5 => {
                // Memory section
                let memory_vec_len = self.content.read_int()?;
                if memory_vec_len > 1 {
                    return Err(Error::Misc(
                        "Multiple memories are unimplemented per WASM spec restrictions.",
                    ));
                }
                for _ in 0..memory_vec_len {
                    // These are called limits in the spec, could abstract if it's ever used somewhere else
                    let (mem_min, mem_max) = match self.content.read_byte()? {
                        0x00 => (self.content.read_int::<u32>()?, u32::MAX),
                        0x01 => (
                            self.content.read_int::<u32>()?,
                            self.content.read_int::<u32>()?,
                        ),
                        _ => return Err(Error::UnexpectedData("Expected a valid limit type")),
                    };
                    let memory = Memory::new(mem_min, mem_max);
                    module.add_memory(memory);
                }
            }
            7 => {
                // Export section
                let export_vec_len = self.content.read_int()?;
                for _ in 0..export_vec_len {
                    let name = self.content.read_name()?;
                    match self.content.read_byte()? {
                        0x00 => {
                            module.add_export(name, Export::Function(self.content.read_int()?))?
                        }
                        0x01 => module.add_export(name, Export::Table(self.content.read_int()?))?,
                        0x02 => {
                            module.add_export(name, Export::Memory(self.content.read_int()?))?
                        }
                        0x03 => {
                            module.add_export(name, Export::Global(self.content.read_int()?))?
                        }
                        _ => {
                            return Err(Error::UnexpectedData(
                                "Expected a valid export descriptor type",
                            ))
                        }
                    }
                }
            }
            10 => {
                // Code section
                let functions_vec_len = self.content.read_int()?;
                for function_index in 0..functions_vec_len {
                    let _function_len_bytes = self.content.read_int::<usize>()?; /* Needs to be read, but we don't use it */
                    let function = module.get_mut_function(function_index);

                    // length of the implicit vector containing one tuple (count, type) for each type of local
                    let locals_types = self.content.read_int()?;

                    for _ in 0..locals_types {
                        let num_locals: usize = self.content.read_int()?; // number of locals of type `typ`
                        let typ = self.content.read_primitive_type()?;
                        let value = Value::from(&typ);
                        function.new_locals(num_locals, value);
                    }

                    loop {
                        match self.content.read_inst() {
                            Ok(Some(i)) => function.push_inst(i),
                            Ok(None) => {
                                break;
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
            x => {
                eprintln!("Unimplemented section: {:X}", x)
                // return Err(Error::UnknownSection);
            }
        }
        Ok(())
    }
}

pub fn parse_wasm(path: &str) -> Result<Module, Error> {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buf).unwrap();

    // Check that this matches the WASM magic number
    match buf[0..=3] {
        [b'\0', b'a', b's', b'm'] => (),
        _ => {
            return Err(Error::InvalidInput);
        }
    };

    // Check that this matches the only version of WASM we support
    match buf[4..=7] {
        [1, 0, 0, 0] => (),
        _ => {
            return Err(Error::BadVersion);
        }
    };

    let mut sections: Vec<ModuleSection> = Vec::new();
    let mut start = 8;
    while start < buf.len() {
        let section_type: u8 = buf[start];
        let (section_length, bytes_read) = parse_unsigned_leb128(&buf[start + 1..]);
        let section_end = 1 + bytes_read + section_length as usize;

        sections.push(ModuleSection::new(
            section_type,
            &buf[(start + 1 + bytes_read)..(start + section_end)],
        ));

        start += section_end;
    }

    let mut module = Module::new();

    for mut section in sections {
        section.update_module(&mut module)?;
    }

    Ok(module)
}
