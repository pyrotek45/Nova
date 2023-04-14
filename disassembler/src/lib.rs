use common::code::Code;

pub fn new() -> Disassembler {
    Disassembler {
        depth: vec![],
        native_functions: common::table::new(),
        ip: 0,
    }
}

pub struct Disassembler {
    depth: Vec<usize>,
    pub native_functions: common::table::Table<String>,
    ip: usize
}

impl Disassembler {
    fn out(&self, output: &str) {
        for _ in 0..self.depth.len() {
            print!("  ")
        }
        println!("{}", output)
    }


    fn next(&mut self, input: &mut std::vec::IntoIter<u8>) -> Option<u8> {
        if let Some(index) = self.depth.last() {
            if self.ip == *index {
                self.depth.pop();
            }
        }
        //println!("ip: {}", self.ip);
        self.ip += 1;
        input.next()
    }

    pub fn dis(
        &mut self,
        mut input: std::vec::IntoIter<u8>,
    ) -> Result<(), common::error::NovaError> {

        while let Some(code) = self.next(&mut input) {
            match code {
                Code::RET => {
                    self.out("Return");
                }
                Code::INTEGER => {
                    let int = i64::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Push Integer {}", int))
                }
                Code::BYTE => {
                    let int = self.next(&mut input).unwrap() as i64;
                    self.out(&format!("Push Integer {}", int))
                }
                Code::FLOAT => {
                    let fl = f64::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Push Float {}", fl))
                }
                Code::ADD => self.out("Add"),
                Code::SUB => self.out("Sub"),
                Code::MUL => self.out("Mul"),
                Code::DIV => self.out("Div"),
                Code::STOREID => {
                    let index = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Store ID {}", index))
                }
                Code::ID => {
                    let index = u16::from_ne_bytes([self.next(&mut input).unwrap(), self.next(&mut input).unwrap()]);
                    self.out(&format!("ID {}", index))
                }
                Code::ASSIGN => self.out("Assign"),
                Code::ALLOCATEREG => {
                    let allocations = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Register allocation {}", allocations))
                }
                Code::BLOCK => {
                    let jump = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.depth.push(self.ip + jump);
                    self.out("Block:")
                }
                Code::CALL => self.out("Call"),
                Code::DIRECTCALL => {
                    let target = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Direct call {}", target))
                }
                Code::NEWLIST => {
                    let size = u64::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Create list: size of {}", size))
                }
                Code::TRUE => self.out("Push True"),
                Code::FALSE => self.out("Push False"),
                Code::STOREFASTID => {
                    let index = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("StoreFast ID {}", index))
                }
                Code::FUNCTION => {
                    let jump = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.depth.push(self.ip + jump);
                    self.out("Function:")
                }
                Code::GTR => self.out("Greater than"),
                Code::LSS => self.out("Less than"),
                Code::JUMPIFFALSE => {
                    let jump = u32::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Jump if false: {}", jump))
                }
                Code::REC => self.out("Recursive call"),
                Code::WHEN => self.out("When"),
                Code::IF => self.out("If"),
                Code::EQUALS => self.out("Equals"),
                Code::MODULO => self.out("Modulo"),
                Code::REFID => {
                    let index = u16::from_ne_bytes([self.next(&mut input).unwrap(), self.next(&mut input).unwrap()]);
                    self.out(&format!("Referance ID {}", index))
                }

                Code::CLOSURE => {
                    let jump = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.depth.push(self.ip + jump);
                    self.out("Closure:")
                }

                Code::CID => {
                    let index = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Closure ID {}", index))
                }

                Code::STRING => {
                    let mut string = vec![];
                    let size = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    for _ in 0..size {
                        string.push(self.next(&mut input).unwrap());
                    }
                    let string = match String::from_utf8(string) {
                        Ok(ok) => ok,
                        Err(_) => todo!(),
                    };
                    self.out(&format!("Push String: {}", string))
                }

                Code::FOR => self.out("For"),

                Code::RANGE => self.out("Range"),
                Code::NATIVE => {
                    let index = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    if let Some(function) = self.native_functions.retreive(index) {
                        self.out(&format!("Function: {}", function))
                    }
                }
                Code::ALLOCATEGLOBAL => {
                    let size = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Global allocation {}", size))
                }

                Code::GLOBALID => {
                    let index = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    self.out(&format!("Global ID {}", index))
                }

                Code::STOREGLOBAL => {
                    let index = usize::from_ne_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    self.out(&format!("Store Global ID {}", index))
                }

                Code::CHAR => {
                    let c = self.next(&mut input).unwrap();

                    self.out(&format!("Push Char {}", c as char))
                }

                Code::POP => self.out("Pop"),
                _ => {}
            }
        }

        Ok(())
    }
}
