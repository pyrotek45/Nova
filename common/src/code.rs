pub struct Code {}

impl Code {
    pub const RET: u8 = 0;

    pub const INTEGER: u8 = 1;
    pub const FLOAT: u8 = 2;

    pub const ADD: u8 = 3;
    pub const SUB: u8 = 4;
    pub const MUL: u8 = 5;
    pub const DIV: u8 = 6;

    pub const STOREID: u8 = 7;
    pub const ID: u8 = 8;
    pub const STOREFASTID: u8 = 9;

    pub const ASSIGN: u8 = 10;
    pub const ALLOCATEREG: u8 = 11;

    pub const CALL: u8 = 12;
    pub const BLOCK: u8 = 13;
    pub const DIRECTCALL: u8 = 14;

    pub const NEWLIST: u8 = 15;

    pub const TRUE: u8 = 16;
    pub const FALSE: u8 = 17;

    pub const FUNCTION: u8 = 18;

    pub const GTR: u8 = 20;
    pub const LSS: u8 = 21;

    pub const JUMPIFFALSE: u8 = 22;

    pub const REC: u8 = 23;

    pub const IF: u8 = 24;
    pub const WHEN: u8 = 25;

    pub const EQUALS: u8 = 26;
    pub const MODULO: u8 = 27;

    pub const REFID: u8 = 28;

    pub const CLOSURE: u8 = 29;
    pub const CID: u8 = 30;

    pub const STRING: u8 = 31;

    pub const FOR: u8 = 32;
    pub const BOUNCE: u8 = 33;

    pub const RANGE: u8 = 34;
    pub const FORINT: u8 = 35;

    pub const BYTE: u8 = 36;

    pub const NATIVE: u8 = 37;

    pub const STOREGLOBAL: u8 = 38;
    pub const GLOBALID: u8 = 39;
    pub const ALLOCATEGLOBAL: u8 = 40;

    pub const CHAR: u8 = 41;

    pub const POP: u8 = 42;

    pub const NEG: u8 = 43;

    pub const BREAK: u8 = 44;

    pub const NEWBINDING: u8 = 45;
    pub const POPBINDING: u8 = 46;

    pub const STOREBIND: u8 = 47;
    pub const GETBIND: u8 = 48;
}
