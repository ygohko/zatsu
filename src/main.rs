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

use chrono::DateTime;
use chrono::Utc;
use flate2::Compression;
use flate2::write::ZlibDecoder;
use flate2::write::ZlibEncoder;
use hex_string::HexString;
use sha1::Digest;
use sha1::Sha1;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::entry::Entry;
use crate::error::ZatsuError;
use crate::file_path_producer::FilePathProducer;
use crate::revision::Revision;
use crate::repository::Repository;

#[allow(dead_code)]
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
const ERROR_PRODUCING_FINISHED: i32 = 10;

trait Command {
    fn execute(&self) -> Result<(), ZatsuError>;
}

struct CommitCommand {
}

impl Command for CommitCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
	let mut repository = match Repository::load(".zatsu/repository.json") {
            Ok(repository) => repository,
            Err(_) => Repository {
		revision_numbers: Vec::new(),
            },
	};
	let latest_revision = repository.latest_revision();
	let revision_number = latest_revision + 1;

	let mut producer = FilePathProducer::new(".".to_string());
	let now = Utc::now();
	let mut revision = Revision {
            commited: now.timestamp_millis(),
            entries: Vec::new(),
            description: "".to_string(),
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
                    permission: 0o644,
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

	let path = format!(".zatsu/revisions/{:02x}", revision_number & 0xFF).to_string();
	let a_path = Path::new(&path);
	let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};
	if !exists {
            match fs::create_dir(&path) {
		Ok(()) => (),
		Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };
	}
	match revision.save(format!("{}/{}.json", &path, revision_number)) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};
	repository.revision_numbers.push(revision_number);
	match repository.save(&PathBuf::from(".zatsu/repository.json")) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};

	Ok(())	
    }
}

impl CommitCommand {
    fn new() -> Self {
	Self{}
    }
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
	let command = CommitCommand::new();
	match command.execute() {
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

fn process_log() -> Result<(), ZatsuError> {
    let repository = match Repository::load(".zatsu/repository.json") {
        Ok(repository) => repository,
        Err(_) => Repository {
            revision_numbers: Vec::new(),
        },
    };

    let count = repository.revision_numbers.len();
    for i in (0..count).rev() {
        let revision_number = repository.revision_numbers[i];
        let revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", revision_number & 0xFF, revision_number)) {
            Ok(revision) => revision,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
        };
        let entries = revision.entries;
        let mut previous_entries: Vec<Entry> = Vec::new();
        if i > 0 {
            let previous_revision_number = repository.revision_numbers[i - 1];
            let previous_revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", previous_revision_number & 0xFF, previous_revision_number)) {
                Ok(revision) => revision,
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
            };
            previous_entries = previous_revision.entries;
        }

        // TODO: Apply time zone.
        let commited = match DateTime::from_timestamp_millis(revision.commited) {
            Some(commited) => commited,
            None => Utc::now(),
        };
        println!("Revision {}, commited at {}", revision_number, commited.format("%Y/%m/%d %H:%M"));

        let mut changes: Vec<String> = Vec::new();
        for entry in &entries {
            let mut found = false;
            let previous_hash = match find_hash(&previous_entries, &entry.path) {
                Some(hash) => {
                    found = true;
                    hash
                },
                None => String::new(),
            };
            if found {
                if previous_hash != entry.hash {
                    changes.push(format!("M {}", entry.path));
                }
            }
            else {
                changes.push(format!("A {}", entry.path));
            }
        }
        for entry in previous_entries {
            let mut found = false;
            match find_hash(&entries, &entry.path) {
                Some(_) => {
                    found = true;
                    ()
                },
                None => (),
            }
            if !found {
                changes.push(format!("D {}", entry.path));
            }
        }

        for change in changes {
            println!("{}", change);
        }
        println!("");
    }

    Ok(())
}

