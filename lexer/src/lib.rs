use common::{
    error::NovaError,
    tokens::{Operator, Token, TokenList},
};

fn extract_current_directory(path: &str) -> Option<String> {
    if let Some(last_slash_index) = path.rfind('/') {
        return Some(path[..last_slash_index + 1].to_string());
    }
    None
}

pub enum LexFrame {
    Paren(usize, usize),
    Block(usize, usize),
    String(usize, usize),
    List(usize, usize),
}

pub struct Lexer {
    line: usize,
    row: usize,
    filepath: String,
    file: String,
    output: Vec<TokenList>,
    buffer: String,
    frames: Vec<LexFrame>,
    globals: common::table::Table<String>,

    is_parsing_stringdq: bool,
    is_parsing_char: bool,
    is_parsing_comment: bool,
}

pub fn new() -> Lexer {
    Lexer {
        line: 1,
        filepath: String::new(),
        file: String::new(),
        output: vec![vec![]],
        buffer: String::new(),
        frames: Vec::new(),
        row: 0,
        globals: common::table::new(),
        is_parsing_stringdq: false,
        is_parsing_char: false,
        is_parsing_comment: false,
    }
}

impl Lexer {
    #[inline(always)]
    pub fn clear(&mut self) {
        self.file.clear()
    }

    #[inline(always)]
    pub fn insert_string(&mut self, input: &str) {
        self.file.push_str(input);
    }

    #[inline(always)]
    pub fn open_file(&mut self, filepath: &str) -> Result<(), String> {
        match std::fs::read_to_string(filepath) {
            Ok(content) => {
                self.filepath = filepath.to_owned();
                self.file = content;
                Ok(())
            }
            Err(_) => Err(format!("file: {} could not be opened", filepath)),
        }
    }

    #[inline(always)]
    fn push_char(&mut self, char: char) {
        self.buffer.push(char)
    }

    #[inline(always)]
    fn check_token_buffer(&mut self) -> Option<Token> {
        if !self.buffer.is_empty() {
            if let Ok(v) = self.buffer.parse() {
                return Some(if self.buffer.contains('.') {
                    Token::Float(v)
                } else {
                    Token::Integer(v as i64)
                });
            }
            return Some(Token::Reg(self.buffer.to_lowercase()));
        }
        None
    }

    #[inline(always)]
    fn take_last_token(&mut self) -> Option<Token> {
        self.output.last_mut().and_then(|last| last.pop())
    }

    #[inline(always)]
    fn last_token(&self) -> Option<&Token> {
        self.output.last().and_then(|last| last.last())
    }

    #[inline(always)]
    pub fn check_token(&mut self) -> Result<(), NovaError> {
        if let Some(token) = self.check_token_buffer() {
            match token {
                Token::Reg(id) => match self.last_token() {
                    Some(Token::Symbol('&')) => {
                        self.take_last_token();
                        self.push_token(Token::RegRef(id))
                    }
                    Some(Token::Reg(last)) => match last.as_str() {
                        "mod" => {
                            self.take_last_token();
                            if self.globals.has(&id) {
                                return Err(common::error::lexer_error(
                                    format!("Module {} is already defined", id),
                                    "Cannot redefine a module".to_string(),
                                    self.line,
                                    self.row - id.len(),
                                    self.filepath.clone(),
                                ));
                            }
                            self.globals.insert(id.to_string());
                            self.push_token(Token::GlobalReg(id))
                        }
                        _ => self.push_token(Token::Reg(id)),
                    },
                    _ => self.push_token(Token::Reg(id)),
                },
                _ => {
                    self.push_token(token);
                }
            }
        }
        self.buffer.clear();
        Ok(())
    }

    #[inline(always)]
    fn push_token(&mut self, token: Token) {
        if let Some(last) = self.output.last_mut() {
            last.push(token)
        }
    }

