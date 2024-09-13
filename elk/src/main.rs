use std::{env, error::Error, fs, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = env::args().nth(1).expect("usage: elk <file>");
    let input = fs::read(&input_path)?;

    println!("Analyzing {:?}...", input_path);

    let file =
        delf::File::parse_or_print_error(&input[..]).unwrap_or_else(|| std::process::exit(1));
    println!("{:#?}", file);

    println!("Executing {:?}...", input_path);
    let status = Command::new(input_path).status()?;
    if !status.success() {
        return Err("process did not exit successfully".into());
    }

    Ok(())
}
