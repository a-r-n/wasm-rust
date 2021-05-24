use std::collections::HashMap;
use std::convert::TryFrom;

use crate::error::Error;

/// The allowable types for any real value in wasm (u8 and others are packed)
#[derive(Copy, Clone, PartialEq)]
pub enum PrimitiveType {
    I32,
    I64,
    F32,
    F64,
}

impl From<i32> for PrimitiveType {
    fn from(_: i32) -> PrimitiveType {
        PrimitiveType::I32
    }
}

impl From<i64> for PrimitiveType {
    fn from(_: i64) -> PrimitiveType {
        PrimitiveType::I64
    }
}

impl From<f32> for PrimitiveType {
    fn from(_: f32) -> PrimitiveType {
        PrimitiveType::F32
    }
}

impl From<f64> for PrimitiveType {
    fn from(_: f64) -> PrimitiveType {
        PrimitiveType::F64
    }
}

/// Storage type for all wasm values
#[derive(Copy, Clone)]
pub union InternalValue {
    i32: i32,
    i64: i64,
    f32: f32,
    f64: f64,
}

impl From<i32> for InternalValue {
    fn from(x: i32) -> InternalValue {
        InternalValue { i32: x }
    }
}

impl From<i64> for InternalValue {
    fn from(x: i64) -> InternalValue {
        InternalValue { i64: x }
    }
}

impl From<f32> for InternalValue {
    fn from(x: f32) -> InternalValue {
        InternalValue { f32: x }
    }
}

impl From<f64> for InternalValue {
    fn from(x: f64) -> InternalValue {
        InternalValue { f64: x }
    }
}

/// Representation of all wasm values
#[derive(Copy, Clone)]
pub struct Value {
    t: PrimitiveType,
    v: InternalValue,
}

impl Value {
    pub fn new<T: Into<InternalValue> + Into<PrimitiveType> + Copy>(x: T) -> Self {
        Self {
            t: x.into(),
            v: x.into(),
        }
    }

    pub fn from_explicit_type(t: PrimitiveType, v: u64) -> Value {
        Self {
            t,
            v: InternalValue { i64: v as i64 },
        }
    }
}

impl TryFrom<Value> for u32 {
    type Error = Error;
    fn try_from(x: Value) -> Result<u32, Error> {
        match x.t {
            PrimitiveType::I32 => Ok(unsafe { x.v.i32 as u32 }),
            _ => Err(Error::Misc("Cannot extract as u32 from incorrect type")),
        }
    }
}

impl From<PrimitiveType> for Value {
    fn from(x: PrimitiveType) -> Value {
        match x {
            PrimitiveType::I32 => Value::new(0_i32),
            PrimitiveType::I64 => Value::new(0_i64),
            PrimitiveType::F32 => Value::new(0_f32),
            PrimitiveType::F64 => Value::new(0_f64),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        unsafe {
            match self.t {
                PrimitiveType::I32 => {
                    write!(f, "(i32:{})", self.v.i32)
                }
                PrimitiveType::I64 => {
                    write!(f, "(i64:{})", self.v.i64)
                }
                PrimitiveType::F32 => {
                    write!(f, "(f32:{})", self.v.f32)
                }
                PrimitiveType::F64 => {
                    write!(f, "(f64:{})", self.v.f64)
                }
            }
        }
    }
}

/// Represents expected runtime errors, i.e. problems with the program, not the interpreter
pub enum Trap {
    MemoryOutOfBounds,
}

pub enum ControlInfo {
    Branch(usize),
    Return,
    Trap(Trap),
    None,
}

/// Representation of a wasm stack.
/// All functions use a new stack when called.
#[derive(Default)]
pub struct Stack {
    values: Vec<Value>,
}

impl Stack {
    fn new() -> Self {
        Self::default()
    }

    fn push_value(&mut self, v: Value) {
        self.values.push(v);
    }

    pub fn pop_value(&mut self) -> Result<Value, Error> {
        match self.values.pop() {
            Some(n) => Ok(n),
            None => Err(Error::StackViolation),
        }
    }

    /// Return the 0-indexed offset'th value from the stack (such that 0 is the most recently pushed value)
    pub fn fetch_value(&self, offset: usize) -> Result<&Value, Error> {
        let stack_size = self.values.len();
        let offset_to_fetch = stack_size - 1 - offset;
        match self.values.get(offset_to_fetch) {
            Some(n) => Ok(n),
            None => Err(Error::StackViolation),
        }
    }

    pub fn assert_empty(&self) -> Result<(), Error> {
        if self.values.is_empty() {
            Ok(())
        } else {
            Err(Error::StackViolation)
        }
    }
}

pub trait Instruction {
    /// A wasm instruction may modify any state of the program
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        locals: &mut Vec<Value>,
    ) -> Result<ControlInfo, Error>;
}

pub mod inst;

#[derive(Default)]
struct Table {
    functions: Vec<usize>,
}

pub struct Function {
    r#type: FunctionType,
    locals: Vec<Value>,
    instructions: Vec<Box<dyn Instruction>>,
}

