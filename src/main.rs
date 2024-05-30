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

mod entry;
mod error;
mod file_path_producer;
mod revision;
mod repository;

use hex_string::HexString;
use sha1::Digest;
use sha1::Sha1;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::entry::Entry;
use crate::error::ZatsuError;
use crate::revision::Revision;
use crate::repository::Repository;

const ERROR_GENERAL: i32 = 0;
const ERROR_READING_META_DATA_FAILED: i32 = 1;
const ERROR_READING_DIRECTORY_FAILED: i32 = 2;
const ERROR_CREATING_REPOSITORY_FAILED: i32 = 3;
const ERROR_LOADING_REPOSITORY_FAILED: i32 = 4;
const ERROR_REVISION_NOT_FOUND: i32 = 5;
const ERROR_LOADING_REVISION_FAILED: i32 = 6;
const ERROR_FILE_NOT_FOUND: i32 = 7;
const ERROR_LOADING_FILE_FAILED: i32 = 8;
const ERROR_SAVING_FILE_FAILED: i32 = 9;
const ERROR_SERIALIZATION_FAILED: i32 = 10;
const ERROR_PRODUCING_FINISHED: i32 = 11;

struct FilePathProducer {
    file_paths: Vec<String>,
    directory_paths: Vec<String>,
}

impl FilePathProducer {
    fn new(path: String) -> FilePathProducer {
	return FilePathProducer {
	    file_paths: Vec::new(),
	    directory_paths: vec![path],
	};
    }

    fn next(&mut self) -> Result<String, ZatsuError> {
	let mut done = false;
	while !done {
	    if self.file_paths.len() > 0 {
		let path = self.file_paths.pop().unwrap();

		return Ok(path);
	    }
	    
	    if self.directory_paths.len() == 0 {
		return Err(ZatsuError::new("FilePathProducer".to_string(), ERROR_PRODUCING_FINISHED));
	    }
	    let directory_path = self.directory_paths.pop().unwrap();

	    println!("Reading directory: {}", directory_path);
	    
	    let mut scan = true;
	    let option = Path::new(&directory_path).file_name();
	    if option.is_some() {
		let file_name = option.unwrap().to_string_lossy().to_string();
		if file_name == ".zatsu".to_string() || file_name == ".jj".to_string() || file_name == ".git".to_string() {
		    scan = false;
		}
	    }

	    if scan {
		let read_dir = match fs::read_dir(directory_path) {
		    Ok(read_dir) => read_dir,
		    Err(_) => return Err(ZatsuError::new("FilePathProducer".to_string(), ERROR_READING_DIRECTORY_FAILED)),
		};
		for result in read_dir {
		    if result.is_ok() {
			let entry = result.unwrap();

			let metadata = match fs::metadata(entry.path()) {
			    Ok(metadata) => metadata,
			    Err(_) => return Err(ZatsuError::new("FilePathProducer".to_string(), ERROR_READING_META_DATA_FAILED)),
			};
			let path = entry.path().to_string_lossy().to_string();
			if metadata.is_file() {

			    println!("Adding to file_paths: {}", path);

			    self.file_paths.push(path);
			}
			else {

			    println!("Adding to directory_paths: {}", path);

			    self.directory_paths.push(path);
			}
		    }
		}
	    }
	}

	Err(ZatsuError::new("FilePathProducer".to_string(), ERROR_PRODUCING_FINISHED))
    }
}

fn process_file(path: &PathBuf) -> Result<String, ZatsuError> {
    let metadata = match fs::metadata(path) {
	Ok(metadata) => metadata,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_READING_META_DATA_FAILED)),
    };
    let mut hex_string = String::new();
    if metadata.is_file() {
	println!("This is file.");

	let values = match fs::read(path) {
	    Ok(values) => values,
	    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
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

	let path = format!(".zatsu/objects/{}", hex_string);
	match std::fs::write(path, values) {
	    Ok(()) => (),
	    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};

    } else {
	println!("This is not file.");
    }

    Ok(hex_string)
}