    #[inline(always)]
    pub fn parse(&mut self) -> Result<&TokenList, NovaError> {
        if self.file.is_empty() {
            return Err(common::error::file_error(
                "Lexer has no file to parse".to_string(),
            ));
        }

        let binding = self.file.clone();
        let mut chars = binding.chars().peekable();

        while let Some(char) = chars.next() {
            self.row += 1;
            if self.is_parsing_comment {
                if char != '\n' {
                    continue;
                } else {
                    self.is_parsing_comment = false;
                }
            }

            if self.is_parsing_stringdq {
                if char != '"' {
                    self.push_char(char);
                    continue;
                } else {
                    self.is_parsing_stringdq = false;
                    match self.last_token() {
                        Some(Token::Reg(caller)) => match caller.as_str() {
                            "import" => {
                                self.take_last_token();
                                let mut lexer = new();

                                if let Some(mut current_directory) =
                                    extract_current_directory(&self.filepath)
                                {
                                    current_directory.push_str(&self.buffer);
                                    match lexer.open_file(&current_directory) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            println!(
                                                "Cant open {}. Current dir: {}",
                                                current_directory, self.filepath
                                            )
                                        }
                                    }
                                } else {
                                    match lexer.open_file(&self.buffer) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            println!(
                                                "Cant open {}. Current dir: {}",
                                                &self.buffer, self.filepath
                                            )
                                        }
                                    }
                                };

                                let mut program = match lexer.parse() {
                                    Ok(lexed) => lexed.to_vec(),
                                    Err(error) => {
                                        error.show();
                                        std::process::exit(1);
                                    }
                                };
                                self.push_token(Token::CurrentFile(self.buffer.clone()));
                                self.output.last_mut().unwrap().append(&mut program);
                                self.push_token(Token::CurrentFile(self.filepath.clone()));
                            }

                            _ => {
                                self.push_token(Token::String(self.buffer.clone()));
                            }
                        },
                        _ => {
                            self.push_token(Token::String(self.buffer.clone()));
                        }
                    }

