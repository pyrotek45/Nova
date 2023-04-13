use common::error::NovaError;
use vm::state::{self, VmBig};

pub fn println(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(tos) = state.pop() {
        match tos {
            VmBig::Float(v) => {
                println!("{v}")
            }
            VmBig::Int(v) => {
                println!("{v}")
            }
            VmBig::Register(index) => {
                println!("register: {}", index);
            }
            VmBig::List(array) => {
                println!("{:?}", array)
            }
            VmBig::Bool(bool) => {
                println!("{bool}")
            }
            VmBig::String(string) => {
                println!("{string}")
            }
            VmBig::Char(c) => {
                println!("{}", c)
            }
            _ => {
                dbg!(tos);
                //todo!()
            }
        }
    }

    Ok(())
}

pub fn print(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(tos) = state.pop() {
        match tos {
            VmBig::Float(v) => {
                print!("{v}")
            }
            VmBig::Int(v) => {
                print!("{v}")
            }
            VmBig::Register(index) => {
                print!("register: {}", index);
            }
            VmBig::List(array) => {
                print!("{:?}", array)
            }
            VmBig::Bool(bool) => {
                print!("{bool}")
            }
            VmBig::String(string) => {
                print!("{string}")
            }
            VmBig::Char(c) => {
                print!("{}", c)
            }
            _ => {
                dbg!(tos);
                //todo!()
            }
        }
    }
    Ok(())
}

pub fn readln(state: &mut state::State) -> Result<(), NovaError> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    state.push(VmBig::String(line));
    Ok(())
}
