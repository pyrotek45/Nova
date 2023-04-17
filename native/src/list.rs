use common::error::NovaError;
use vm::state::{self, VmBig, VmSmall};

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

// list index item
pub fn insert(state: &mut state::State) -> Result<(), NovaError> {
    if let (Some(arg1),Some(arg2),Some(arg3))= (state.pop(),state.pop_fast(),state.pop()) {
        match (arg1,arg2,arg3) {
            (item,VmSmall::Int(index),VmBig::List(mut list)) => {
                list.insert(index as usize, item);
                state.push(VmBig::List(list.clone()));
            }
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for insert".to_string(),
                ));
            }
        }
    }
    Ok(())
}

// list index 
pub fn remove(state: &mut state::State) -> Result<(), NovaError> {
    if let (Some(arg1),Some(arg2))= (state.pop_fast(),state.pop()) {
        match (arg1,arg2) {
            (VmSmall::Int(index),VmBig::List(mut list)) => {
                list.remove(index as usize);
                state.push(VmBig::List(list.clone()));
            }
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for remove".to_string(),
                ));
            }
        }
    }
    Ok(())
}