                    self.buffer.clear();
                    continue;
                }
            }

            if self.is_parsing_char {
                if char != '\'' {
                    self.push_char(char);
                    continue;
                } else {
                    self.is_parsing_char = false;
                    if self.buffer.len() > 1 {
                        return Err(common::error::lexer_error(
                            "Char cannot contain more than one character".to_string(),
                            "Try using double quotes instead, if you need a string".to_string(),
                            self.line,
                            self.row - self.buffer.len(),
                            self.filepath.clone(),
                        ));
                    }

                    if let Some(c) = self.buffer.chars().next() {
                        self.push_token(Token::Char(c));
                    }

                    self.buffer.clear();
                    continue;
                }
            }

            match char {
                '\'' => {
                    self.is_parsing_char = true;
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }
                '"' => {
                    self.is_parsing_stringdq = true;
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }
                // newline
                '\n' => {
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                    self.push_token(Token::LinePosition(self.line));
                    self.line += 1;
                    self.row = 0;
                }
                // Letters and numbers
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' | '.' | ':' => {
                    self.push_char(char);
                }
                // Spaces
                ' ' => match self.check_token() {
                    Ok(_) => {}
                    Err(error) => return Err(error),
                },

                // Symbols
                '+' | '*' | '/' | '(' | ')' | '<' | '>' | '`' | '~' | '@' | '%' | '^' | '&'
                | ',' | '?' | ';' | '!' | '$' | '|' | '=' | '-' => {
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                    match char {
                        '-' => match self.last_token() {
                            Some(Token::Reg(_))
                            | Some(Token::RegRef(_))
                            | Some(Token::RegStore(_))
                            | Some(Token::Integer(_))
                            | Some(Token::Float(_))
                            | Some(Token::Symbol(')')) => {
                                self.push_token(Token::Op(Operator::Sub));
                            }
                            _ => {
                                self.push_token(Token::Op(Operator::Neg));
                            }
                        },
                        '/' => match chars.peek() {
                            Some(&'/') => {
                                chars.next();
                                self.is_parsing_comment = true;
                            }
                            _ => self.push_token(Token::Op(Operator::Div)),
                        },
                        '@' => self.push_token(Token::Symbol(char)),
                        '?' => self.push_token(Token::Symbol(char)),
                        '&' => self.push_token(Token::Symbol(char)),
                        ',' => self.push_token(Token::Symbol(char)),
                        '<' => self.push_token(Token::Op(Operator::Lss)),
                        '>' => self.push_token(Token::Op(Operator::Gtr)),
                        '!' => self.push_token(Token::Op(Operator::Not)),
                        '%' => self.push_token(Token::Op(Operator::Mod)),
                        '*' => self.push_token(Token::Op(Operator::Mul)),
                        '+' => self.push_token(Token::Op(Operator::Add)),

                        '(' => {
                            // check for function calls
                            match self.take_last_token() {
                                Some(Token::Reg(caller)) => {
                                    if caller.as_str() == "import" {
                                    } else {
                                        self.push_token(Token::Call(caller))
                                    }
                                }
                                Some(Token::Symbol(')')) => {}
                                Some(token) => self.push_token(token),
                                None => {}
                            }
                            self.push_token(Token::Symbol(char));
                            self.frames.push(LexFrame::Paren(self.line, self.row))
                        }
                        ')' => {
                            if let Some(LexFrame::Paren(_, _)) = self.frames.pop() {
                                self.push_token(Token::Symbol(char));
                            } else {
                                self.frames.push(LexFrame::Paren(self.line, self.row));
                            }
                        }
                        '=' => {
                            match chars.peek() {
                                Some(&'=') => {
                                    // push equality
                                    self.push_token(Token::Op(Operator::Equals));
                                    chars.next();
                                    self.row += 1;
                                }
                                _ => match self.take_last_token() {
                                    Some(Token::Reg(id)) => {
                                        self.push_token(Token::RegStore(id));
                                        self.push_token(Token::Op(Operator::Assign));
                                    }
                                    Some(Token::RegRef(id)) => {
                                        self.push_token(Token::RegRef(id));
                                        self.push_token(Token::Op(Operator::Assign));
                                    }
                                    Some(Token::GlobalReg(id)) => {
                                        self.push_token(Token::GlobalReg(id));
                                        self.push_token(Token::Op(Operator::Assign));
                                    }
                                    Some(Token::Symbol(')')) => {
                                        self.push_token(Token::Symbol(')'));
                                        self.push_token(Token::Op(Operator::Assign));
                                    }
                                    _ => {
                                        return Err(common::error::lexer_error(
                                            "Assingment is missing Identifier".to_string(),
                                            "Try putting a varaible befere the = Assingment"
                                                .to_string(),
                                            self.line,
                                            self.row,
                                            self.filepath.clone(),
                                        ));
                                    }
                                },
                            }
                        }
                        _ => {
                            return Err(common::error::lexer_error(
                                format!("Unknown char {}", char),
                                "Try removing this character".to_string(),
                                self.line,
                                self.row,
                                self.filepath.clone(),
                            ));
                        }
                    }
                }

                // Parsing blocks
                '{' => {
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                    self.frames.push(LexFrame::Block(self.line, self.row));
                    self.output.push(vec![]);
                }
                '}' => {
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                    match self.frames.pop() {
                        Some(LexFrame::Block(_, _)) => {
                            if let Some(block) = self.output.pop() {
                                match self.last_token() {
                                    Some(Token::Symbol('?')) => {
                                        self.take_last_token();
                                        self.push_token(Token::Symbol(','));
                                        self.push_token(Token::ConditionalBlock(block.to_vec()))
                                    }
                                    Some(Token::Symbol('@')) => {
                                        self.take_last_token();
                                        self.push_token(Token::Doblock(block.to_vec()))
                                    }
                                    _ => self.push_token(Token::BlockLiteral(block.to_vec())),
                                }
                            }
                        }
                        _ => {
                            return Err(common::error::lexer_error(
                                "Unbalanced or unexpected brace".to_string(),
                                "Missing opening brace".to_string(),
                                self.line,
                                self.row,
                                self.filepath.clone(),
                            ))
                        }
                    }
                }

                // Parsing blocks
                '[' => {
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                    self.frames.push(LexFrame::List(self.line, self.row));
                    self.output.push(vec![]);
                }
                ']' => {
                    match self.check_token() {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                    match self.frames.pop() {
                        Some(LexFrame::List(_, _)) => {
                            if let Some(block) = self.output.pop() {
                                if let Some(&':') = chars.peek() {
                                    self.push_token(Token::Arguments(block.to_vec()));
                                    chars.next();
                                    self.row += 1;
                                } else {
                                    self.push_token(Token::List(block.to_vec()));
                                }
                            }
                        }
                        _ => {
                            return Err(common::error::lexer_error(
                                "Unbalanced or unexpected brace".to_string(),
                                "Missing opening brace".to_string(),
                                self.line,
                                self.row,
                                self.filepath.clone(),
                            ))
                        }
                    }
                }

                _ => {}
            }
        }

        // Last token check
        match self.check_token() {
            Ok(_) => {}
            Err(error) => return Err(error),
        }

        // Make sure no frames are left, if so its an error
        if let Some(frame) = self.frames.pop() {
            let (msg, line, row) = match frame {
                LexFrame::Block(line, row) => {
                    ("Unbalanced or unexpected brace".to_string(), line, row)
                }
                LexFrame::String(line, row) => ("String Left open".to_string(), line, row),
                LexFrame::List(line, row) => ("List Left open".to_string(), line, row),
                LexFrame::Paren(line, row) => ("Parenthesis Left open".to_string(), line, row),
            };
            return Err(common::error::lexer_error(
                msg,
                "Failed after lexing".to_string(),
                line,
                row,
                self.filepath.clone(),
            ));
        }

        // println!("{}", self.buffer);

        self.output.get(0).ok_or_else(|| {
            common::error::file_error(
                "Failed to parse, Could not retrieve last index on output".to_string(),
            )
        })
    }
}
