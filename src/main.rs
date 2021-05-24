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
        Err(Error::UnknownOpcode(x)) => {
            println!("unknown opcode: 0x{:X}", x)
        }
        Err(Error::EndOfData) => {
            println!("end of data")
        }
        Err(Error::UnexpectedData(s)) => {
            println!("{}", s);
        }
        Err(Error::IntSizeViolation) => {
            println!("int size violation")
        }
        Err(Error::StackViolation) => {
            println!("stack violation")
        }
        Err(_) => {
            println!("unknown error")
        }
    }
    std::process::exit(1);
}

fn main() {
    env_logger::init();
    println!("enter main");
    let mut module = handle_error(parse_wasm("test_inputs/addition.wasm"));
    let ret_val = handle_error(module.call("main"));
    println!("Final value: {}", ret_val);
    // return module.call_external("main");
}
