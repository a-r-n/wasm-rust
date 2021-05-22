use std::collections::HashMap;

use crate::error::Error;

/// The allowable types for any real value in wasm (u8 and others are packed)
#[derive(Copy, Clone)]
pub enum PrimitiveType {
    I32,
    I64,
    F32,
    F64,
}

/// Storage type for all wasm values
#[derive(Copy, Clone)]
union InternalValue {
    i32: i32,
    i64: i64,
    f32: f32,
    f64: f64,
}

/// Representation of all wasm values
#[derive(Copy, Clone)]
pub struct Value {
    t: PrimitiveType,
    v: InternalValue,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        unsafe {
            match self.t {
                PrimitiveType::I32 => {
                    write!(f, "({}:{})", "i32", self.v.i32)
                }
                PrimitiveType::I64 => {
                    write!(f, "({}:{})", "i64", self.v.i64)
                }
                PrimitiveType::F32 => {
                    write!(f, "({}:{})", "32", self.v.f32)
                }
                PrimitiveType::F64 => {
                    write!(f, "({}:{})", "F64", self.v.f64)
                }
            }
        }
    }
}

pub enum ControlInfo {
    Branch(usize),
    Return,
    None,
}

/// Representation of a wasm stack.
/// Formally, all functions get their own stack.
/// In properly formed wasm code, this is not required from an implementation perspective.\
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
    fn execute(&self, stack: &mut Stack) -> ControlInfo;
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

    pub fn call(&self) -> Result<Value, Error> {
        let mut stack = Stack::new();
        for instruction in &self.instructions {
            instruction.execute(&mut stack);
        }
        let ret = stack.pop_value();
        stack.assert_empty()?;
        ret
    }
}

#[derive(Default)]
struct Memory {
    bytes: Vec<u8>,
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
        let function = match self.functions.get(function_index) {
            Some(n) => n,
            None => {
                return Err(Error::Misc(
                    "Function index given by export section is not valid",
                ))
            }
        };
        function.call()
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
