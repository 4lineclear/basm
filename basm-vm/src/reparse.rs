use std::{str::FromStr, u16};

use basm::{
    parse::{ParseError, Parser},
    Basm, Either, Line, Value as PValue,
};
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner};

use crate::{
    Code, GlobalMap, LabelMap, Loc, LocKind, LocThenVal, Register, Sequence, Value, VariableMap,
};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum ReparseError {
    CompileError(CompileError),
    InputError(InputError),
}

#[derive(Debug)]
pub enum CompileError {
    InvalidSymbol(DefaultSymbol),
}

#[derive(Debug)]
pub enum InputError {
    InvalidType(DefaultSymbol),
    InvalidInstruction(DefaultSymbol),
    InvalidArgCount { exp: usize, got: usize },
    UnexpectedLiteral(Value),
    DuplicateLabel(DefaultSymbol),
}

#[derive(Default)]
struct Reparser {
    si: StringInterner<DefaultBackend>,
    lines: Vec<Line>,
    sequences: Vec<Sequence>,
    variables: VariableMap,
    globals: GlobalMap,
    labels: LabelMap,
}

impl Reparser {
    // TODO: compile errors instead of fail fast.
    fn reparse(mut self) -> (Code, Vec<ReparseError>) {
        let errors: Vec<_> = (0..self.lines.len())
            .filter_map(|i| match self.reparse_line(i) {
                Ok(()) => None,
                Err(e) => Some(e),
            })
            .collect();

        let code = Code {
            sequences: self.sequences,
            si: self.si,
            variables: self.variables,
            globals: self.globals,
            labels: self.labels,
        };
        (code, errors)
    }

