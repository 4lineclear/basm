fn main() {
    let src = read_in().expect("failed to read stdin");
    match basm_vm::BasmVM::parse(&src) {
        Ok(vm) => {
            println!("running:");
            vm.run()
        }
        Err(errs) => {
            let mut o = "".to_owned();
            for err in errs {
                o.push_str(&err.to_string());
            }
            println!("unable to run:{o}");
        }
    }
}

fn read_in() -> std::io::Result<String> {
    use std::io::{stdin, Read};
    let mut out = String::new();
    stdin().read_to_string(&mut out)?;
    Ok(out)
}