fn process_get(revision_number: i32, path: &String) -> Result<(), ZatsuError> {
    let repository = match Repository::load(".zatsu/repository.json") {
        Ok(repository) => repository,
        Err(_) => Repository {
            revision_numbers: Vec::new(),
        },
    };
    let mut found = false;
    for a_revision_number in repository.revision_numbers {
        if a_revision_number == revision_number {
            found = true;
        } 
    }
    if !found {
        return Err(ZatsuError::new("main".to_string(), ERROR_REVISION_NOT_FOUND));
    }

    let revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", revision_number & 0xFF, revision_number)) {
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

    let directory_name = hash[0..2].to_string();
    let values = match fs::read(&PathBuf::from(format!(".zatsu/objects/{}/{}", directory_name, hash))) {
        Ok(values) => values,
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
    };
    let mut decoder = ZlibDecoder::new(Vec::new());
    match decoder.write_all(&values) {
        Ok(()) => (),
        Err(_) =>return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
    };
    let decoded = match decoder.finish() {
        Ok(decoded) => decoded,
        Err(_) =>return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
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
    match fs::write(&PathBuf::from(file_name), decoded) {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };

    Ok(())
}

fn process_forget(revision_count: i32) -> Result<(), ZatsuError> {
    let mut repository = match Repository::load(".zatsu/repository.json") {
        Ok(repository) => repository,
        // TODO: Ensure repository is created when zatsu init.
        Err(_) => Repository {
            revision_numbers: Vec::new(),
        },
    };
    let current_count = repository.revision_numbers.len() as i32;
    let removed_count = current_count - revision_count;
    if removed_count <= 0 {
        return Ok(());
    }
    let index: usize = removed_count as usize;
    repository.revision_numbers = repository.revision_numbers.drain(index..).collect();
    repository.save(".zatsu/repository.json")?;
    process_garbage_collection()?;

    Ok(())
}

fn process_init() -> Result<(), ZatsuError> {
    match fs::create_dir_all(".zatsu") {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    match fs::write(".zatsu/version.txt", "1") {
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
    let repository = Repository {
        revision_numbers: Vec::new(),
    };
    match repository.save(&PathBuf::from(".zatsu/repository.json")) {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };

    Ok(())
}

fn process_garbage_collection() -> Result<(), ZatsuError> {
    let repository = match Repository::load(".zatsu/repository.json") {
        Ok(repository) => repository,
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_REPOSITORY_FAILED)),
    };

    let read_dir = match fs::read_dir(".zatsu/revisions") {
        Ok(read_dir) => read_dir,
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_READING_DIRECTORY_FAILED)),
    };
    let mut revision_paths: Vec<PathBuf> = Vec::new();
    for result in read_dir {
        if result.is_ok() {
            let entry = result.unwrap();
            revision_paths.push(entry.path());
        }
    }

    for path in revision_paths {
        let read_dir = match fs::read_dir(path) {
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
                        let option = repository.revision_numbers.iter().find(|&value| *value == revision_number);
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
    }

    let read_dir = match fs::read_dir(".zatsu/objects") {
        Ok(read_dir) => read_dir,
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_READING_DIRECTORY_FAILED)),
    }; 
    let mut object_paths: Vec<PathBuf> = Vec::new();
    for result in read_dir {
        if result.is_ok() {
            let entry = result.unwrap();
            object_paths.push(entry.path());
        }
    }

    for path in object_paths {
        let read_dir = match fs::read_dir(path) {
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
                    for revision_number in &repository.revision_numbers {
                        let result = Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", revision_number & 0xFF, revision_number));
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
    }

    Ok(())
}

fn process_file(path: impl AsRef<Path>) -> Result<String, ZatsuError> {
    let metadata = match fs::metadata(&path) {
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

        let directory_name = hex_string[0..2].to_string();
        let path = format!(".zatsu/objects/{}", directory_name).to_string();
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
        };
        if !exists {
            match fs::create_dir(&path) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };
        }

        let path = format!("{}/{}", &path, hex_string);
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
        };
        if !exists {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            match encoder.write_all(&values) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            }
            let compressed = match encoder.finish() {
                Ok(compressed) => compressed,
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };

            match fs::write(path, compressed) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };
        }
    } else {
        println!("This is not file.");
    }

    Ok(hex_string)
}

fn find_hash(entries:&Vec<Entry>, path: &String) -> Option<String> {
    for entry in entries {
        if entry.path == *path {
            return Some(entry.hash.clone());
        }
    }

    None
}
