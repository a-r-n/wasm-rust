use std::env;
use wasm_interpreter::error::Error;
use wasm_interpreter::parser::*;
use wasm_interpreter::wasm::Value;

fn handle_error<T>(x: Result<T, Error>) -> T {
    match x {
        Ok(n) => {
            return n;
        }
        Err(Error::InvalidInput) => {
            println!("Invalid input")
        }
        Err(Error::BadVersion) => {
            println!("bad version")
        }
        Err(Error::UnknownOpcode(x)) => {
            println!("Unknown opcode: 0x{:X}", x)
        }
        Err(Error::UnknownSecondaryOpcode(x)) => {
            println!("unknown secondary opcode: 0x{:X}", x)
        }
        Err(Error::EndOfData) => {
            println!("End of data")
        }
        Err(Error::IntSizeViolation) => {
            println!("Int size violation")
        }
        Err(Error::FloatSizeViolation) => {
            println!("float size violation")
        }
        Err(Error::StackViolation) => {
            println!("Stack violation")
        }
        Err(Error::UnexpectedData(s)) => {
            println!("{}", s);
        }
        Err(Error::Misc(s)) => {
            println!("{}", s);
        }
        Err(_) => {
            println!("Unknown error")
        }
    }
    std::process::exit(1);
}

fn main() {
    use core::arch::x86_64::_rdtsc;
    
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let function_name = &args[2];

    let mut module = handle_error(parse_wasm(filename));
    let start_cycles = unsafe { _rdtsc() };
    let ret_val = handle_error(module.call(function_name, vec![Value::from(100000_i64)]));
    let end_cycles = unsafe { _rdtsc() };

    println!("Final value: {}", ret_val);
    println!("In {} cycles", end_cycles - start_cycles);
    // return module.call_external("main");
}
