use crate::{Loc, LocKind, LocThenVal, Register, Sequence, Value};

#[derive(Debug, Clone, Copy)]
struct SeqCode(u8);

impl SeqCode {
    fn is_loc(self) -> bool {
        self.0 & 0b1100 == 0
    }
    fn is_deref(self) -> bool {
        self.0 & 0b0001 == 1
    }
    fn is_reg(self) -> bool {
        self.0 & 0b0010 == 0
    }
    #[allow(unused)]
    fn is_word(self) -> bool {
        self.0 & 0b1100 == 0b0100
    }
    #[allow(unused)]
    fn is_multi_word(self) -> bool {
        self.0 & 0b1100 == 0b1000
    }
}

pub fn decode(mem: &[u16]) -> impl Iterator<Item = Sequence> + '_ {
    mem[mem[0] as usize..mem[1] as usize]
        .chunks(4)
        .filter_map(|word| {
            let &[ins, v1, v2, _] = word else {
                return None;
            };
            // println!("{ins:#06x} {v1:#06x} {v2:#06x}");
            decode_seq(ins, v1, v2)
        })
}

fn loc(sq: &SeqCode, v: u16) -> Option<Loc> {
    if !sq.is_loc() {
        return None;
    }
    // println!("here! {sq:?} -> {v}");
    let loc = Loc {
        location: if sq.is_reg() {
            LocKind::Reg(Register::try_from(v).ok()?)
        } else {
            LocKind::Mem(v)
        },
        deref: sq.is_deref(),
    };
    Some(loc)
}

fn value(sq: &SeqCode, v: u16) -> Option<Value> {
    if let Some(loc) = loc(sq, v) {
        return Some(Value::Loc(loc));
    }
    Some(Value::Word(v))
}

fn loc_then_val(sq1: &SeqCode, sq2: &SeqCode, v1: u16, v2: u16) -> Option<LocThenVal> {
    let loc = loc(sq1, v1)?;
    let val = value(sq2, v2)?;
    Some(LocThenVal(loc, val))
}

pub fn decode_seq(ins: u16, v1: u16, v2: u16) -> Option<Sequence> {
    use Sequence::*;
    let sq = ins as u8;
    let sq1 = SeqCode(sq >> 4);
    let sq2 = SeqCode(sq & 0x00ff);
    // println!("{v1:?} {v2:?}");

    Some(match ins >> 8 {
        0x01 => Mov(loc_then_val(&sq1, &sq2, v1, v2)?),
        0x02 => Add(loc_then_val(&sq1, &sq2, v1, v2)?),
        0x03 => Sub(loc_then_val(&sq1, &sq2, v1, v2)?),
        0x04 => Xor(loc_then_val(&sq1, &sq2, v1, v2)?),
        0x05 => And(loc_then_val(&sq1, &sq2, v1, v2)?),
        0x06 => Or(loc_then_val(&sq1, &sq2, v1, v2)?),
        0x07 => Push(value(&sq2, v1)?),
        0x08 => Pop(loc(&sq2, v1)?),
        0x09 => Call(loc(&sq2, v1)?),
        0x0a => Je(loc(&sq2, v1)?),
        0x0b => Jne(loc(&sq2, v1)?),
        0x0c => Inc(loc(&sq2, v1)?),
        0x0d => Dec(loc(&sq2, v1)?),
        0x0e => Cmp(value(&sq1, v1)?, value(&sq2, v2)?),
        0x0f => SysCall,
        0x10 => Ret,
        _ => return None,
    })
}
