use common::code::Code;
use common::error::NovaError;
use common::tokens::Token;

pub fn new() -> Compiler {
    Compiler {
        native_functions: common::table::new(),
        variables: common::table::new(),
        output: Vec::new(),
        currentline: 0,
        filepath: String::new(),
        upvalues: common::table::new(),
        global: common::table::new(),
        entry: 0,
        bindings: common::table::new(),
    }
}

pub struct Compiler {
    pub bindings: common::table::Table<String>,
    pub global: common::table::Table<String>,
    pub variables: common::table::Table<String>,
    pub upvalues: common::table::Table<String>,
    pub native_functions: common::table::Table<String>,
    output: Vec<u8>,
    pub currentline: usize,
    filepath: String,
    pub entry: usize,
}

impl Compiler {
    pub fn clear(&mut self) {
        self.output.clear()
    }

    pub fn get_entry(&self) -> usize {
        self.entry
    }

    #[inline(always)]
    pub fn compile(&mut self, input: Vec<Token>, filepath: String) -> Result<Vec<u8>, NovaError> {
        self.filepath = filepath;
        let chunks = match self.compile_chunk(input) {
            Ok(chunk) => chunk,
            Err(error) => return Err(error),
        };

        let packaged = self.load_package(chunks);
        let packaged = self.load_globals(packaged);

        self.output.extend_from_slice(&packaged);
        Ok(self.output.to_owned())
    }
    #[inline(always)]
    fn compile_chunk(&mut self, input: Vec<Token>) -> Result<Vec<u8>, NovaError> {
        let mut output = vec![];
        for op in input.iter() {
            match op {
                Token::LinePosition(line) => self.currentline = *line,
                Token::Reg(id) => match id.as_str() {
                    "true" => output.push(Code::TRUE),
                    "false" => output.push(Code::FALSE),
                    _ => {
                        if let Some(index) = self.variables.get_index(id.to_string()) {
                            output.push(Code::ID);
                            let bytes = (index as u16).to_ne_bytes();
                            output.extend_from_slice(&bytes);
                        } else if let Some(index) = self.upvalues.get_index(id.to_string()) {
                            output.push(Code::CID);
                            let bytes = index.to_ne_bytes();
                            output.extend_from_slice(&bytes);
                        } else {
                            if let Some(index) = self.global.get_index(id.to_string()) {
                                output.push(Code::GLOBALID);
                                let bytes = index.to_ne_bytes();
                                output.extend_from_slice(&bytes);
                                continue;
                            }

                            return Err(common::error::compiler_error(
                                format!("[ID] {} is not initialized", &id),
                                self.currentline,
                                self.filepath.clone(),
                            ));
                        }
                    }
                },
                Token::RegStore(id) => {
                    if let Some(index) = self.variables.get_index(id.to_string()) {
                        output.push(Code::STOREID);
                        let bytes = index.to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    } else {
                        self.variables.insert(id.to_string());
                        let index = self.variables.len() - 1;
                        output.push(Code::STOREID);
                        let bytes = index.to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    }
                }
                Token::RegStoreFast(id) => {
                    if self.variables.get_index(id.to_string()).is_some() {
                        todo!()
                    } else {
                        self.variables.insert(id.to_string());
                        let index = self.variables.len() - 1;
                        output.push(Code::STOREFASTID);
                        let bytes = index.to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    }
                }
                Token::StoreFastBindId(id) => {
                    if self.bindings.get_index(id.to_string()).is_some() {
                        todo!()
                    } else {
                        self.bindings.insert(id.to_string());
                        output.push(Code::STOREBIND);
                    }
                },
                Token::BindingRef(id) => {
                    if let Some(index) = self.bindings.get_index(id.to_string()) {
                        output.push(Code::GETBIND);
                        let bytes = index.to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    } else {
                        return Err(common::error::compiler_error(
                            format!("{} is not initialized", &id),
                            self.currentline,
                            self.filepath.clone(),
                        ));
                    }
                },
                Token::Integer(value) => {
                    if value < &(u8::MAX as i64) && value > &0 {
                        output.push(Code::BYTE);
                        let int = *value as u8;
                        output.push(int);
                    } else {
                        output.push(Code::INTEGER);
                        let int = value.to_ne_bytes();
                        output.extend_from_slice(&int);
                    }
                }
                Token::Float(value) => {
                    output.push(Code::FLOAT);
                    let float = value.to_ne_bytes();
                    output.extend_from_slice(&float);
                }
                Token::String(string) => {
                    output.push(Code::STRING);
                    let size = string.len().to_ne_bytes();
                    output.extend_from_slice(&size);
                    let cast = string.as_bytes();
                    output.extend_from_slice(cast);
                }
                Token::Char(c) => {
                    output.push(Code::CHAR);
                    output.push(*c as u8);
                }
                Token::Symbol(_) => todo!(),
                Token::Bool(_) => todo!(),
                Token::BlockLiteral(block) => {
                    let mut bytes = match self.compile_chunk(block.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };
                    output.push(Code::BLOCK);
                    let cast = bytes.len();
                    let int = cast.to_ne_bytes();
                    output.extend_from_slice(&int);
                    output.append(&mut bytes)
                }
                Token::Function(input, logic) => {
                    let mut function_c = new();
                    function_c.currentline = self.currentline;
                    function_c.native_functions = self.native_functions.clone();
                    function_c.global = self.global.clone();
                    function_c.filepath = self.filepath.clone();
                    let mut arguments = vec![];
                    for args in input.iter().rev() {
                        match args {
                            Token::Reg(id) => arguments.push(Token::RegStoreFast(id.to_string())),
                            _ => {
                                todo!()
                            }
                        }
                    }
                    arguments.extend_from_slice(logic);
                    let bytes = match function_c.compile_chunk(arguments.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };

                    let mut bytes = function_c.load_package(bytes);

                    output.push(Code::FUNCTION);
                    let cast = bytes.len();
                    let int = cast.to_ne_bytes();
                    output.extend_from_slice(&int);
                    output.append(&mut bytes);
                    self.global = function_c.global.clone();
                }
                Token::Call(name) => match name.as_str() {
                    "loop" => output.push(Code::LOOP),
                    "range" => output.push(Code::RANGE),
                    "for" => {
                        output.push(Code::FOR);
                        output.push(Code::BOUNCE)
                    }
                    "when" => output.push(Code::WHEN),
                    "if" => output.push(Code::IF),
                    "return" => output.push(Code::RET),
                    "rec" => output.push(Code::REC),
                    _ => {
                        if let Some(index) = self.native_functions.get_index(name.to_string()) {
                            output.push(Code::NATIVE);
                            let bytes = index.to_ne_bytes();
                            output.extend_from_slice(&bytes);
                            continue;
                        }

                        if let Some(index) = self.variables.get_index(name.to_string()) {
                            output.push(Code::DIRECTCALL);
                            let bytes = index.to_ne_bytes();
                            output.extend_from_slice(&bytes);
                        } else if let Some(index) = self.upvalues.get_index(name.to_string()) {
                            output.push(Code::CID);
                            let bytes = index.to_ne_bytes();
                            output.extend_from_slice(&bytes);
                            output.push(Code::CALL)
                        } else {
                            if let Some(index) = self.global.get_index(name.to_string()) {
                                output.push(Code::GLOBALID);
                                let bytes = index.to_ne_bytes();
                                output.extend_from_slice(&bytes);
                                output.push(Code::CALL);
                                continue;
                            }

                            return Err(common::error::compiler_error(
                                format!("[CALL] {} is not initialized", &name),
                                self.currentline,
                                self.filepath.clone(),
                            ));
                        }
                    }
                },
                Token::Op(operation) => match operation {
                    common::tokens::Operator::Assign => output.push(Code::ASSIGN),
                    common::tokens::Operator::BindVar => todo!(),
                    common::tokens::Operator::New => todo!(),
                    common::tokens::Operator::AccessCall => todo!(),
                    common::tokens::Operator::ModuleCall => todo!(),
                    common::tokens::Operator::UserFunctionChain => todo!(),
                    common::tokens::Operator::StoreTemp => todo!(),
                    common::tokens::Operator::And => todo!(),
                    common::tokens::Operator::Or => todo!(),
                    common::tokens::Operator::Not => todo!(),
                    common::tokens::Operator::Equals => output.push(Code::EQUALS),
                    common::tokens::Operator::Gtr => output.push(Code::GTR),
                    common::tokens::Operator::Lss => output.push(Code::LSS),
                    common::tokens::Operator::Invert => todo!(),
                    common::tokens::Operator::Mod => output.push(Code::MODULO),
                    common::tokens::Operator::Add => output.push(Code::ADD),
                    common::tokens::Operator::Sub => output.push(Code::SUB),
                    common::tokens::Operator::Mul => output.push(Code::MUL),
                    common::tokens::Operator::Div => output.push(Code::DIV),
                    common::tokens::Operator::PopBindings => todo!(),
                    common::tokens::Operator::Neg => output.push(Code::NEG),
                    common::tokens::Operator::Break => output.push(Code::BREAK),
                    common::tokens::Operator::Continue => todo!(),
                    common::tokens::Operator::ResolveBind => todo!(),
                },
                Token::List(list) => {
                    let mut bytes = match self.compile_chunk(list.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };
                    // removeing last ret statement
                    bytes.pop();
                    output.extend_from_slice(&bytes);
                    output.push(Code::NEWLIST);
                    let cast = (list.len()) as u64;
                    let int = cast.to_ne_bytes();
                    output.extend_from_slice(&int);
                }
                Token::Arguments(_) => todo!(),
                Token::ConditionalBlock(block) => {
                    let mut bytes = match self.compile_chunk(block.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };
                    bytes.pop();

                    output.push(Code::JUMPIFFALSE);
                    let cast = bytes.len();
                    let int = (cast as u32).to_ne_bytes();
                    output.extend_from_slice(&int);
                    output.append(&mut bytes)
                }
                Token::RegRef(id) => {
                    if let Some(index) = self.variables.get_index(id.to_string()) {
                        output.push(Code::REFID);
                        let bytes = (index as u16).to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    } else {
                        return Err(common::error::compiler_error(
                            format!("{} is not initialized", &id),
                            self.currentline,
                            self.filepath.clone(),
                        ));
                    }
                }
                Token::Closure(closed, input, logic) => {
                    // collect upvalues into list
                    let mut upvalues = common::table::new();

                    // spit out values as in a new array
                    for v in closed.iter() {
                        match v {
                            Token::Reg(id) => {
                                if self.variables.has(id) {
                                    upvalues.insert(id.clone())
                                } else {
                                    return Err(common::error::compiler_error(
                                        format!("[CLOSURE] {} is not initialized", &id),
                                        self.currentline,
                                        self.filepath.clone(),
                                    ));
                                }
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }

                    // todo setup input list
                    let mut bytes = match self.compile_chunk(closed.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };
                    // removeing last ret statement
                    bytes.pop();
                    output.extend_from_slice(&bytes);
                    // newarray
                    output.push(Code::NEWLIST);
                    let cast = (closed.len()) as u64;
                    let int = cast.to_ne_bytes();
                    output.extend_from_slice(&int);

                    let mut function_c = new();
                    function_c.upvalues = upvalues;
                    function_c.currentline = self.currentline;
                    function_c.native_functions = self.native_functions.clone();
                    function_c.global = self.global.clone();
                    function_c.filepath = self.filepath.clone();
                    let mut arguments = vec![];
                    for args in input.iter().rev() {
                        match args {
                            Token::Reg(id) => arguments.push(Token::RegStoreFast(id.to_string())),
                            _ => {
                                todo!()
                            }
                        }
                    }
                    arguments.extend_from_slice(logic);
                    let bytes = match function_c.compile_chunk(arguments.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };

                    let mut bytes = function_c.load_package(bytes);
                    output.push(Code::CLOSURE);
                    let cast = bytes.len();
                    let int = cast.to_ne_bytes();
                    output.extend_from_slice(&int);
                    output.append(&mut bytes);
                    self.global = function_c.global.clone();
                }
                Token::Doblock(_) => {
                    todo!();
                    // let mut bytes = match self.compile_chunk(block.to_vec()) {
                    //     Ok(chunk) => chunk,
                    //     Err(error) => return Err(error),
                    // };

                    // output.push(Code::FORINT);
                    // // let cast = bytes.len();
                    // // let int = cast.to_ne_bytes();
                    // // output.extend_from_slice(&int);
                    // output.append(&mut bytes)
                }
                Token::CurrentFile(currentfile) => self.filepath = currentfile.clone(),
                Token::GlobalReg(id) => {
                    if let Some(index) = self.global.get_index(id.to_string()) {
                        output.push(Code::STOREGLOBAL);
                        let bytes = index.to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    } else {
                        self.global.insert(id.to_string());
                        let index = self.global.len() - 1;
                        output.push(Code::STOREGLOBAL);
                        let bytes = index.to_ne_bytes();
                        output.extend_from_slice(&bytes);
                    }
                }
                Token::Entry => {
                    self.entry = output.len();
                }
                Token::Pop => output.push(Code::POP),
                Token::Bindings(_) => todo!(),
                Token::LetBinding(input, logic) => {
                    let mut function_c = new();
                    function_c.upvalues = self.upvalues.clone();
                    function_c.currentline = self.currentline;
                    function_c.native_functions = self.native_functions.clone();
                    function_c.global = self.global.clone();
                    function_c.filepath = self.filepath.clone();
                    function_c.variables = self.variables.clone();

                    let mut arguments = vec![];
                    for args in input.iter().rev() {
                        match args {
                            Token::Reg(id) => {
                                arguments.push(Token::StoreFastBindId(id.to_string()))
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }

                    arguments.extend_from_slice(logic);
                    let mut bind = match function_c.compile_chunk(arguments.to_vec()) {
                        Ok(chunk) => chunk,
                        Err(error) => return Err(error),
                    };

                    bind.pop();
                    bind.insert(0, Code::NEWBINDING);
                    bind.push(Code::POPBINDING);
                    output.extend_from_slice(&bind);
                    self.global = function_c.global.clone();
                }
            }
        }
        // end return
        output.push(Code::RET);
        Ok(output)
    }
    #[inline(always)]
    fn load_package(&mut self, bytes: Vec<u8>) -> Vec<u8> {
        let mut package = vec![];
        package.push(Code::ALLOCATEREG);
        let allocations = self.variables.len().to_ne_bytes();
        package.extend_from_slice(&allocations);
        package.extend_from_slice(&bytes);
        package
    }

    #[inline(always)]
    fn load_globals(&mut self, bytes: Vec<u8>) -> Vec<u8> {
        let mut package = vec![];
        package.push(Code::ALLOCATEGLOBAL);
        let allocations = self.global.len().to_ne_bytes();
        package.extend_from_slice(&allocations);
        package.extend_from_slice(&bytes);
        package
    }
}
