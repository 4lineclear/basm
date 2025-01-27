use ahash::{AHashMap, AHashSet};
use basm::parse::ParseError;
use basm::{Address, Value};
use string_interner::symbol::SymbolU32;
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner};

use self::decoder::{decode, DecodeError};

pub mod decoder;

// TODO: learn how stack works before continuing

#[derive(Debug)]
pub struct BasmVM {
    pub reg: [u16; REGISTER_COUNT],
    pub stack: [u16; STACK_SIZE],
    pub code: Code,
}

type VariableMap = AHashMap<DefaultSymbol, Box<[u16]>>;
type GlobalMap = AHashSet<DefaultSymbol>;
type LabelMap = AHashMap<DefaultSymbol, Address>;

#[derive(Debug, Default)]
pub struct Code {
    pub si: StringInterner<DefaultBackend>,
    pub sequences: Vec<Sequence>,
    pub variables: VariableMap,
    pub globals: GlobalMap,
    pub labels: LabelMap,
}

#[derive(Debug)]
pub enum Sequence {
    Mov(ValueAndLoc),
    Add(ValueAndLoc),
    Sub(ValueAndLoc),
    Xor(ValueAndLoc),
    And(ValueAndLoc),
    Or(ValueAndLoc),
    Push(Value),
    Pop(Loc),
    Call(Loc),
    Je(Loc),
    Jne(Loc),
    Inc(Loc),
    Dec(Loc),
    Cmp(Value, Value),
    SysCall,
    Ret,
}

#[derive(Debug)]
pub struct ValueAndLoc {
    pub value: Value,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct Loc {
    pub deref: bool,
    pub loc: SymbolU32,
}

pub enum VmError {
    ParseError(Vec<ParseError>),
    DecodeError(Vec<DecodeError>),
}

impl BasmVM {
    pub fn parse(src: &str) -> Result<Self, VmError> {
        let (code, err) = decode(src);
        match err {
            basm::Either::A(err) if !err.is_empty() => return Err(VmError::ParseError(err)),
            basm::Either::B(err) if !err.is_empty() => return Err(VmError::DecodeError(err)),
            _ => (),
        }

        let reg = [0; REGISTER_COUNT];
        let stack = [0; STACK_SIZE];
        Ok(Self { reg, stack, code })
    }
    pub fn run(&self) {
        use Sequence::*;
        let mut address = 0;
        while (address as usize) < self.code.sequences.len() {
            #[allow(unused)]
            match &self.code.sequences[address] {
                Mov(ValueAndLoc { value, loc }) => {}
                Add(vl) => (),
                Sub(vl) => (),
                Xor(vl) => (),
                And(vl) => (),
                Or(vl) => (),
                Push(value) => (),
                Pop(loc) => (),
                Call(loc) => (),
                Je(loc) => (),
                Jne(loc) => (),
                Inc(loc) => (),
                Dec(loc) => (),
                Cmp(value, value1) => (),
                SysCall => (),
                Ret => (),
            }
            println!("{:?}", self.code.sequences[address]);
            address += 1;
        }
    }
    fn reg(&self, reg: Register) -> u16 {
        self.reg[reg as usize]
    }
    fn reg_mut(&mut self, reg: Register) -> &mut u16 {
        &mut self.reg[reg as usize]
    }
}

pub const REGISTER_COUNT: usize = 16;
pub const STACK_SIZE: usize = u16::MAX as usize;

// TODO: create run/rest counter part to call/ret
// maybe also a proc instruction?

#[derive(Debug)]
pub enum Register {
    /// accumulator, volatile, return value
    RAX,
    /// base, stable, storage
    RBX,
    /// counter, volatile, arg 4
    RCX,
    /// data, volatile, arg 3
    RDX,
    /// source, stable, arg 2
    RSI,
    /// destination, stable, arg 1
    RDI,
    /// stack pointer, stable, stack top
    RSP,
    /// base pointer, stable, stack bot
    RBP,
    /// register 8, volatile, arg 5
    R08,
    /// register 9, volatile, arg 6
    R09,
    /// register 10, volatile
    R10,
    /// register 11, volatile
    R11,
    /// register 12, stable
    R12,
    /// register 13, stable
    R13,
    /// register 14, stable
    R14,
    /// register 15, stable
    R15,
}

impl std::convert::TryFrom<i32> for Register {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Register::RAX),
            1 => Ok(Register::RBX),
            2 => Ok(Register::RCX),
            3 => Ok(Register::RDX),
            4 => Ok(Register::RSI),
            5 => Ok(Register::RDI),
            6 => Ok(Register::RSP),
            7 => Ok(Register::RBP),
            8 => Ok(Register::R08),
            9 => Ok(Register::R09),
            10 => Ok(Register::R10),
            11 => Ok(Register::R11),
            12 => Ok(Register::R12),
            13 => Ok(Register::R13),
            14 => Ok(Register::R14),
            15 => Ok(Register::R15),
            _ => Err(()),
        }
    }
}
