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

    #[inline]
    pub fn as_i32_unchecked(&self) -> i32 {
        unsafe { self.v.i32 }
    }
    #[inline]
    pub fn as_i64_unchecked(&self) -> i64 {
        unsafe { self.v.i64 }
    }
    #[inline]
    pub fn as_f32_unchecked(&self) -> f32 {
        unsafe { self.v.f32 }
    }
    #[inline]
    pub fn as_f64_unchecked(&self) -> f64 {
        unsafe { self.v.f64 }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self {
            t: PrimitiveType::from(v),
            v: InternalValue::from(v),
        }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self {
            t: PrimitiveType::from(v),
            v: InternalValue::from(v),
        }
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self {
            t: PrimitiveType::from(v),
            v: InternalValue::from(v),
        }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self {
            t: PrimitiveType::from(v),
            v: InternalValue::from(v),
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

impl From<&PrimitiveType> for Value {
    fn from(x: &PrimitiveType) -> Value {
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
    UndefinedDivision,
}

pub enum ControlInfo {
    Branch(u32),
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
        log::debug!("Pushing {}", v);
        self.values.push(v);
    }

    pub fn pop_value(&mut self) -> Result<Value, Error> {
        log::debug!("Current stack len {}", self.values.len());

        if self.values.is_empty() {
            Err(Error::StackViolation)
        } else {
            unsafe { Ok(self.values.pop().unwrap_unchecked()) }
        }
    }

    /// Return the 0-indexed offset'th value from the stack (such that 0 is the most recently pushed value)
    pub fn fetch_value(&self, offset: usize) -> Result<&Value, Error> {
        let stack_size = self.values.len();
        let offset_to_fetch = stack_size - 1 - offset;
        match self.values.get(offset_to_fetch) {
            Some(n) => Ok(n),
            None => {
                log::debug!("Try to read {} stack size {}", offset_to_fetch, stack_size);
                Err(Error::StackViolation)
            }
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

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Current stack:\n[")?;
        for v in self.values.iter() {
            writeln!(f, "  {}", v)?;
        }
        write!(f, "]\n\n")?;
        Ok(())
    }
}

pub trait Instruction {
    /// A wasm instruction may modify any state of the program
    fn execute(
        &self,
        stack: &mut Stack,
        memory: &mut Memory,
        locals: &mut Vec<Value>,
        functions: &Vec<Function>,
    ) -> Result<ControlInfo, Error>;
}

pub mod inst;

#[derive(Default)]
struct Table {
    functions: Vec<usize>,
}

pub struct Function {
    r#type: FunctionType,
    local_types: Vec<PrimitiveType>,
    instructions: Vec<Box<dyn Instruction>>,
}

impl Function {
    pub fn new(r#type: FunctionType) -> Self {
        Self {
            r#type,
            local_types: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn push_inst(&mut self, i: Box<dyn Instruction>) {
        self.instructions.push(i);
    }

    pub fn num_params(&self) -> usize {
        self.r#type.num_params()
    }

    pub fn num_locals(&self) -> usize {
        self.local_types.len()
    }

    pub fn new_locals(&mut self, count: usize, t: PrimitiveType) {
        self.local_types.reserve(count);
        for _ in 0..count {
            self.local_types.push(t);
        }
    }

    fn do_return(mut stack: Stack) -> Result<Value, Error> {
        let ret = stack.pop_value();
        stack.assert_empty()?;
        ret
    }

    pub fn call(
        &self,
        functions: &Vec<Function>,
        memory: &mut Memory,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        let mut stack = Stack::new();
        let mut locals = Vec::with_capacity(self.num_params() + self.num_locals());
        for arg in args {
            locals.push(arg);
        }
        for t in &self.local_types {
            locals.push(Value::from(t));
        }
        for instruction in &self.instructions {
            match instruction.execute(&mut stack, memory, &mut locals, functions)? {
                ControlInfo::Return => {
                    return Self::do_return(stack);
                }
                ControlInfo::Trap(Trap::MemoryOutOfBounds) => panic!(), //TODO: don't panic, handle traps gracefully
                ControlInfo::Trap(Trap::UndefinedDivision) => panic!(),
                _ => (),
            };
        }
        Self::do_return(stack)
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
        let mut s = Self {
            bytes: Vec::with_capacity((PAGE_SIZE * min as u64) as usize),
            virtual_size_pages: min,
            upper_limit_pages: max,
        };
        s.write(PAGE_SIZE * min as u64, 32, 4); // It looks like
        s
    }

    pub fn write(&mut self, mut value: u64, bitwidth: u8, address: u64) -> Option<()> {
        log::debug!(
            "Write to address 0x{:x} with bitwidth {} and value 0x{:x}",
            address,
            bitwidth,
            value
        );
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
        if self.bytes.is_empty() || last_write_address > (self.bytes.len() - 1) as u64 {
            self.bytes.resize((last_write_address + 1) as usize, 0);
        }

        for i in (address..(address + bytes_to_write as u64)).rev() {
            self.bytes[i as usize] = (value & 0xFF) as u8;
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
        let bytes_to_read = (bitwidth / 8) as u64;

        let mut result = 0_u64;

        for i in address..(address + bytes_to_read) {
            result <<= 8;
            result += self.bytes[i as usize] as u64;
        }

        log::debug!(
            "Read from address 0x{:x} with bitwidth {} and value 0x{:x}",
            address,
            bitwidth,
            result
        );
        Some(Value::from_explicit_type(result_type, result))
    }
}

#[derive(Default, Clone)]
pub struct FunctionType {
    pub params: Vec<PrimitiveType>,
    pub returns: Vec<PrimitiveType>,
}

impl FunctionType {
    pub fn new(params: Vec<PrimitiveType>, returns: Vec<PrimitiveType>) -> Self {
        Self { params, returns }
    }

    pub fn num_params(&self) -> usize {
        self.params.len()
    }

    pub fn params_iter(&self) -> std::slice::Iter<PrimitiveType> {
        self.params.iter()
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

    pub fn call(&mut self, function_name: &str, args: Vec<Value>) -> Result<Value, Error> {
        let function_index = match self.exports.get(function_name) {
            Some(Export::Function(n)) => *n,
            _ => return Err(Error::Misc("On module call, given name is not a function")),
        };
        let function = match self.functions.get(function_index) {
            Some(n) => n,
            None => {
                return Err(Error::Misc(
                    "Function index given by export section is not valid",
                ))
            }
        };
        function.call(&self.functions, &mut self.memory, args)
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
