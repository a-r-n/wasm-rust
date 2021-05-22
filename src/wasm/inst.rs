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
    fn execute(&self, stack: &mut Stack) -> ControlInfo {
        stack.push_value(self.value);
        ControlInfo::None
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
    fn execute(&self, stack: &mut Stack) -> ControlInfo {
        stack.push_value(self.value);
        ControlInfo::None
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
    fn execute(&self, stack: &mut Stack) -> ControlInfo {
        stack.push_value(self.value);
        ControlInfo::None
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
    fn execute(&self, stack: &mut Stack) -> ControlInfo {
        stack.push_value(self.value);
        ControlInfo::None
    }
}
