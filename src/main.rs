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
// use serde::Deserialize;
// use serde::Serialize;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use sha1::Digest;
use sha1::Sha1;
// use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct GeneralError {
}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "General error.")
    }
}

impl Error for GeneralError {
}

#[derive(Debug)]
struct TestError {
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "Test error.")
    }
}

impl Error for TestError {
}

#[derive(Serialize, Deserialize)]
struct Entry {
    path: String,
    hash: String,
}

#[derive(Serialize, Deserialize)]
struct Revision {
    entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Repository {
    revisions: Vec<i32>,
}

impl Repository {
    fn load(path: &PathBuf) -> Self {
	// TODO: Imprement this.
	return Repository{
	    revisions: Vec::new(),
	};
    }

    fn save(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let serialized = match serde_json::to_string(self) {
	    Ok(serialized) => serialized,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};
	println!("serialized: {}", serialized);
	let _ = match std::fs::write(path, serialized) {
	    Ok(result) => result,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};

	Ok(())
    }
}

fn process_file(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let metadata = match fs::metadata(path) {
	Ok(metadata) => metadata,
	Err(_) => return Err(Box::new(GeneralError {})),
    };
    let mut hex_string = String::new();
    if metadata.is_file() {
	println!("This is file.");

	let values = match fs::read(path) {
	    Ok(values) => values,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};
	println!("{} bytes read.", values.len());
	let mut sha1 = Sha1::new();
	sha1.update(values.clone());
	let hash = sha1.finalize();
	let hash_values = hash.to_vec();
	println!("{} bytes of hash generated.", hash_values.len());
	// println!("{}", hash_values);
	let hex = HexString::from_bytes(&hash_values);
	hex_string = hex.as_string();
	println!("{}", hex_string);

	let path = format!(".zatsu/{}", hex_string);
	match std::fs::write(path, values) {
	    Ok(()) => (),
	    Err(_) => return Err(Box::new(GeneralError {})),
	};

    } else {
	println!("This is not file.");
    }

    Ok(hex_string)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let read_dir = match fs::read_dir(".") {
	Ok(read_dir) => read_dir,
	Err(_) => return Err(Box::new(TestError {})),
    };

    let mut entries: Vec<Entry> = Vec::new();
    for result in read_dir.into_iter() {
	let entry = match result {
	    Ok(entry) => entry,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};
	let path = entry.path();
	println!("{}", path.display());
	let hash = match process_file(&path) {
	    Ok(hash) => hash,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};
	let entry = Entry{
	    path: path.to_string_lossy().to_string(),
	    hash: hash,
	};
	entries.push(entry);
    }

    let serialized = match serde_json::to_string(&entries) {
	Ok(serialized) => serialized,
	Err(_) => return Err(Box::new(GeneralError {})),
    };

    println!("serialized: {}", serialized);
    let _ = match std::fs::write(".zatsu/revision.json", serialized) {
	Ok(result) => result,
	Err(_) => return Err(Box::new(GeneralError {})),
    };

    Ok(())
}
