use common::{error::NovaError, tokens::Token};

pub struct Core {
    lexer: lexer::Lexer,
    parser: parser::Parser,
    compiler: compiler::Compiler,
    vm: vm::Vm,
    filepath: String,
    program: Vec<Token>,
}

pub fn new() -> Core {
    Core {
        lexer: lexer::new(),
        parser: parser::new(),
        compiler: compiler::new(),
        vm: vm::new(),
        filepath: String::new(),
        program: vec![],
    }
}

impl Core {
    pub fn dis(&mut self) {
        let program = match self.lexer.parse() {
            Ok(lexed) => lexed,
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };
        let program = match self.parser.parse(program.to_owned()) {
            Ok(parsed) => parsed,
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };

        let program = match self.compiler.compile(program, self.filepath.clone()) {
            Ok(parsed) => parsed,
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };

        let mut dis = disassembler::new();
        dis.native_functions = self.compiler.native_functions.clone();
        let _ = dis.dis(program.into_iter());
    }
    pub fn add_function(&mut self, name: &str, function: vm::CallBack) {
        self.compiler.native_functions.insert(name.to_string());
        self.vm.native_functions.push(function);
    }

    pub fn open_file(&mut self, filepath: &str) -> Result<(), String> {
        self.filepath = filepath.to_string();
        self.lexer.open_file(filepath)
    }

    pub fn eval(&mut self, input: &str, repl: bool) -> Result<(), NovaError> {
        self.lexer = lexer::new();
        self.parser = parser::new();
        let mut vm = vm::new();

        vm.native_functions = self.vm.native_functions.clone();

        self.lexer.insert_string(input);

        let program = match self.lexer.parse() {
            Ok(lexed) => lexed,
            Err(error) => {
                return Err(error);
            }
        };

        let program = match self.parser.parse(program.to_owned()) {
            Ok(parsed) => parsed,
            Err(error) => {
                return Err(error);
            }
        };

        let last = self.program.clone();

        if repl {
            let mut replprogram = vec![];
            for t in self.program.clone() {
                match t {
                    Token::Call(ref call) => match call.as_str() {
                        "println" => replprogram.push(Token::Pop),
                        "print" => replprogram.push(Token::Pop),
                        _ => replprogram.push(t),
                    },
                    _ => replprogram.push(t),
                }
            }
            self.program = replprogram;
        }

        self.program.extend_from_slice(&program);

        self.compiler.clear();
        let program = match self
            .compiler
            .compile(self.program.clone(), self.filepath.clone())
        {
            Ok(parsed) => parsed,
            Err(error) => {
                self.program = last;
                return Err(error);
            }
        };

        vm.program(program);

        match vm.run() {
            Ok(_) => {}
            Err(error) => {
                self.program = last;
                return Err(error);
            }
        };
        Ok(())
    }

    pub fn run(&mut self) {
        //let start = Instant::now();
        let program = match self.lexer.parse() {
            Ok(lexed) => lexed,
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };

        // for toks in program {
        //     println!("{:?}", toks)
        // }

        // let duration = start.elapsed();
        // println!("Lexer Execution >> {:?}", duration);

        //let start = Instant::now();
        let program = match self.parser.parse(program.to_owned()) {
            Ok(parsed) => parsed,
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };

        // println!();
        // for tok in &program {
        //     println!("{:?}", tok);
        // }

        // let duration = start.elapsed();
        // println!("Parser Execution >> {:?}", duration);

        //let start = Instant::now();
        let program = match self.compiler.compile(program, self.filepath.clone()) {
            Ok(parsed) => parsed,
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };
        //let duration = start.elapsed();

        //println!("{:?}", program);
        // println!(
        //     "Size of VmBig: {:?}",
        //     std::mem::size_of::<vm::state::VmBig>()
        // );
        //println!("Compiler Execution >> {:?}", duration);
        //println!("{}", rhexdump::hexdump(&program));

        self.vm.program(program);

        //let start = Instant::now();
        match self.vm.run() {
            Ok(_) => {
                //let duration = start.elapsed();
                // println!("VM Execution >> {:?}", duration);
                // println!();
            }
            Err(error) => {
                error.show();
                std::process::exit(1);
            }
        };
    }
}