fn process_commit() -> Result<(), ZatsuError> {
    let mut repository = match Repository::load(&PathBuf::from(".zatsu/repository.json")) {
	Ok(repository) => repository,
	Err(_) => Repository {
	    revisions: Vec::new(),
	},
    };
    let latest_revision = repository.latest_revision();
    let revision_number = latest_revision + 1;

    let mut producer = FilePathProducer::new(".".to_string());
    let mut revision = Revision {
	entries: Vec::new(),
    };
    let mut done = false;
    while !done {
	let result = producer.next();
	if result.is_ok() {
	    let path = result.unwrap();
	    println!("{}", path);
	    let hash = match process_file(&PathBuf::from(path.clone())) {
		Ok(hash) => hash,
		Err(error) => return Err(error),
	    };
	    let entry = Entry{
		path: path,
		hash: hash,
	    };
	    revision.entries.push(entry);
	}
	else {
	    let error = result.unwrap_err();

	    println!("error.code: {}", error.code);

	    if error.code == ERROR_PRODUCING_FINISHED {
		done = true;
	    }
	}
    }

    // TODO: Move to revision.rs.
    let serialized = match serde_json::to_string(&revision) {
	Ok(serialized) => serialized,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SERIALIZATION_FAILED)),
    };

    println!("serialized: {}", serialized);
    let _ = match std::fs::write(format!(".zatsu/revisions/{}.json", revision_number), serialized) {
	Ok(result) => result,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };
    repository.revisions.push(revision_number);
    match repository.save(&PathBuf::from(".zatsu/repository.json")) {
	Ok(_) => (),
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };

    Ok(())
}

fn process_log() -> Result<(), ZatsuError> {
    let repository = match Repository::load(&PathBuf::from(".zatsu/repository.json")) {
	Ok(repository) => repository,
	Err(_) => Repository {
	    revisions: Vec::new(),
	},
    };

    let count = repository.revisions.len();
    for i in (0..count).rev() {
	let revision_number = repository.revisions[i];
	let revision = match Revision::load(&PathBuf::from(format!(".zatsu/revisions/{}.json", revision_number))) {
	    Ok(revision) => revision,
	    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
	};
	println!("Revision {}", revision_number);
	for entry in revision.entries {
	    println!("{}", entry.path);
	}
	println!("");
    }

    Ok(())
}

fn process_get(revision_number: i32, path: &String) -> Result<(), ZatsuError> {
    let repository = match Repository::load(&PathBuf::from(".zatsu/repository.json")) {
	Ok(repository) => repository,
	Err(_) => Repository {
	    revisions: Vec::new(),
	},
    };
    let mut found = false;
    for a_revision_number in repository.revisions {
	if a_revision_number == revision_number {
	    found = true;
	} 
    }
    if !found {
	return Err(ZatsuError::new("main".to_string(), ERROR_REVISION_NOT_FOUND));
    }

    let revision = match Revision::load(&PathBuf::from(format!(".zatsu/revisions/{}.json", revision_number))) {
	Ok(revision) => revision,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_REVISION_FAILED)),
    };
    let mut hash = "".to_string();
    let mut found = false;
    for entry in revision.entries {
	if entry.path == *path {
	    found = true;
	    hash = entry.hash;
	}
    }
    if !found {
	return Err(ZatsuError::new("main".to_string(), ERROR_FILE_NOT_FOUND));
    }

    let values = match fs::read(&PathBuf::from(format!(".zatsu/objects/{}", hash))) {
	Ok(values) => values,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
    };
    let split: Vec<_> = path.split("/").collect();
    let mut file_name = "out.dat".to_string();
    if split.len() >= 1 {
	let original_file_name = split[split.len() - 1].to_string();
	let split: Vec<_> = original_file_name.split(".").collect();
	if split.len() > 1 {
	    file_name = format!("{}-r{}.{}", split[0], revision_number, split[1]);
	}
    }
    match fs::write(&PathBuf::from(file_name), values) {
	Ok(()) => (),
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };

    Ok(())
}

fn process_forget(revision_count: i32) -> Result<(), ZatsuError> {
    let mut repository = match Repository::load(&PathBuf::from(".zatsu/repository.json")) {
	Ok(repository) => repository,
	// TODO: Ensure repository is created when zatsu init.
	Err(_) => Repository {
	    revisions: Vec::new(),
	},
    };
    let current_count = repository.revisions.len() as i32;
    let removed_count = current_count - revision_count;
    if removed_count <= 0 {
	return Ok(());
    }
    let index: usize = removed_count as usize;
    repository.revisions = repository.revisions.drain(index..).collect();
    repository.save(&PathBuf::from(".zatsu/repository.json"))?;
    process_garbage_collection()?;
    
    Ok(())
}

