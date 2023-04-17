pub type List = Vec<VmSmall>;
pub type LargeList = Vec<VmBig>;

pub fn new() -> State {
    State {
        stack: Vec::new(),
        heap: Vec::new(),
        registers: Vec::new(),
        window: Vec::new(),
        offset: 0,
        upvalues: vec![],
        globals: vec![],
        bindings: vec![],
    }
}

#[derive(Debug)]
pub struct State {
    stack: List,
    heap: LargeList,
    registers: LargeList,
    bindings: Vec<LargeList>,
    window: Vec<usize>,
    offset: usize,
    upvalues: Vec<LargeList>,
    globals: LargeList,
}

impl State {
    #[inline(always)]
    pub fn push_binding(&mut self, item: VmBig) {
        self.bindings.last_mut().unwrap().push(item)
    }

    #[inline(always)]
    pub fn binding_to_stack(&mut self, index: usize) {
        self.push(self.bindings.last().unwrap()[index].clone())
    }

    #[inline(always)]
    pub fn new_bindings(&mut self) {
        self.bindings.push(vec![]);
    }

    #[inline(always)]
    pub fn pop_bindings(&mut self) {
        self.bindings.pop();
    }

    #[inline(always)]
    pub fn store_in_global(&mut self, index: usize, item: VmBig) {
        self.globals[index] = item
    }

    #[inline(always)]
    pub fn global_to_stack(&mut self, index: usize) {
        self.push(self.globals[index].clone())
    }

    #[inline(always)]
    pub fn allocate_globals(&mut self, size: usize) {
        for _ in 0..size {
            self.globals.push(VmBig::None)
        }
    }

    #[inline(always)]
    pub fn allocate_upvalue(&mut self, values: LargeList) {
        self.upvalues.push(values)
    }

    #[inline(always)]
    pub fn _get_upvalue(&mut self, index: usize) -> VmBig {
        self.upvalues.last().unwrap()[index].clone()
    }

    #[inline(always)]
    pub fn upvalue_to_stack(&mut self, index: usize) {
        self.push(self.upvalues.last().unwrap()[index].clone())
    }

    #[inline(always)]
    pub fn deallocate_upvalue(&mut self) {
        self.upvalues.pop();
    }

    #[inline(always)]
    pub fn reg_count(&self) -> usize {
        self.registers.len()
    }

    #[inline(always)]
    pub fn push(&mut self, data: VmBig) {
        match &data {
            VmBig::Int(int) => self.stack.push(VmSmall::Int(*int)),
            VmBig::Float(float) => self.stack.push(VmSmall::Float(*float)),
            VmBig::Register(index) => self.stack.push(VmSmall::Register(*index)),
            VmBig::Block(index) => self.stack.push(VmSmall::Block(*index)),
            VmBig::Bool(bool) => self.stack.push(VmSmall::Bool(*bool)),
            VmBig::List(_) => {
                self.heap.push(data);
                self.stack.push(VmSmall::List)
            }
            VmBig::None => self.stack.push(VmSmall::None),
            VmBig::Function(index) => self.stack.push(VmSmall::Function(*index)),
            VmBig::Closure(_, _) => {
                self.heap.push(data);
                self.stack.push(VmSmall::Closure)
            }
            VmBig::String(_) => {
                self.heap.push(data);
                self.stack.push(VmSmall::String)
            }
            VmBig::Global(index) => self.stack.push(VmSmall::Global(*index)),
            VmBig::Char(c) => self.stack.push(VmSmall::Char(*c)),
        }
    }

    #[inline(always)]
    pub fn push_fast(&mut self, data: VmSmall) {
        self.stack.push(data)
    }

