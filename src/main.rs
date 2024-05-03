use std::fs;
use std::io;

fn main() -> io::Result<()> {
    println!("Hello, world!");

    let result = fs::read_dir(".")?;
    for entry in result.into_iter() {
	println!("{}", entry?.path().display());
    }

    Ok(())
}
