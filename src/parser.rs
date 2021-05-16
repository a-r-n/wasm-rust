use std::any::TypeId;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use crate::wasm::*;

/// Returns (value, length read)
fn parse_leb128(bytes: &[u8]) -> (u64, usize) {
    let mut value = 0_u64;
    let mut offset = 0_usize;
    while bytes[offset] & (1_u8 << 7) != 0 {
        value += ((bytes[offset] & 0b01111111) as u64) << 7 * offset;
        offset += 1;
    }
    value += ((bytes[offset] & 0b01111111) as u64) << 7 * offset;
    offset += 1;

    (value, offset)
}

pub enum Error {
    InvalidInput,
    BadVersion,
    UnknownSection,
    UnknownOpcode,
}

struct ByteReader {
    content: Vec<u8>,
    offset: usize,
}

impl ByteReader {
    fn new(content: &[u8]) -> Self {
        Self {
            content: Vec::from(content),
            offset: 0,
        }
    }

    fn read_byte(&mut self) -> u8 {
        todo!()
    }

    fn read_int(&mut self) -> u64 {
        let (value, read_bytes) = parse_leb128(&self.content[self.offset..]);
        self.offset += read_bytes;
        value
    }

    fn read_inst(&mut self) -> Result<Option<Box<dyn Instruction>>, Error> {
        let opcode = self.read_int();
        match opcode {
            0x0B => Ok(None),
            0x41 => Ok(Some(Box::new(I32Const::new(self.read_int() as i32)))),
            _ => {
                return Err(Error::UnknownOpcode);
            }
        }
    }

    fn read_utf8(&mut self) -> String {
        todo!()
    }
}

struct ModuleSection {
    section_type: u8,
    content: ByteReader,
}

impl ModuleSection {
    fn new(section_type: u8, content: &[u8]) -> Self {
        /// TODO: make a macro for this
        for i in 0..content.len() {
            print!("{:02X} ", content[i]);
        }
        println!();
        ModuleSection {
            section_type,
            content: ByteReader::new(content),
        }
    }

    fn update_module(&mut self, module: &mut Module) -> Result<(), Error> {
        match self.section_type {
            0x0A => {
                // Handle code section
                let functions_vec_len = self.content.read_int();
                for i in 0..functions_vec_len {
                    let function_len_bytes = self.content.read_int();
                    let locals_vec_len = self.content.read_int();
                    for local_index in 0..locals_vec_len {
                        todo!();
                    }

                    let mut function = Function::new();

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
            _ => {
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
