use std::fs;
use std::io;
use std::path::PathBuf;
use sha1::Digest;
use sha1::Sha1;

fn process_file(path: &PathBuf) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if (metadata.is_file()) {
	println!("This is file.");

	let values = fs::read(path)?;
	println!("{} bytes read.", values.len());
	let mut sha1 = Sha1::new();
	sha1.update(values);
	let hash = sha1.finalize();
	let hash_values = hash.to_vec();
	println!("{} bytes of hash generated.", hash_values.len());
	

    } else {
	println!("This is not file.");
    }

    Ok(())
}

fn main() -> io::Result<()> {
    println!("Hello, world!");

    let result = fs::read_dir(".")?;
    for entry in result.into_iter() {
	let path = entry?.path();
	println!("{}", path.display());
	process_file(&path);
    }

    Ok(())
}
