use common::error::NovaError;
use vm::state::{self, VmBig};

pub fn length(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(list) = state.pop() {
        match list {
            VmBig::List(list) => state.push(VmBig::Int(list.len() as i64)),
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for length".to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn push(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(args) = state.pop2() {
        match args {
            (item, VmBig::List(mut list)) => {
                list.push(item);
                state.push(VmBig::List(list));
            }
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for push".to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn pop(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(args) = state.pop() {
        match args {
            VmBig::List(mut list) => {
                list.pop();
                state.push(VmBig::List(list));
            }
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for pop".to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn last(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(args) = state.pop() {
        match args {
            VmBig::List(list) => {
                state.push(list.last().unwrap().clone());
            }
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for last".to_string(),
                ));
            }
        }
    }
    Ok(())
}
