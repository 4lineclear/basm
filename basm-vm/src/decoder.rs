use std::u16;

use basm::{
    parse::{ParseError, Parser},
    Basm, Either, Line, Value,
};
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner};

use crate::{Code, GlobalMap, LabelMap, Loc, Sequence, ValueAndLoc, VariableMap};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum DecodeError {
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
struct Decoder {
    si: StringInterner<DefaultBackend>,
    lines: Vec<Line>,
    sequences: Vec<Sequence>,
    variables: VariableMap,
    globals: GlobalMap,
    labels: LabelMap,
}

impl Decoder {
    // TODO: compile errors instead of fail fast.
    fn decode(mut self) -> (Code, Vec<DecodeError>) {
        let errors = (0..self.lines.len())
            .filter_map(|i| match self.decode_line(i) {
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

    fn decode_line(&mut self, line: usize) -> Result<(), DecodeError> {
        use basm::Line::*;
        let line = &self.lines[line];
        Ok(match line {
            NoOp => (),
            Global { name } => {
                self.globals.insert(*name);
            }
            Label { name } => {
                if self.labels.contains_key(name) {
                    return Err(DecodeError::InputError(InputError::DuplicateLabel(*name)));
                }
                self.labels.insert(*name, self.sequences.len() as u16);
            }
            Instruction { ins, values } => {
                let seq = Sequence::decode(&self, ins, values)?;
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
        values: &[Value],
    ) -> Result<Box<[u16]>, DecodeError> {
        match self.resolve(r#type)? {
            "str" => self.parse_str_value(&values),
            "bss" => {
                let [Value::Digit(_, n)] = &values[..] else {
                    return Err(DecodeError::InputError(InputError::InvalidType(r#type)));
                };
                Ok(vec![0; *n as usize].into_boxed_slice())
            }
            _ => Err(DecodeError::InputError(InputError::InvalidType(r#type))),
        }
    }

    fn parse_str_value(&self, values: &[Value]) -> Result<Box<[u16]>, DecodeError> {
        let mut vars = Vec::new();
        for value in values {
            match value {
                Value::Digit(_, n) => vars.push(*n),
                Value::Deref(symbol) | Value::Ident(symbol) | Value::String(symbol) => {
                    var_read_string(&mut vars, self.resolve(*symbol)?);
                }
            }
        }
        Ok(vars.into_boxed_slice())
    }
    fn resolve(&self, symbol: DefaultSymbol) -> Result<&str, DecodeError> {
        self.si
            .resolve(symbol)
            .ok_or_else(|| DecodeError::CompileError(CompileError::InvalidSymbol(symbol)))
    }
}

impl Sequence {
    fn decode(dec: &Decoder, ins: &DefaultSymbol, values: &[Value]) -> Result<Self, DecodeError> {
        use Sequence::*;
        Ok(match dec.resolve(*ins)? {
            // loc, value
            "mov" => Mov(loc_then_value(values)?),
            "add" => Add(loc_then_value(values)?),
            "sub" => Sub(loc_then_value(values)?),
            "xor" => Xor(loc_then_value(values)?),
            "and" => And(loc_then_value(values)?),
            "or" => Or(loc_then_value(values)?),
            // any value
            "push" => Push(single_value(values)?.clone()),
            // loc
            "pop" => Pop(loc(values)?),
            "call" => Call(loc(values)?),
            "je" => Je(loc(values)?),
            "jne" => Jne(loc(values)?),
            "inc" => Inc(loc(values)?),
            "dec" => Dec(loc(values)?),
            "cmp" => {
                let (a, b) = double_value(values)?;
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
                return Err(DecodeError::InputError(InputError::InvalidInstruction(
                    *ins,
                )))
            }
        })
    }
}

fn empty(values: &[Value]) -> Result<(), DecodeError> {
    if let [] = values {
        Ok(())
    } else {
        Err(DecodeError::InputError(InputError::InvalidArgCount {
            exp: 0,
            got: values.len(),
        }))
    }
}

fn loc_then_value(values: &[Value]) -> Result<ValueAndLoc, DecodeError> {
    let (loc, val) = double_value(values)?;
    let (loc, deref) = match loc {
        Value::Deref(s) => (*s, true),
        Value::Ident(s) => (*s, false),
        _ => {
            return Err(DecodeError::InputError(InputError::UnexpectedLiteral(
                loc.clone(),
            )));
        }
    };
    Ok(ValueAndLoc {
        value: val.clone(),
        loc: Loc { deref, loc },
    })
}

fn double_value(values: &[Value]) -> Result<(&Value, &Value), DecodeError> {
    if let [a, b] = values {
        Ok((a, b))
    } else {
        Err(DecodeError::InputError(InputError::InvalidArgCount {
            exp: 2,
            got: values.len(),
        }))
    }
}

fn single_value(values: &[Value]) -> Result<&Value, DecodeError> {
    if let [value] = values {
        Ok(value)
    } else {
        Err(DecodeError::InputError(InputError::InvalidArgCount {
            exp: 1,
            got: values.len(),
        }))
    }
}

fn loc(values: &[Value]) -> Result<Loc, DecodeError> {
    let (loc, deref) = match single_value(values)? {
        Value::Deref(s) => (*s, true),
        Value::Ident(s) => (*s, false),
        _ => {
            return Err(DecodeError::InputError(InputError::UnexpectedLiteral(
                single_value(values)?.clone(),
            )));
        }
    };
    Ok(Loc { loc, deref })
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

pub fn decode(src: &str) -> (Code, Either<Vec<ParseError>, Vec<DecodeError>>) {
    let (Basm { si, lines }, errors) = Parser::base(&src).parse();

    if errors.len() != 0 {
        return (Code::default(), Either::A(errors));
    }

    let (code, err) = Decoder {
        sequences: Vec::with_capacity(lines.len()),
        si,
        lines,
        ..Default::default()
    }
    .decode();
    (code, Either::B(err))
}
