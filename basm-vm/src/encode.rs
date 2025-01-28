use ahash::AHashMap;
use string_interner::DefaultSymbol;

use crate::{Code, Loc, LocKind, LocThenVal, Sequence, Value};

// TODO: consider encoding length at the start of multi word vars

#[derive(Debug, Default)]
struct Encoder<'a> {
    var_address: AHashMap<DefaultSymbol, u16>,
    mem: &'a mut [u16],
    i: usize,
}

#[derive(Debug)]
pub enum EncodeError {
    MissingSymbol(DefaultSymbol),
}

impl Encoder<'_> {
    fn curr(&mut self) -> &mut [u16] {
        &mut self.mem[self.i..]
    }

    fn write(&mut self, words: &[u16]) {
        self.curr()[..words.len()].copy_from_slice(words);
        self.i += words.len();
    }

    fn loc_address(&self, loc: Loc, code: &Code) -> Result<u16, EncodeError> {
        match loc.location {
            LocKind::Reg(reg) => Ok(reg as u16),
            LocKind::Sym(mem) => {
                if let Some(&address) = code.labels.get(&mem) {
                    return Ok(address);
                }
                if let Some(&address) = self.var_address.get(&mem) {
                    return Ok(address);
                }
                Err(EncodeError::MissingSymbol(mem))
            }
            LocKind::Mem(add) => Ok(add),
        }
    }

    fn value_to_word(&self, val: &Value, code: &Code) -> Result<u16, EncodeError> {
        match val {
            Value::Loc(loc) => self.loc_address(*loc, code),
            Value::Word(word) => Ok(*word),
            Value::Words(words) => Ok(*words.get(0).unwrap_or(&0)),
        }
    }

    fn loc_value_to_words(&self, vl: &LocThenVal, code: &Code) -> Result<[u16; 2], EncodeError> {
        Ok([
            self.loc_address(vl.0, code)?,
            self.value_to_word(&vl.1, code)?,
        ])
    }

    fn loc_code(&self, loc: &Loc) -> u8 {
        match loc.location {
            LocKind::Reg(_) if loc.deref => 0x01,
            LocKind::Reg(_) => 0x00,
            _ if loc.deref => 0x03,
            _ => 0x02,
        }
    }

    fn value_code(&self, val: &Value) -> u8 {
        match val {
            Value::Loc(loc) => self.loc_code(loc),
            Value::Word(_) => 0x04,
            Value::Words(_) => 0x08,
        }
    }

    fn double_value_code(&self, v1: &Value, v2: &Value) -> u8 {
        self.value_code(v1) << 4 | self.value_code(v2)
    }

    fn loc_value_code(&self, vl: &LocThenVal) -> u8 {
        self.loc_code(&vl.0) << 4 | self.value_code(&vl.1)
    }

    fn seq_code_and_values(
        &self,
        seq: &Sequence,
        code: &Code,
    ) -> Result<(u8, [u16; 3]), EncodeError> {
        use Sequence::*;
        Ok(match seq {
            SysCall | Ret => (0, [0; 3]),
            Mov(vl) | Add(vl) | Sub(vl) | Xor(vl) | And(vl) | Or(vl) => {
                let [v1, v2] = self.loc_value_to_words(vl, code)?;
                (self.loc_value_code(vl), [v1, v2, 0])
            }
            Push(value) => {
                let v = self.value_to_word(value, code)?;
                (self.value_code(&value), [v, 0, 0])
            }
            Pop(loc) | Call(loc) | Je(loc) | Jne(loc) | Inc(loc) | Dec(loc) => {
                let v = self.loc_address(*loc, code)?;
                (self.loc_code(&loc), [v, 0, 0])
            }
            Cmp(v1, v2) => {
                let w1 = self.value_to_word(v1, code)?;
                let w2 = self.value_to_word(v2, code)?;
                (self.double_value_code(v1, v2), [w1, w2, 0])
            }
        })
    }

    fn encode(&mut self, code: Code) {
        self.write(&[0, 0]);
        for (name, words) in &code.variables {
            self.write(&[words.len() as u16]);
            self.var_address.insert(*name, self.i as u16);
            self.write(&words);
        }
        self.mem[0] = self.i as u16;
        for seq in &code.sequences {
            let (code, vals) = match self.seq_code_and_values(seq, &code) {
                Ok(v) => v,
                Err(e) => {
                    println!("{e:?}");
                    continue;
                }
            };
            let words = [
                u16::from_le_bytes([code, seq.code()]),
                vals[0],
                vals[1],
                vals[2],
            ];
            self.write(&words);
        }
        self.mem[1] = self.i as u16;
    }
}

pub fn encode(code: Code, mem: &mut [u16]) -> usize {
    let mut enc = Encoder {
        mem,
        i: 0,
        ..Default::default()
    };
    enc.encode(code);
    enc.i
}