    fn reparse_line(&mut self, line: usize) -> Result<(), ReparseError> {
        use basm::Line::*;
        let line = &self.lines[line];
        Ok(match line {
            NoOp => (),
            Global { name } => {
                self.globals.insert(*name);
            }
            Label { name } => {
                if self.labels.contains_key(name) {
                    return Err(ReparseError::InputError(InputError::DuplicateLabel(*name)));
                }
                self.labels.insert(*name, self.sequences.len() as u16);
            }
            Instruction { ins, values } => {
                let seq = Sequence::reparse(&self, ins, values)?;
                self.sequences.push(seq);
            }
            Variable {
                name,
                r#type,
                values,
            } => {
                let value = self.handle_var(*r#type, &values)?;
                self.variables.insert(*name, value);
            }
        })
    }

    fn handle_var(
        &self,
        r#type: DefaultSymbol,
        values: &[PValue],
    ) -> Result<Box<[u16]>, ReparseError> {
        match self.resolve(r#type)? {
            "str" => self.parse_str_value(&values),
            "bss" => {
                let [PValue::Digit(_, n)] = &values[..] else {
                    return Err(ReparseError::InputError(InputError::InvalidType(r#type)));
                };
                Ok(vec![0; *n as usize].into_boxed_slice())
            }
            _ => Err(ReparseError::InputError(InputError::InvalidType(r#type))),
        }
    }

    fn parse_str_value(&self, values: &[PValue]) -> Result<Box<[u16]>, ReparseError> {
        let mut vars = Vec::new();
        for value in values {
            match value {
                PValue::Digit(_, n) => vars.push(*n),
                PValue::Deref(symbol) | PValue::Ident(symbol) | PValue::String(symbol) => {
                    var_read_string(&mut vars, self.resolve(*symbol)?);
                }
            }
        }
        Ok(vars.into_boxed_slice())
    }

    fn loc_then_value(&self, values: &[PValue]) -> Result<LocThenVal, ReparseError> {
        let (loc, value) = self.double_value(values)?;
        let Value::Loc(loc) = loc else {
            return Err(ReparseError::InputError(InputError::UnexpectedLiteral(
                loc.clone(),
            )));
        };
        Ok(LocThenVal(loc, value))
    }

    fn loc(&self, values: &[PValue]) -> Result<Loc, ReparseError> {
        let val = self.single_value(values)?;
        if let Value::Loc(loc) = val {
            Ok(loc)
        } else {
            Err(ReparseError::InputError(InputError::UnexpectedLiteral(
                val.clone(),
            )))
        }
    }

    fn double_value(&self, values: &[PValue]) -> Result<(Value, Value), ReparseError> {
        if let [a, b] = values {
            Ok((self.reparse_value(a)?, self.reparse_value(b)?))
        } else {
            Err(ReparseError::InputError(InputError::InvalidArgCount {
                exp: 2,
                got: values.len(),
            }))
        }
    }

    fn single_value(&self, values: &[PValue]) -> Result<Value, ReparseError> {
        if let [value] = values {
            self.reparse_value(value)
        } else {
            Err(ReparseError::InputError(InputError::InvalidArgCount {
                exp: 1,
                got: values.len(),
            }))
        }
    }

    fn reparse_value(&self, value: &PValue) -> Result<Value, ReparseError> {
        Ok(match value {
            PValue::Deref(sym) => Value::Loc(Loc {
                location: Register::from_str(self.resolve(*sym)?)
                    .map_or(LocKind::Sym(*sym), LocKind::Reg),
                deref: true,
            }),
            PValue::Ident(sym) => Value::Loc(Loc {
                location: Register::from_str(self.resolve(*sym)?)
                    .map_or(LocKind::Sym(*sym), LocKind::Reg),
                deref: false,
            }),
            PValue::String(sym) => {
                let mut words = Vec::new();
                var_read_string(&mut words, self.resolve(*sym)?);
                Value::Words(words.into())
            }
            PValue::Digit(_, n) => Value::Word(*n),
        })
    }

    fn resolve(&self, symbol: DefaultSymbol) -> Result<&str, ReparseError> {
        self.si
            .resolve(symbol)
            .ok_or_else(|| ReparseError::CompileError(CompileError::InvalidSymbol(symbol)))
    }
}

impl Sequence {
    fn reparse(
        dec: &Reparser,
        ins: &DefaultSymbol,
        values: &[PValue],
    ) -> Result<Self, ReparseError> {
        use Sequence::*;
        Ok(match dec.resolve(*ins)? {
            // loc, value
            "mov" => Mov(dec.loc_then_value(values)?),
            "add" => Add(dec.loc_then_value(values)?),
            "sub" => Sub(dec.loc_then_value(values)?),
            "xor" => Xor(dec.loc_then_value(values)?),
            "and" => And(dec.loc_then_value(values)?),
            "or" => Or(dec.loc_then_value(values)?),
            // any value
            "push" => Push(dec.single_value(values)?.clone()),
            // loc
            "pop" => Pop(dec.loc(values)?),
            "call" => Call(dec.loc(values)?),
            "je" => Je(dec.loc(values)?),
            "jne" => Jne(dec.loc(values)?),
            "inc" => Inc(dec.loc(values)?),
            "dec" => Dec(dec.loc(values)?),
            "cmp" => {
                let (a, b) = dec.double_value(values)?;
                Cmp(a.clone(), b.clone())
            }
            // nil
            "syscall" => {
                empty(values)?;
                SysCall
            }
            "ret" => {
                empty(values)?;
                Ret
            }
            _ => {
                return Err(ReparseError::InputError(InputError::InvalidInstruction(
                    *ins,
                )))
            }
        })
    }
}

fn empty(values: &[PValue]) -> Result<(), ReparseError> {
    if let [] = values {
        Ok(())
    } else {
        Err(ReparseError::InputError(InputError::InvalidArgCount {
            exp: 0,
            got: values.len(),
        }))
    }
}

impl FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Register::*;
        Ok(match s {
            "rax" => RAX,
            "rbx" => RBX,
            "rcx" => RCX,
            "rdx" => RDX,
            "rsi" => RSI,
            "rdi" => RDI,
            "rsp" => RSP,
            "rbp" => RBP,
            "r08" => R08,
            "r09" => R09,
            "r10" => R10,
            "r11" => R11,
            "r12" => R12,
            "r13" => R13,
            "r14" => R14,
            "r15" => R15,
            _ => return Err(()),
        })
    }
}

// TODO: encode first word as length of str

fn var_read_string(vars: &mut Vec<u16>, s: &str) {
    let mut ce = s.as_bytes().chunks_exact(2);
    while let Some(&[b, a]) = ce.next() {
        vars.push(u16::from_le_bytes([a, b]));
    }
    if let [b] = ce.remainder() {
        vars.push(u16::from_le_bytes([0, *b]));
    }
}

pub fn reparse(src: &str) -> (Code, Either<Vec<ParseError>, Vec<ReparseError>>) {
    let (Basm { si, lines }, errors) = Parser::base(&src).parse();

    if errors.len() != 0 {
        return (Code::default(), Either::A(errors));
    }

    let (code, err) = Reparser {
        sequences: Vec::with_capacity(lines.len()),
        si,
        lines,
        ..Default::default()
    }
    .reparse();
    (code, Either::B(err))
}
