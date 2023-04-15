//use std::time::Instant;
pub type CallBack = fn(state: &mut state::State) -> Result<(), NovaError>;

use common::{code::Code, error::NovaError};
//use modulo::Mod;
mod frame;
pub mod state;
use fxhash::FxHashMap;
use state::{VmBig, VmSmall};

use crate::frame::CallType;
#[allow(dead_code)]
pub struct Vm {
    program: Vec<u8>,
    pub native_functions: Vec<CallBack>,
    current_instruction: usize,
    callstack: Vec<frame::Frame>,
    state: state::State,
    dispatch: usize,
    analizer: FxHashMap<u8, std::time::Duration>,
}

pub fn new() -> Vm {
    Vm {
        program: vec![],
        current_instruction: 0,
        state: state::new(),
        callstack: vec![],
        dispatch: 0,
        analizer: FxHashMap::default(),
        native_functions: vec![],
    }
}

impl Vm {
    pub fn program(&mut self, program: Vec<u8>) {
        self.program = program
    }

    fn next(&mut self) -> u8 {
        //self.dispatch += 1;
        let result = &self.program[self.current_instruction];
        self.current_instruction += 1;
        *result
    }

    pub fn goto(&mut self, addr: usize) {
        self.current_instruction = addr;
    }

    pub fn run(&mut self) -> Result<(), common::error::NovaError> {
        //let mut calls: u128 = 0;
        loop {
            // calls += 1;
            // let start = Instant::now();

            //println!("current current_instruction: {}, Instruction: {} Calls: {}", self.current_instruction, self.program[self.current_instruction], calls);
            match self.next() {
                Code::RET => {
                    if let Some(ret) = self.callstack.pop() {
                        match ret.kind {
                            frame::CallType::Block => {
                                self.goto(ret.ret);
                            }
                            frame::CallType::Function => {
                                self.state.deallocate_registers();
                                self.goto(ret.ret)
                            }
                            frame::CallType::Closure => {
                                self.state.deallocate_registers();
                                self.state.deallocate_upvalue();
                                self.goto(ret.ret);
                            }
                            CallType::For(reg, array, currentindex) => {
                                if currentindex < array.len() {
                                    self.state
                                        .store_in_register(reg, array[currentindex].clone());
                                    self.callstack.push(frame::Frame {
                                        kind: frame::CallType::For(reg, array, currentindex + 1),
                                        target: ret.target,
                                        ret: ret.ret,
                                    });
                                    self.goto(ret.target);
                                } else {
                                    self.goto(ret.ret);
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
                Code::INTEGER => {
                    let int = i64::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.state.push_fast(VmSmall::Int(int));
                }
                Code::BYTE => {
                    let int = self.next() as i64;
                    self.state.push_fast(VmSmall::Int(int));
                }
                Code::FLOAT => {
                    let fl = f64::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    self.state.push_fast(VmSmall::Float(fl));
                }
                Code::ADD => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => VmSmall::Int(arg1 + arg2),
                            (VmSmall::Int(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg1 as f64 + arg2)
                            }
                            (VmSmall::Float(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Float(arg1 + arg2 as f64)
                            }
                            (VmSmall::Float(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg1 + arg2)
                            }
                            (a, b) => {
                                return Err(common::error::runetime_error(format!(
                                    "Cannot add {:?} + {:?}",
                                    a, b
                                )));
                            }
                        };
                        self.state.push_fast(result)
                    } else {
                        return Err(common::error::runetime_error(
                            "Not enough arguments for addition".to_string(),
                        ));
                    }
                }
                Code::SUB => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => VmSmall::Int(arg2 - arg1),
                            (VmSmall::Int(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg2 - arg1 as f64)
                            }
                            (VmSmall::Float(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Float(arg2 as f64 - arg1)
                            }
                            (VmSmall::Float(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg2 - arg1)
                            }
                            (a, b) => {
                                return Err(common::error::runetime_error(format!(
                                    "Cannot subtract {:?} - {:?}",
                                    a, b
                                )));
                            }
                        };
                        self.state.push_fast(result)
                    } else {
                        return Err(common::error::runetime_error(
                            "Not enough arguments for subtraction".to_string(),
                        ));
                    }
                }
                Code::MUL => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => VmSmall::Int(arg1 * arg2),
                            (VmSmall::Int(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg1 as f64 * arg2)
                            }
                            (VmSmall::Float(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Float(arg1 * arg2 as f64)
                            }
                            (VmSmall::Float(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg1 * arg2)
                            }
                            (a, b) => {
                                return Err(common::error::runetime_error(format!(
                                    "Cannot multiply {:?} * {:?}",
                                    a, b
                                )));
                            }
                        };
                        self.state.push_fast(result)
                    } else {
                        return Err(common::error::runetime_error(
                            "Not enough arguments for multiplication".to_string(),
                        ));
                    }
                }
                Code::DIV => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Float((arg2 / arg1) as f64)
                            }
                            (VmSmall::Int(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg2 / arg1 as f64)
                            }
                            (VmSmall::Float(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Float(arg2 as f64 / arg1)
                            }
                            (VmSmall::Float(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Float(arg2 / arg1)
                            }
                            (a, b) => {
                                return Err(common::error::runetime_error(format!(
                                    "Cannot division {:?} * {:?}",
                                    a, b
                                )));
                            }
                        };
                        self.state.push_fast(result)
                    } else {
                        return Err(common::error::runetime_error(
                            "Not enough arguments for divide".to_string(),
                        ));
                    }
                }
                Code::STOREID => {
                    let index = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    self.state.push_fast(VmSmall::Register(index));
                }
                Code::ID => {
                    let index = u16::from_ne_bytes([self.next(), self.next()]);
                    let item = self.state.get_from_register(index as usize);
                    //println!("getting {}", index);
                    self.state.push(item);
                }
                Code::ASSIGN => {
                    if let (Some(arg1), Some(arg2)) = (self.state.pop(), self.state.pop_fast()) {
                        match (arg1, arg2) {
                            (data, VmSmall::Register(index)) => {
                                self.state.store_in_register(index, data.clone());
                                //println!("storing {:?} in reg {}", data, index);
                            }
                            (data, VmSmall::Global(index)) => {
                                self.state.store_in_global(index, data.clone());
                                //println!("storing {:?} in reg {}", data, index);
                            }
                            (a, b) => {
                                return Err(common::error::runetime_error(format!(
                                    "Cannot Store {:?} in {:?}",
                                    a, b
                                )));
                            }
                        }
                    } else {
                        return Err(common::error::runetime_error(
                            "Not enough arguments for assignment".to_string(),
                        ));
                    }
                }
                Code::ALLOCATEREG => {
                    let allocations = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.state.allocate_registers(allocations);
                }
                Code::BLOCK => {
                    self.state.push(VmBig::Block(self.current_instruction + 8));
                    let jump = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.current_instruction += jump;
                }
                Code::CALL => {
                    if let Some(callee) = self.state.pop() {
                        match callee {
                            VmBig::Function(target) => {
                                self.callstack.push(frame::Frame {
                                    kind: frame::CallType::Function,
                                    target,
                                    ret: self.current_instruction,
                                });
                                self.goto(target);
                            }
                            VmBig::Block(target) => {
                                self.callstack.push(frame::Frame {
                                    kind: frame::CallType::Block,
                                    target,
                                    ret: self.current_instruction,
                                });
                                self.goto(target);
                            }
                            VmBig::Closure(target, upvalues) => {
                                self.callstack.push(frame::Frame {
                                    kind: frame::CallType::Closure,
                                    target,
                                    ret: self.current_instruction,
                                });
                                self.state.allocate_upvalue(upvalues);
                                self.goto(target);
                            }
                            a => {
                                dbg!(a);
                                todo!()
                            }
                        }
                    }
                }
                Code::DIRECTCALL => {
                    let target = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    let target = self.state.get_from_register(target);

                    match target {
                        VmBig::Function(target) => {
                            self.callstack.push(frame::Frame {
                                kind: frame::CallType::Function,
                                target,
                                ret: self.current_instruction,
                            });
                            self.goto(target);
                        }
                        VmBig::Block(target) => {
                            self.callstack.push(frame::Frame {
                                kind: frame::CallType::Block,
                                target,
                                ret: self.current_instruction,
                            });
                            self.goto(target);
                        }
                        VmBig::Closure(target, upvalues) => {
                            self.callstack.push(frame::Frame {
                                kind: frame::CallType::Closure,
                                target,
                                ret: self.current_instruction,
                            });
                            self.state.allocate_upvalue(upvalues);
                            self.goto(target);
                        }
                        VmBig::List(list) => {
                            if let Some(index) = self.state.pop_fast() {
                                match index {
                                    VmSmall::Int(index) => {
                                        self.state.push(list[index as usize].clone());
                                    }
                                    _ => {
                                        return Err(common::error::runetime_error(
                                            "Not enough arguments for list".to_string(),
                                        ));
                                    }
                                }
                            }
                        }
                        a => {
                            dbg!(a);
                            todo!()
                        }
                    }
                }
                Code::NEWLIST => {
                    let size = u64::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    let mut myarray = vec![];

                    for _ in 0..size {
                        if let Some(value) = self.state.pop() {
                            myarray.push(value)
                        } else {
                            todo!()
                        }
                    }
                    myarray.reverse();
                    self.state.push(VmBig::List(myarray));
                }
                Code::TRUE => {
                    self.state.push_fast(VmSmall::Bool(true));
                }
                Code::FALSE => {
                    self.state.push_fast(VmSmall::Bool(false));
                }
                Code::STOREFASTID => {
                    let index = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    if let Some(item) = self.state.pop() {
                        self.state.store_in_register(index, item.clone());
                    }
                }
                Code::FUNCTION => {
                    self.state
                        .push(VmBig::Function(self.current_instruction + 8));

                    let jump = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.current_instruction += jump;
                }
                Code::GTR => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => VmSmall::Bool(arg2 > arg1),
                            (VmSmall::Int(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Bool(arg2 > arg1 as f64)
                            }
                            (VmSmall::Float(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Bool(arg2 as f64 > arg1)
                            }
                            (VmSmall::Float(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Bool(arg2 > arg1)
                            }
                            _ => {
                                todo!()
                            }
                        };
                        self.state.push_fast(result);
                    }
                }
                Code::LSS => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => VmSmall::Bool(arg2 < arg1),
                            (VmSmall::Int(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Bool(arg2 < arg1 as f64)
                            }
                            (VmSmall::Float(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Bool((arg2 as f64) < arg1)
                            }
                            (VmSmall::Float(arg1), VmSmall::Float(arg2)) => {
                                VmSmall::Bool(arg2 < arg1)
                            }
                            _ => {
                                todo!()
                            }
                        };
                        self.state.push_fast(result);
                    }
                }
                Code::JUMPIFFALSE => {
                    let jump =
                        u32::from_ne_bytes([self.next(), self.next(), self.next(), self.next()]);

                    if let Some(VmSmall::Bool(false)) = self.state.pop_fast() {
                        self.current_instruction += jump as usize;
                    }
                }
                Code::REC => {
                    let frame = self.callstack[self.callstack.len() - 1].clone();
                    match frame.kind {
                        CallType::Function => {
                            self.callstack.push(frame::Frame {
                                kind: frame::CallType::Function,
                                target: frame.target,
                                ret: self.current_instruction,
                            });
                            self.goto(frame.target);
                        }
                        CallType::Block => {
                            self.callstack.push(frame::Frame {
                                kind: frame::CallType::Block,
                                target: frame.target,
                                ret: self.current_instruction,
                            });
                            self.goto(frame.target);
                        }
                        CallType::Closure => todo!(),
                        CallType::For(_, _, _) => todo!(),
                    };
                }
                Code::WHEN => {
                    if let Some(args) = self.state.pop_fast2() {
                        match args {
                            (VmSmall::Block(callee), VmSmall::Bool(test)) => {
                                if test {
                                    self.callstack.push(frame::Frame {
                                        kind: frame::CallType::Block,
                                        target: callee,
                                        ret: self.current_instruction,
                                    });
                                    self.goto(callee);
                                }
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }
                Code::IF => {
                    if let Some(args) = self.state.pop_fast3() {
                        match args {
                            (VmSmall::Block(elseb), VmSmall::Block(b), VmSmall::Bool(test)) => {
                                if test {
                                    self.callstack.push(frame::Frame {
                                        kind: frame::CallType::Block,
                                        target: b,
                                        ret: self.current_instruction,
                                    });
                                    self.goto(b);
                                } else {
                                    self.callstack.push(frame::Frame {
                                        kind: frame::CallType::Block,
                                        target: elseb,
                                        ret: self.current_instruction,
                                    });
                                    self.goto(elseb);
                                }
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }

                Code::EQUALS => {
                    if let Some(args) = self.state.pop2() {
                        let (one, two) = args;
                        if one == two {
                            self.state.push_fast(VmSmall::Bool(true))
                        } else {
                            self.state.push_fast(VmSmall::Bool(false))
                        };
                    }
                }

                Code::MODULO => {
                    if let Some(args) = self.state.pop_fast2() {
                        let result = match args {
                            (VmSmall::Int(arg1), VmSmall::Int(arg2)) => {
                                VmSmall::Int(modulo::Mod::modulo(arg2, arg1))
                            }
                            (a, b) => {
                                println!("{:?} {:?}", a, b);
                                todo!()
                            }
                        };
                        self.state.push_fast(result)
                    }
                }

                Code::REFID => {
                    let index = u16::from_ne_bytes([self.next(), self.next()]);
                    self.state.push_fast(VmSmall::Register(index as usize))
                }
                Code::CLOSURE => {
                    if let Some(VmBig::List(array)) = self.state.pop() {
                        self.state
                            .push(VmBig::Closure(self.current_instruction + 8, array));
                    }
                    let jump = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    self.current_instruction += jump;
                }
                Code::CID => {
                    let index = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    self.state.upvalue_to_stack(index);
                }

                Code::STRING => {
                    let mut string = vec![];
                    let size = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);
                    for _ in 0..size {
                        string.push(self.next());
                    }
                    let string = match String::from_utf8(string) {
                        Ok(ok) => ok,
                        Err(_) => todo!(),
                    };
                    self.state.push(VmBig::String(string));
                }

                Code::FOR => {
                    if let (Some(arg1), Some(arg2), Some(arg3)) = (
                        self.state.pop_fast(),
                        self.state.pop(),
                        self.state.pop_fast(),
                    ) {
                        match (arg3, arg2, arg1) {
                            (
                                VmSmall::Register(reg),
                                VmBig::List(array),
                                VmSmall::Block(target),
                            ) => {
                                if !array.is_empty() {
                                    self.state.store_in_register(reg, array[0].clone());
                                    self.callstack.push(frame::Frame {
                                        kind: frame::CallType::For(reg, array, 1),
                                        target,
                                        ret: self.current_instruction,
                                    });

                                    self.goto(target)
                                }
                            }
                            (a, b, c) => {
                                dbg!(a, b, c);
                                todo!()
                            }
                        }
                    } else {
                        todo!()
                    }
                }

                Code::RANGE => {
                    if let Some(args) = self.state.pop_fast2() {
                        match args {
                            (VmSmall::Int(to), VmSmall::Int(from)) => {
                                let mut array = vec![];
                                for i in from..=to {
                                    array.push(VmBig::Int(i));
                                }
                                self.state.push(VmBig::List(array));
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }

                Code::NATIVE => {
                    let index = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    match self.native_functions[index](&mut self.state) {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }
                Code::ALLOCATEGLOBAL => {
                    let size = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.state.allocate_globals(size);
                }

                Code::GLOBALID => {
                    let index = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.state.global_to_stack(index);
                }

                Code::STOREGLOBAL => {
                    let index = usize::from_ne_bytes([
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                        self.next(),
                    ]);

                    self.state.push_fast(VmSmall::Global(index))
                }

                Code::CHAR => {
                    let c = self.next();

                    self.state.push_fast(VmSmall::Char(c as char))
                }

                Code::POP => {
                    self.state.pop();
                }
                Code::NEG => {
                    if let Some(item) = self.state.pop_fast() {
                        let result = match item {
                            VmSmall::Int(int) => {VmSmall::Int(-int)}
                            VmSmall::Float(float) => {VmSmall::Float(-float)}
                            _ => {
                                todo!()
                            }
                        };
                        self.state.push_fast(result)
                    } else {
                        todo!()
                    }
                    
                }
                _ => {}
            }

            // println!("current offset {}",self.offset);

            // let duration = start.elapsed();
            // if let Some(data) = self.analizer.get_mut(&self.program[self.current_instruction]) {
            //     *data += duration;
            // } else {
            //     let new = std::time::Duration::new(0, 0);
            //     self.analizer.insert(self.program[self.current_instruction], new);
            // }
        }

        // if let Some(scope) = self.variable_map.last() {
        //     for (k, v) in scope {
        //         let string = std::str::from_utf8(v).unwrap();
        //         println!("{:?} @ {:?}", k, string)
        //     }
        // }

        // println!("");
        //println!("STACK: {:?}", self.stack)
        // println!("Dispatch: {}", self.dispatch);
        // println!("Total Calls: {}", calls);
        // for data in self.analizer.iter() {
        //     println!("Instruction: {} Time elapsed: {:?}", data.0, data.1)
        // }
        Ok(())
    }
}