impl Function {
    pub fn new(r#type: FunctionType) -> Self {
        Self {
            r#type,
            locals: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn push_inst(&mut self, i: Box<dyn Instruction>) {
        self.instructions.push(i);
    }

    pub fn new_local(&mut self, v: Value) {
        self.locals.push(v);
    }

    pub fn call(&mut self, memory: &mut Memory) -> Result<Value, Error> {
        let mut stack = Stack::new();
        for instruction in &self.instructions {
            instruction.execute(&mut stack, memory, &mut self.locals)?;
        }
        let ret = stack.pop_value();
        stack.assert_empty()?;
        ret
    }
}

#[derive(Default)]
pub struct Memory {
    bytes: Vec<u8>,
    virtual_size_pages: u32,
    upper_limit_pages: u32,
}

const PAGE_SIZE: u64 = 0x10000;
impl Memory {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            bytes: Vec::new(),
            virtual_size_pages: min,
            upper_limit_pages: max,
        }
    }

    pub fn write(&mut self, mut value: u64, bitwidth: u8, address: u64) -> Option<()> {
        if bitwidth % 8 != 0 {
            // Probably don't even need to implement this
            panic!();
        }

        let bytes_to_write = bitwidth / 8;
        let last_write_address = address + bytes_to_write as u64;

        // Check for out of bounds access
        if last_write_address > PAGE_SIZE * self.virtual_size_pages as u64 {
            return None;
        }

        // Resize internal vector if needed
        if last_write_address > (self.bytes.len() - 1) as u64 {
            self.bytes.resize(last_write_address as usize, 0); // resize may not be correct -ARN
        }

        for i in (address + bytes_to_write as u64)..address {
            self.bytes[(address + i) as usize] = (value & 0xFF) as u8;
            value >>= 8;
        }

        Some(())
    }

    pub fn read(
        &mut self,
        result_type: PrimitiveType,
        bitwidth: u8,
        address: u64,
    ) -> Option<Value> {
        let final_byte_bits = bitwidth % 8;
        let bytes_to_read = (bitwidth / 8) + if final_byte_bits == 0 { 0 } else { 1 };
        let last_read_address = address + bytes_to_read as u64;
        // Check for out of bounds access
        if last_read_address > PAGE_SIZE * self.virtual_size_pages as u64 {
            return None;
        }
        // Resize internal vector if needed
        if last_read_address > (self.bytes.len() - 1) as u64 {
            self.bytes.resize(last_read_address as usize, 0); // resize may not be correct -ARN
        }
        let mut result = 0_u64;
        for i in address..(last_read_address - 1) {
            // Read entire bytes
            result += self.bytes[i as usize] as u64;
            result <<= 8;
        }
        // Final byte
        if final_byte_bits == 0 {
            // Actually read all 8 bytes
            result += self.bytes[last_read_address as usize] as u64;
        } else {
            let final_byte = self.bytes[last_read_address as usize];
            for i in 0..final_byte_bits {
                result |= final_byte as u64 & 1 << i;
            }
        }

        Some(Value::from_explicit_type(result_type, result))
    }
}

#[derive(Default, Clone)]
pub struct FunctionType {
    params: Vec<PrimitiveType>,
    returns: Vec<PrimitiveType>,
}

impl FunctionType {
    pub fn new(params: Vec<PrimitiveType>, returns: Vec<PrimitiveType>) -> Self {
        Self { params, returns }
    }
}

pub enum Export {
    Function(usize),
    Table(usize),
    Memory(usize),
    Global(usize),
}

#[derive(Default)]
pub struct Module {
    function_types: Vec<FunctionType>,
    functions: Vec<Function>,
    exports: HashMap<String, Export>,
    table: Table,
    memory: Memory,
    globals: Vec<Value>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn call(&mut self, function_name: &str) -> Result<Value, Error> {
        let function_index = match self.exports.get(function_name) {
            Some(Export::Function(n)) => *n,
            _ => return Err(Error::Misc("On module call, given name is not a function")),
        };
        let function = match self.functions.get_mut(function_index) {
            Some(n) => n,
            None => {
                return Err(Error::Misc(
                    "Function index given by export section is not valid",
                ))
            }
        };
        function.call(&mut self.memory)
    }

    pub fn add_function_type(&mut self, ft: FunctionType) {
        self.function_types.push(ft);
    }

    pub fn get_function_type(&self, i: usize) -> FunctionType {
        self.function_types[i].clone()
    }

    pub fn add_function(&mut self, f: Function) {
        self.functions.push(f);
    }

    pub fn add_memory(&mut self, m: Memory) {
        self.memory = m;
    }

    pub fn add_export(&mut self, name: String, export: Export) -> Result<(), Error> {
        if self.exports.contains_key(&name) {
            return Err(Error::UnexpectedData("Expected a unique export name"));
        }
        self.exports.insert(name, export);
        Ok(())
    }

    pub fn get_mut_function(&mut self, i: usize) -> &mut Function {
        &mut self.functions[i]
    }
}
