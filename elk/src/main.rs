use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = env::args().nth(1).expect("usage: elk <file>");
    let input = fs::read(&input_path)?;
    let file =
        delf::File::parse_or_print_error(&input[..]).unwrap_or_else(|| std::process::exit(1));
    println!("{:#?}", file);
    Ok(())
}
