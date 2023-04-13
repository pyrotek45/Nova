use common::error::NovaError;
use rand::Rng;
use vm::state::{self, VmSmall};

pub fn random(state: &mut state::State) -> Result<(), NovaError> {
    if let Some((arg1, arg2)) = state.pop_fast2() {
        match (arg2, arg1) {
            (VmSmall::Int(start), VmSmall::Int(end)) => {
                let mut rng = rand::thread_rng();
                state.push_fast(VmSmall::Int(rng.gen_range(start..=end)))
            }
            _ => {
                return Err(common::error::runetime_error(
                    "Not enough arguments for random".to_string(),
                ));
            }
        }
    }
    Ok(())
}
