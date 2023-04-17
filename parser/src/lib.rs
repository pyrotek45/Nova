use common::tokens::{Operator, Token};

pub struct Parser {
    operator: Vec<Token>,
    output: Vec<Token>,
}

impl Parser {
    #[inline(always)]
    pub fn parse(&mut self, input: Vec<Token>) -> Result<Vec<Token>, common::error::NovaError> {
        for token in input {
            match &token {
                Token::GlobalReg(_) => {
                    self.output.push(token);
                }
                Token::CurrentFile(_) => {
                    self.output.push(token);
                }
                Token::Arguments(_) => {
                    self.output.push(token);
                }
                Token::Bindings(_) => {
                    self.output.push(token);
                }
                Token::ConditionalBlock(block) => {
                    let mut parser = new();
                    let parsed = match parser.parse(block.to_vec()) {
                        Ok(parsed) => parsed,
                        Err(error) => return Err(error),
                    };
                    self.output.push(Token::ConditionalBlock(parsed.to_vec()))
                }
                Token::Doblock(block) => {
                    let mut parser = new();
                    let parsed = match parser.parse(block.to_vec()) {
                        Ok(parsed) => parsed,
                        Err(error) => return Err(error),
                    };
                    self.output.push(Token::Doblock(parsed.to_vec()))
                }
                Token::BlockLiteral(block) => match self.output.last().cloned() {
                    Some(Token::Arguments(input)) => {
                        self.output.pop();
                        if let Some(Token::Arguments(input2)) = self.output.last().cloned() {
                            self.output.pop();
                            let mut parser = new();
                            let parsed = match parser.parse(block.to_vec()) {
                                Ok(parsed) => parsed,
                                Err(error) => return Err(error),
                            };
                            self.output.push(Token::Closure(
                                input2.to_vec(),
                                input.to_vec(),
                                parsed.to_vec(),
                            ))
                        } else {
                            let mut parser = new();
                            let parsed = match parser.parse(block.to_vec()) {
                                Ok(parsed) => parsed,
                                Err(error) => return Err(error),
                            };
                            self.output
                                .push(Token::Function(input.to_vec(), parsed.to_vec()))
                        }
                    }
                    Some(Token::Bindings(input)) => {
                        self.output.pop();

                        let mut parser = new();
                        let parsed = match parser.parse(block.to_vec()) {
                            Ok(parsed) => parsed,
                            Err(error) => return Err(error),
                        };
                        self.output
                            .push(Token::LetBinding(input.to_vec(), parsed.to_vec()))
                    }
                    _ => {
                        let mut parser = new();
                        let parsed = match parser.parse(block.to_vec()) {
                            Ok(parsed) => parsed,
                            Err(error) => return Err(error),
                        };
                        self.output.push(Token::BlockLiteral(parsed.to_vec()))
                    }
                },
                Token::List(block) => {
                    let mut parser = new();
                    let mut parsed = match parser.parse(block.to_vec()) {
                        Ok(parsed) => parsed,
                        Err(error) => return Err(error),
                    };
                    parsed.retain(|x| *x != Token::Symbol(' '));
                    parsed.retain(|x| *x != Token::Symbol(','));
                    parsed.retain(|x| !matches!(*x, Token::LinePosition(_)));
                    self.output.push(Token::List(parsed.to_vec()))
                }
                Token::LinePosition(_) => {
                    self.output.push(token);
                    self.empty_until_open_paren();
                }
                Token::Reg(_)
                | Token::RegRef(_)
                | Token::RegStore(_)
                | Token::RegStoreFast(_)
                | Token::Integer(_)
                | Token::Float(_)
                | Token::BindingRef(_)
                | Token::StoreFastBindId(_)
                | Token::Char(_)
                | Token::String(_) => {
                    self.output.push(token);
                }
                Token::Call(_) => {
                    self.empty_until_open_paren();
                    self.operator.push(token);
                }
                Token::Symbol(',') => {
                    self.empty_until_open_paren();
                }
                Token::Symbol('(') => {
                    self.operator.push(token);
                }
                Token::Symbol(')') => {
                    while let Some(last) = self.operator.pop() {
                        if last == Token::Symbol('(') {
                            break;
                        } else {
                            self.output.push(last);
                        }
                    }
                    if let Some(Token::Call(_)) = self.operator.last() {
                        if let Some(last) = self.operator.pop() {
                            self.output.push(last);
                        }
                    }
                }
                Token::Op(function) => match function {
                    Operator::Add
                    | Operator::Sub
                    | Operator::Mul
                    | Operator::Div
                    | Operator::Equals
                    | Operator::Assign
                    | Operator::Not
                    | Operator::Mod
                    | Operator::And
                    | Operator::Or
                    | Operator::Gtr
                    | Operator::Lss
                    | Operator::Invert => {
                        if let Some(temp) = self.operator.last().cloned() {
                            if temp != Token::Symbol('(') {
                                while let Some(op) = self.operator.last() {
                                    if op.precedence() > token.precedence() {
                                        if let Some(t) = self.operator.pop() {
                                            self.output.push(t);
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                while let Some(op) = self.operator.last() {
                                    if op.precedence() == token.precedence()
                                        && token.is_left_associative()
                                    {
                                        if let Some(t) = self.operator.pop() {
                                            self.output.push(t);
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        self.operator.push(token);
                    }
                    Operator::PopBindings => {
                        self.empty_until_open_paren();
                        self.output.push(token);
                    }
                    Operator::UserFunctionChain
                    | Operator::New
                    | Operator::ResolveBind
                    | Operator::BindVar => {
                        self.output.push(token);
                    }
                    Operator::StoreTemp => {
                        self.operator.push(token.clone());
                        self.output.push(token);
                    }

                    _ => {
                        self.operator.push(token);
                    }
                },
                Token::Function(_, _) => self.operator.push(token),
                _ => {}
            }
        }

        self.emtpy_operators();

        Ok(self.output.to_owned())
    }

    fn empty_until_open_paren(&mut self) {
        while let Some(last) = self.operator.last() {
            match last {
                Token::Symbol('(') => break,
                _ => {
                    if let Some(tok) = self.operator.pop() {
                        self.output.push(tok)
                    }
                }
            }
        }
    }

    fn emtpy_operators(&mut self) {
        while let Some(t) = self.operator.pop() {
            self.output.push(t);
        }
    }
}

pub fn new() -> Parser {
    Parser {
        operator: vec![],
        output: vec![],
    }
}
