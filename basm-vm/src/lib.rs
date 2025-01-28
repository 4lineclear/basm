use std::process::ExitCode;

use ahash::{AHashMap, AHashSet};
use string_interner::symbol::SymbolU32;
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner};

use basm::{parse::ParseError, Address};

use self::encode::EncodeError;
use self::reparse::{reparse, ReparseError};

pub mod decode;
pub mod encode;
pub mod reparse;

// TODO: parse values within decoder

#[derive(Debug)]
pub struct BasmVM {
    pub flag: u16,
    pub reg: [u16; REGISTER_COUNT],
    pub mem: [u16; MEM_SIZE],
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

// NOTE: need to load variables into memory early.

#[derive(Debug)]
pub enum Sequence {
    /// Does nothing
    Mov(LocThenVal),
    Add(LocThenVal),
    Sub(LocThenVal),
    Xor(LocThenVal),
    And(LocThenVal),
    Or(LocThenVal),
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

impl Sequence {
    fn code(&self) -> u8 {
        use Sequence::*;
        match self {
            Mov(_) => 0x01,
            Add(_) => 0x02,
            Sub(_) => 0x03,
            Xor(_) => 0x04,
            And(_) => 0x05,
            Or(_) => 0x06,
            Push(_) => 0x07,
            Pop(_) => 0x08,
            Call(_) => 0x09,
            Je(_) => 0x0a,
            Jne(_) => 0x0b,
            Inc(_) => 0x0c,
            Dec(_) => 0x0d,
            Cmp(_, _) => 0x0e,
            SysCall => 0x0f,
            Ret => 0x10,
        }
    }
}

#[derive(Debug)]
pub struct LocThenVal(pub Loc, pub Value);

#[derive(Debug, Clone)]
pub enum Value {
    Loc(Loc),
    Word(u16),
    Words(Box<[u16]>),
}

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub location: LocKind,
    pub deref: bool,
}
#[derive(Debug, Clone, Copy)]
pub enum LocKind {
    Mem(Address),
    Reg(Register),
    Sym(SymbolU32),
}

pub enum VmError {
    ParseError(Vec<ParseError>),
    ReparseError(Vec<ReparseError>),
    EncodeError(Vec<EncodeError>),
}

impl BasmVM {
    pub fn parse(src: &str) -> Result<Self, VmError> {
        let (code, err) = reparse(src);
        match err {
            basm::Either::A(err) if !err.is_empty() => return Err(VmError::ParseError(err)),
            basm::Either::B(err) if !err.is_empty() => return Err(VmError::ReparseError(err)),
            _ => (),
        }

        let reg = [0; REGISTER_COUNT];
        let mut mem = [0; MEM_SIZE];
        encode::encode(code, &mut mem);
        Ok(Self { flag: 0, reg, mem })
    }
    pub fn run(&mut self) -> ExitCode {
        for seq in decode::decode(&self.mem).collect::<Vec<_>>() {
            // println!("{seq:?}");
            use Sequence::*;
            #[allow(unused)]
            match seq {
                Mov(LocThenVal(loc, val)) => *self.loc_mut(loc) = self.value(val),
                Add(LocThenVal(loc, val)) => *self.loc_mut(loc) += self.value(val),
                Sub(LocThenVal(loc, val)) => *self.loc_mut(loc) -= self.value(val),
                Xor(LocThenVal(loc, val)) => *self.loc_mut(loc) ^= self.value(val),
                And(LocThenVal(loc, val)) => *self.loc_mut(loc) &= self.value(val),
                Or(LocThenVal(loc, val)) => *self.loc_mut(loc) |= self.value(val),
                Push(val) => (),
                Pop(l) => (),
                Call(loc) => (),
                Je(l) => (),
                Jne(l) => (),
                Inc(l) => (),
                Dec(l) => (),
                Cmp(v1, v2) => (),
                SysCall => {
                    match self.reg(Register::RAX) {
                        // sys_write
                        0x01 => {
                            let fd = self.reg(Register::RDI);
                            let buf = self.reg(Register::RSI);
                            let count = self.reg(Register::RDX);
                            let start = buf as usize;
                            let end = start + count as usize / 2;

                            let bytes: Vec<_> = self.mem[start..end]
                                .iter()
                                .flat_map(|&w| [(w >> 8) as u8, w as u8])
                                .chain((count % 2 != 0).then(|| self.mem(end as u16) as u8))
                                .collect();
                            print!("{}", String::from_utf8_lossy(&bytes));
                        }
                        // sys_exit
                        0x3C => {
                            return ExitCode::from(self.reg(Register::RDI) as u8);
                        }
                        _ => panic!(),
                    }
                }
                Ret => (),
            }
        }
        ExitCode::default()
    }

    fn flag(&self, flag: Flag) -> bool {
        (self.flag & flag as u16) != 0
    }
    fn set_flag(&mut self, flag: Flag, set: bool) {
        if set {
            self.flag |= flag as u16;
        } else {
            self.flag &= !(flag as u16);
        }
    }
    fn reg(&self, reg: Register) -> u16 {
        self.reg[reg as usize]
    }
    fn reg_mut(&mut self, reg: Register) -> &mut u16 {
        &mut self.reg[reg as usize]
    }
    fn set_reg(&mut self, reg: Register, val: u16) {
        self.reg[reg as usize] = val;
    }
    fn mem(&self, address: u16) -> u16 {
        self.mem[address as usize]
    }
    fn mem_mut(&mut self, address: u16) -> &mut u16 {
        &mut self.mem[address as usize]
    }
    fn loc_mut(&mut self, loc: Loc) -> &mut u16 {
        match loc.location {
            LocKind::Mem(ad) if loc.deref => self.mem_mut(ad),
            LocKind::Mem(ad) => self.mem_mut(ad),
            LocKind::Reg(reg) if loc.deref => self.mem_mut(self.reg(reg)),
            LocKind::Reg(reg) => self.reg_mut(reg),
            LocKind::Sym(_) => unreachable!(),
        }
    }
    fn loc(&mut self, loc: Loc) -> u16 {
        match loc.location {
            LocKind::Mem(ad) if loc.deref => self.mem(self.mem(ad)),
            LocKind::Mem(ad) => ad,
            LocKind::Reg(reg) if loc.deref => self.mem(self.reg(reg)),
            LocKind::Reg(reg) => self.reg(reg),
            LocKind::Sym(_) => unreachable!(),
        }
    }
    fn value(&mut self, val: Value) -> u16 {
        match val {
            Value::Loc(loc) => self.loc(loc),
            Value::Word(v) => v,
            Value::Words(w) => w.get(0).copied().unwrap_or_default(),
        }
    }
}

pub const REGISTER_COUNT: usize = 16;
pub const MEM_SIZE: usize = u16::MAX as usize;

// TODO: create run/rest counter part to call/ret
// maybe also a proc instruction?

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub enum Flag {
    /// sign
    Sf = 0b1,
    /// zero
    Zf = 0b10,
    /// carry
    Cf = 0b100,
    /// auxiliary carry
    Af = 0b1000,
    /// parity
    Pf = 0b10000,
    /// overflow
    Of = 0b100000,
}

impl std::convert::TryFrom<u16> for Register {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, Self::Error> {
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
