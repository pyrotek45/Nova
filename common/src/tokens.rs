pub type TokenList = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Assign,
    BindVar,
    New,
    AccessCall,
    ModuleCall,
    UserFunctionChain,
    StoreTemp,
    And,
    Or,
    Not,
    Equals,
    Gtr,
    Lss,
    Invert,
    Mod,
    Add,
    Sub,
    Mul,
    Div,
    PopBindings,
    Neg,
    Break,
    Continue,
    ResolveBind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Line information for Error messages
    LinePosition(usize),

    // Identifer types
    Reg(String),
    RegRef(String),
    RegStore(String),
    RegStoreFast(String),

    GlobalReg(String),

    StoreFastBingId(String),
    BindingRef(String),

    // Basic types
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Symbol(char),
    Bool(bool),

    // Block types
    BlockLiteral(TokenList),
    ConditionalBlock(TokenList),
    Doblock(TokenList),
    Function(TokenList, TokenList),
    Closure(TokenList, TokenList, TokenList),
    List(TokenList),
    Arguments(TokenList),

    // Function call
    Call(String),
    CurrentFile(String),
    Op(Operator),

    Entry,

    Pop,
}

impl Token {
    #[inline(always)]
    pub fn precedence(&self) -> usize {
        match self {
            Token::Op(Operator::Assign) => 2,
            Token::Op(Operator::And) => 6,
            Token::Op(Operator::Or) => 7,
            Token::Op(Operator::Not) => 8,
            Token::Op(Operator::Equals) | Token::Op(Operator::Gtr) | Token::Op(Operator::Lss) => 9,
            Token::Op(Operator::Add) | Token::Op(Operator::Sub) => 12,
            Token::Op(Operator::Mul) | Token::Op(Operator::Div) | Token::Op(Operator::Mod) => 13,
            Token::Op(Operator::Invert) => 15,
            _ => 0,
        }
    }
    #[inline(always)]
    pub fn is_left_associative(&self) -> bool {
        match self {
            Token::Op(Operator::Invert) => false,
            Token::Op(Operator::Or) => true,
            Token::Op(Operator::And) => true,
            Token::Op(Operator::Not) => true,
            Token::Op(Operator::Assign) => false,
            Token::Op(Operator::Add) | Token::Op(Operator::Sub) => true,
            Token::Op(Operator::Mul) | Token::Op(Operator::Div) | Token::Op(Operator::Mod) => true,
            _ => true,
        }
    }
}