fn process_init() -> Result<(), ZatsuError> {
    match fs::create_dir_all(".zatsu") {
	Ok(()) => (),
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    match fs::create_dir_all(".zatsu/revisions") {
	Ok(()) => (),
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    match fs::create_dir_all(".zatsu/objects") {
	Ok(()) => (),
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    let repository = Repository{
	revisions: Vec::new(),
    };
    match repository.save(&PathBuf::from(".zatsu/repository.json")) {
	Ok(()) => (),
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };

    Ok(())
}

fn process_garbage_collection() -> Result<(), ZatsuError> {
    let repository = match Repository::load(&PathBuf::from(".zatsu/repository.json")) {
	Ok(repository) => repository,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_REPOSITORY_FAILED)),
    };
    let read_dir = match fs::read_dir(".zatsu/revisions") {
	Ok(read_dir) => read_dir,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_READING_DIRECTORY_FAILED)),
    };
    for result in read_dir {
	if result.is_ok() {
	    let entry = result.unwrap();
	    let path = entry.path();
	    let mut found = false;
	    let option = path.file_stem();
	    if option.is_some() {
		let file_stem = option.unwrap().to_string_lossy();
		println!("file_stem: {}", file_stem);
		let result = file_stem.parse();
		if result.is_ok() {
		    let revision_number: i32 = result.unwrap();
		    let option = repository.revisions.iter().find(|&value| *value == revision_number);
		    if option.is_some() {
			found = true;
		    }
		}
	    }

	    if !found {
		match fs::remove_file(path) {
		    Ok(()) => (),
		    Err(_) => (),
		}
	    }
	}
    }

    let read_dir = match fs::read_dir(".zatsu/objects") {
	Ok(read_dir) => read_dir,
	Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_READING_DIRECTORY_FAILED)),
    };
    for result in read_dir {
	if result.is_ok() {
	    let entry = result.unwrap();
	    let path = entry.path();
	    let option = path.file_name();
	    if option.is_some() {
		let hash = option.unwrap().to_string_lossy();
		let mut found = false;
		for revision_number in &repository.revisions {
		    let result = Revision::load(&PathBuf::from(format!(".zatsu/revisions/{}.json", revision_number)));
		    if result.is_ok() {
			let revision = result.unwrap();
			for entry in revision.entries {
			    if entry.hash == hash {
				found = true;
			    }
			}
		    }
		}

		if !found {
		    match fs::remove_file(path) {
			Ok(()) => (),
			Err(_) => (),
		    };
		}
	    }
	}
    }

    Ok(())
}

fn main() -> Result<(), ZatsuError> {
    println!("Hello, world!");

    let arguments: Vec<_> = env::args().collect();
    let count = arguments.len();
    println!("count: {}", count);
    if count > 0 {
	println!("arguments[0]: {}", arguments[0]);

    }
    if count > 1 {
	println!("arguments[1]: {}", arguments[1]);
    }

    let mut subcommand = "commit".to_string();
    if count > 1 {
	subcommand = arguments[1].clone();
    }

    if subcommand == "commit" {
	match process_commit() {
	    Ok(()) => (),
	    Err(error) => return Err(error),
	};
    }
    if subcommand == "log" {
	match process_log() {
	    Ok(()) => (),
	    Err(error) => return Err(error),
	};
    }
    if subcommand == "get" {
	if count > 3 {
	    // TODO: Do not panic is parse failed.
	    let revision_number :i32 = arguments[2].parse().unwrap();
	    let path = arguments[3].clone();
	    match process_get(revision_number, &path) {
		Ok(()) => (),
		Err(error) => return Err(error),
	    };
	}
    }
    if subcommand == "forget" {
	if count > 2 {
	    // TODO: Do not panic is parse failed.
	    let revision_count :i32 = arguments[2].parse().unwrap();
	    match process_forget(revision_count) {
		Ok(()) => (),
		Err(error) => return Err(error),
	    };
	}
    }
    if subcommand == "init" {
	match process_init() {
	    Ok(()) => (),
	    Err(error) => return Err(error),
	};
    }

    Ok(())
}
