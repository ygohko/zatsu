/*
 * Copyright (c) 2024 Yasuaki Gohko
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE ABOVE LISTED COPYRIGHT HOLDER(S) BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

use hex_string::HexString;
use sha1::Digest;
use sha1::Sha1;
use std::fs;
use std::path::PathBuf;

fn process_file(path: &PathBuf) -> Result<(), ()> {
    let metadata = match fs::metadata(path) {
	Ok(metadata) => metadata,
	Err(_) => return Err(()),
    };
    if metadata.is_file() {
	println!("This is file.");

	let values = match fs::read(path) {
	    Ok(values) => values,
	    Err(_) => return Err(()),
	};
	println!("{} bytes read.", values.len());
	let mut sha1 = Sha1::new();
	sha1.update(values);
	let hash = sha1.finalize();
	let hash_values = hash.to_vec();
	println!("{} bytes of hash generated.", hash_values.len());
	// println!("{}", hash_values);
	let hex = HexString::from_bytes(&hash_values);
	println!("{}", hex.as_string());

    } else {
	println!("This is not file.");
    }

    Ok(())
}

fn main() -> Result<(), ()> {
    println!("Hello, world!");

    let read_dir = match fs::read_dir(".") {
	Ok(read_dir) => read_dir,
	Err(_) => return Err(()),
    };
    for result in read_dir.into_iter() {
	let entry = match result {
	    Ok(entry) => entry,
	    Err(_) => return Err(()),
	};
	let path = entry.path();
	println!("{}", path.display());
	let _ = process_file(&path);
    }

    Ok(())
}
