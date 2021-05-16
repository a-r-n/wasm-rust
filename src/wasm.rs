use std::collections::HashMap;
use std::collections::LinkedList;

/// The allowable types for any real value in wasm (u8 and others are packed)
#[derive(Copy, Clone)]
enum PrimitiveType {
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
struct Value {
    t: PrimitiveType,
    v: InternalValue,
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
    fn push_value(&mut self, v: Value) {
        self.values.push(v);
    }
}

pub trait Instruction {
    /// A wasm instruction may modify any state of the program
    fn execute(&self, stack: &mut Stack, module: &mut Module) -> ControlInfo;
}

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
    fn execute(&self, stack: &mut Stack, module: &mut Module) -> ControlInfo {
        stack.push_value(self.value);
        ControlInfo::None
    }
}

#[derive(Default)]
struct Table {
    functions: Vec<usize>,
}

#[derive(Default)]
pub struct Function {
    stack: Stack,
    locals: Vec<Value>,
    instructions: Vec<Box<dyn Instruction>>,
}

impl Function {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_inst(&mut self, i: Box<dyn Instruction>) {
        self.instructions.push(i);
    }
}

#[derive(Default)]
struct Memory {
    bytes: Vec<u8>,
}

#[derive(Default)]
pub struct Module {
    functions: Vec<Function>,
    table: Table,
    memory: Memory,
    globals: Vec<Value>,
    function_names: HashMap<String, usize>,
    global_names: HashMap<String, usize>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }
}
