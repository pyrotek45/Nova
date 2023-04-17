use crate::state::LargeList;

#[derive(Debug, Clone)]
pub enum CallType {
    Loop,
    Block,
    Function,
    Closure,
    For(usize, LargeList, usize),
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub kind: CallType,
    pub target: usize,
    pub ret: usize,
}
