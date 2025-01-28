use std::process::ExitCode;

fn main() -> ExitCode {
    let src = read_in().expect("failed to read stdin");
    match basm_vm::BasmVM::parse(&src) {
        Ok(mut vm) => {
            println!("running:");
            // println!("{:#?}", vm.reg);
            let ec = vm.run();
            // println!("{:#?}", vm.reg);
            ec
        }
        Err(errs) => {
            let mut o = "".to_owned();
            match errs {
                basm_vm::VmError::ParseError(errs) => {
                    for err in errs {
                        o.push_str(&err.to_string());
                    }
                }
                basm_vm::VmError::ReparseError(errs) => {
                    for err in errs {
                        o.push_str(&format!("\n{err:?}"));
                    }
                }
                basm_vm::VmError::EncodeError(errs) => {
                    for err in errs {
                        o.push_str(&format!("\n{err:?}"));
                    }
                }
            }
            println!("unable to run:{o}");
            ExitCode::FAILURE
        }
    }
}

fn read_in() -> std::io::Result<String> {
    use std::io::{stdin, Read};
    let mut out = String::new();
    stdin().read_to_string(&mut out)?;
    Ok(out)
}
