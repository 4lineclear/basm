fn main() {
    let src = read_in().expect("failed to read stdin");
    let out = basm_fmt::apply_fmt(&src);
    println!("{out}");
}

fn read_in() -> std::io::Result<String> {
    use std::io::{stdin, Read};
    let mut out = String::new();
    stdin().read_to_string(&mut out)?;
    Ok(out)
}
