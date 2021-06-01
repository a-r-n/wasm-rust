use std::env;
use wasm_interpreter::error::Error;
use wasm_interpreter::parser::*;

fn handle_error<T>(x: Result<T, Error>) -> T {
    match x {
        Ok(n) => {
            return n;
        }
        Err(Error::BadVersion) => {
            println!("Bad version")
        }
        Err(Error::InvalidInput) => {
            println!("Invalid input")
        }
        Err(Error::UnknownOpcode(x)) => {
            println!("Unknown opcode: 0x{:X}", x)
        }
        Err(Error::EndOfData) => {
            println!("End of data")
        }
        Err(Error::UnexpectedData(s)) => {
            println!("{}", s);
        }
        Err(Error::IntSizeViolation) => {
            println!("Int size violation")
        }
        Err(Error::StackViolation) => {
            println!("Stack violation")
        }
        Err(_) => {
            println!("Unknown error")
        }
    }
    std::process::exit(1);
}

fn main() {
    println!("Enter main");
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    let mut module = handle_error(parse_wasm(filename));

    let ret_val = handle_error(module.call("main"));

    println!("Final value: {}", ret_val);
    // return module.call_external("main");
}
