use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use crate::error::Error;
use crate::wasm::inst::*;
use crate::wasm::*;

/// Returns (value, length read)
fn parse_leb128(bytes: &[u8]) -> (u64, usize) {
    let mut value = 0_u64;
    let mut offset = 0_usize;
    while bytes[offset] & (1_u8 << 7) != 0 {
        value += ((bytes[offset] & 0b01111111) as u64) << (7 * offset);
        offset += 1;
    }
    value += ((bytes[offset] & 0b01111111) as u64) << (7 * offset);
    offset += 1;

    (value, offset)
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

impl CheckedFromU64 for i64 {
    fn from(u: u64) -> Result<Self, Error> {
        match Self::try_from(u) {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::IntSizeViolation),
        }
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

impl CheckedFromU64 for i32 {
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
        let (value, read_bytes) = parse_leb128(&self.content[self.offset..]);
        self.offset += read_bytes;
        Ok(I::from(value)?)
    }

    fn read_inst(&mut self) -> Result<Option<Box<dyn Instruction>>, Error> {
        let opcode = self.read_int::<u64>()?;
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
            0x6A => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Add)),
            0x6B => inst!(IBinOp::new(PrimitiveType::I32, IBinOpType::Sub)),
            0x41 => inst!(ConstOp::new(Value::new(self.read_int::<i32>()?))),
            0x42 => inst!(ConstOp::new(Value::new(self.read_int::<i64>()?))),
            x => {
                return Err(Error::UnknownOpcode(x));
            }
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
            // println!();
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

                    let locals_vec_len = self.content.read_int()?;
                    for _ in 0..locals_vec_len {
                        let _t_vec: usize = self.content.read_int()?; // For vector types (I think) which we don't currently support -ARN
                        let t = self.content.read_primitive_type()?;
                        let value = Value::from(t);
                        function.new_local(value);
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
                println!("Unimplemented section: {:X}", x)
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
        let (section_length, bytes_read) = parse_leb128(&mut &buf[start + 1..]);
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
