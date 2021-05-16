use wasm_interpreter::parser::*;

fn main() {
    println!("enter main");
    match parse_wasm("test_inputs/program.wasm") {
        Ok(_) => {
            println!("fine")
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
        Err(_) => {
            println!("unknown error")
        }
    }
    // return module.call_external("main");
}
