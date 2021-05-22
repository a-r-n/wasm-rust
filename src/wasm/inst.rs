use super::*;

pub struct I32Const {
    value: Value,
}

impl I32Const {
    pub fn new(v: i32) -> Self {
        I32Const {
            value: Value {
                t: PrimitiveType::I32,
                v: InternalValue { i32: v },
            },
        }
    }
}

impl Instruction for I32Const {
    fn execute(&self, stack: &mut Stack, memory: &mut Memory) -> Result<ControlInfo, Error> {
        stack.push_value(self.value);
        Ok(ControlInfo::None)
    }
}

pub struct I64Const {
    value: Value,
}

impl I64Const {
    pub fn new(v: i64) -> Self {
        I64Const {
            value: Value {
                t: PrimitiveType::I64,
                v: InternalValue { i64: v },
            },
        }
    }
}

impl Instruction for I64Const {
    fn execute(&self, stack: &mut Stack, memory: &mut Memory) -> Result<ControlInfo, Error> {
        stack.push_value(self.value);
        Ok(ControlInfo::None)
    }
}

pub struct F32Const {
    value: Value,
}

impl F32Const {
    pub fn new(v: f32) -> Self {
        F32Const {
            value: Value {
                t: PrimitiveType::F32,
                v: InternalValue { f32: v },
            },
        }
    }
}

impl Instruction for F32Const {
    fn execute(&self, stack: &mut Stack, memory: &mut Memory) -> Result<ControlInfo, Error> {
        stack.push_value(self.value);
        Ok(ControlInfo::None)
    }
}

pub struct F64Const {
    value: Value,
}

impl F64Const {
    pub fn new(v: f64) -> Self {
        F64Const {
            value: Value {
                t: PrimitiveType::F64,
                v: InternalValue { f64: v },
            },
        }
    }
}

impl Instruction for F64Const {
    fn execute(&self, stack: &mut Stack, memory: &mut Memory) -> Result<ControlInfo, Error> {
        stack.push_value(self.value);
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
    fn execute(&self, stack: &mut Stack, memory: &mut Memory) -> Result<ControlInfo, Error> {
        let address = u32::try_from(stack.pop_value()?)? as u64 + self.offset as u64;
        match memory.read(self.result_type, self.load_bitwidth, address) {
            Some(s) => {
                stack.push_value(s);
                Ok(ControlInfo::None)
            }
            None => return Ok(ControlInfo::Trap(Trap::MemoryOutOfBounds)),
        }
    }
}