    #[inline(always)]
    pub fn pop2(&mut self) -> Option<(VmBig, VmBig)> {
        if let (Some(vm1), Some(vm2)) = (self.pop(), self.pop()) {
            Some((vm1, vm2))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<VmBig> {
        if let Some(data) = self.stack.pop() {
            match &data {
                VmSmall::Int(_) => Some(data.to_heap()),
                VmSmall::Float(_) => Some(data.to_heap()),
                VmSmall::Register(_) => Some(data.to_heap()),
                VmSmall::Block(_) => Some(data.to_heap()),
                VmSmall::Bool(_) => Some(data.to_heap()),
                VmSmall::List => self.heap.pop(),
                VmSmall::None => Some(data.to_heap()),
                VmSmall::Function(_) => Some(data.to_heap()),
                VmSmall::Closure => self.heap.pop(),
                VmSmall::String => self.heap.pop(),
                VmSmall::Global(_) => Some(data.to_heap()),
                VmSmall::Char(_) => Some(data.to_heap()),
            }
        } else {
            Some(VmBig::None)
        }
    }

    #[inline(always)]
    pub fn pop_fast3(&mut self) -> Option<(VmSmall, VmSmall, VmSmall)> {
        if let (Some(vm1), Some(vm2), Some(vm3)) =
            (self.stack.pop(), self.stack.pop(), self.stack.pop())
        {
            Some((vm1, vm2, vm3))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn pop_fast2(&mut self) -> Option<(VmSmall, VmSmall)> {
        if let (Some(vm1), Some(vm2)) = (self.stack.pop(), self.stack.pop()) {
            Some((vm1, vm2))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn pop_fast(&mut self) -> Option<VmSmall> {
        self.stack.pop()
    }

    #[inline(always)]
    pub fn allocate_registers(&mut self, size: usize) {
        self.offset = self.reg_count();
        self.window.push(self.offset);
        for _ in 0..size {
            self.registers.push(VmBig::None)
        }
    }

    #[inline(always)]
    pub fn deallocate_registers(&mut self) {
        if let Some(window) = self.window.pop() {
            let remove = self.reg_count() - window;
            for _ in 0..remove {
                self.registers.pop();
            }
        }
        self.offset = *self.window.last().unwrap();
    }

    #[inline(always)]
    pub fn get_from_register(&mut self, index: usize) -> VmBig {
        self.registers[self.offset + index].clone()
    }

    #[inline(always)]
    pub fn store_in_register(&mut self, index: usize, item: VmBig) {
        self.registers[self.offset + index] = item
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum VmSmall {
    Char(char),
    Int(i64),
    Float(f64),
    Register(usize),
    Global(usize),
    Block(usize),
    Function(usize),
    Bool(bool),
    String,
    Closure,
    List,
    None,
}

impl VmSmall {
    #[inline(always)]
    pub fn to_heap(self) -> VmBig {
        match self {
            VmSmall::Int(int) => VmBig::Int(int),
            VmSmall::Float(fl) => VmBig::Float(fl),
            VmSmall::Register(index) => VmBig::Register(index),
            VmSmall::Block(index) => VmBig::Block(index),
            VmSmall::Bool(bool) => VmBig::Bool(bool),
            VmSmall::Function(index) => VmBig::Function(index),
            VmSmall::None => VmBig::None,
            VmSmall::List => todo!(),
            VmSmall::Closure => todo!(),
            VmSmall::String => todo!(),
            VmSmall::Global(index) => VmBig::Global(index),
            VmSmall::Char(c) => VmBig::Char(c),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmBig {
    Char(char),
    Global(usize),
    Function(usize),
    Closure(usize, LargeList),
    Int(i64),
    Float(f64),
    Register(usize),
    Block(usize),
    Bool(bool),
    List(LargeList),
    String(String),
    None,
}

impl VmBig {
    #[inline(always)]
    pub fn int(&self) -> i64 {
        match self {
            VmBig::Int(int) => *int,
            _ => {
                todo!()
            }
        }
    }

    #[inline(always)]
    pub fn _to_data(&self) -> VmSmall {
        match self {
            VmBig::Int(int) => VmSmall::Int(*int),
            VmBig::Float(fl) => VmSmall::Float(*fl),
            VmBig::Register(index) => VmSmall::Register(*index),
            VmBig::Block(index) => VmSmall::Block(*index),
            VmBig::Bool(bool) => VmSmall::Bool(*bool),
            VmBig::List(_) => todo!(),
            VmBig::None => VmSmall::None,
            VmBig::Function(index) => VmSmall::Function(*index),
            VmBig::Closure(_, _) => todo!(),
            VmBig::String(_) => todo!(),
            VmBig::Global(index) => VmSmall::Global(*index),
            VmBig::Char(c) => VmSmall::Char(*c),
        }
    }

    #[inline(always)]
    pub fn _to_target(&self) -> usize {
        match self {
            VmBig::Int(_int) => todo!(),
            VmBig::Float(_fl) => todo!(),
            VmBig::Register(_index) => todo!(),
            VmBig::Block(index) => *index,
            VmBig::Bool(_) => todo!(),
            VmBig::List(_) => todo!(),
            VmBig::None => todo!(),
            VmBig::Function(index) => *index,
            VmBig::Closure(_, _) => todo!(),
            VmBig::String(_) => todo!(),
            VmBig::Global(index) => *index,
            VmBig::Char(_) => todo!(),
        }
    }
}
