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

mod command;
mod commit_command;
mod entry;
mod error;
mod file_path_producer;
mod get_command;
mod log_command;
mod revision;
mod repository;

use flate2::write::ZlibDecoder;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::command::Command;
use crate::commit_command::CommitCommand;
use crate::entry::Entry;
use crate::error::ZatsuError;
use crate::file_path_producer::FilePathProducer;
use crate::log_command::LogCommand;
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

    let mut command = "commit".to_string();
    if count > 1 {
        command = arguments[1].clone();
    }

    if command == "commit" {
	let command = CommitCommand::new();
	match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }
    if command == "log" {
	let command = LogCommand::new();
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }
    if command == "get" {
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
    if command == "forget" {
        if count > 2 {
            // TODO: Do not panic is parse failed.
            let revision_count :i32 = arguments[2].parse().unwrap();
            match process_forget(revision_count) {
                Ok(()) => (),
                Err(error) => return Err(error),
            };
        }
    }
    if command == "init" {
        match process_init() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
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
