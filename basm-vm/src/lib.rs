use basm::parse::{ParseError, Parser};
use basm::Basm;

pub const REGISTER_COUNT: usize = u16::MAX as usize;
pub type Register = u16;

#[derive(Debug)]
pub struct BasmVM {
    registers: [Register; REGISTER_COUNT],
    basm: Basm,
}

impl BasmVM {
    pub fn parse(src: &str) -> Result<Self, Vec<ParseError>> {
        let (basm, errors) = Parser::base(&src).parse();
        if errors.len() != 0 {
            return Err(errors);
        }
        let registers = [0; REGISTER_COUNT];
        Ok(Self { registers, basm })
    }
    pub fn run(&self) {
        use basm::Line::*;
        let mut address = 0;
        while (address as usize) < self.basm.lines.len() {
            #[allow(unused)]
            match &self.basm.lines[address] {
                NoOp => (),
                Section { name } => (),
                Label { name } => (),
                Instruction { ins, values } => (),
                Variable {
                    name,
                    r#type,
                    values,
                } => (),
            }
            address += 1;
        }
    }
}
