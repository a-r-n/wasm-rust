use wasm_interpreter::error::Error;
use wasm_interpreter::parser::*;

fn handle_error<T>(x: Result<T, Error>) -> T {
    match x {
        Ok(n) => {
            return n;
        }
        Err(Error::BadVersion) => {
            println!("bad version")
        }
        Err(Error::InvalidInput) => {
            println!("invalid input")
        }
        Err(Error::UnknownOpcode) => {
            println!("unknown opcode")
        }
        Err(Error::EndOfData) => {
            println!("end of data")
        }
        Err(Error::UnexpectedData(s)) => {
            println!("{}", s);
        }
        Err(_) => {
            println!("unknown error")
        }
    }
    std::process::exit(1);
}

fn main() {
    println!("enter main");
    let mut module = handle_error(parse_wasm("test_inputs/program.wasm"));
    let ret_val = handle_error(module.call("main"));
    println!("Final value: {}", ret_val);
    // return module.call_external("main");
}
