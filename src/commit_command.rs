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

use chrono::Utc;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::error;
use crate::repository::factory;
use crate::Command;
use crate::Entry;
use crate::FilePathProducer;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub struct CommitCommand {}

impl Command for CommitCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository = match factory::load(".zatsu") {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(error::CODE_LOADING_REPOSITORY_FAILED));
            }
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
                println!("Processing: {}", path);
                let hash = match process_file(&PathBuf::from(path.clone()), &repository) {
                    Ok(hash) => hash,
                    Err(error) => return Err(error),
                };
                let entry = Entry {
                    path: path,
                    hash: hash,
                    permission: 0o644,
                };
                revision.entries.push(entry);
            } else {
                let error = result.unwrap_err();
                if error.code == error::CODE_PRODUCING_FINISHED {
                    done = true;
                }
            }
        }

        let path = format!(".zatsu/revisions/{:02x}", revision_number & 0xFF).to_string();
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };
        if !exists {
            match fs::create_dir(&path) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
            };
        }
        match revision.save(format!("{}/{}.json", &path, revision_number)) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };
        let mut revision_numbers = repository.revision_numbers();
        revision_numbers.push(revision_number);
        repository.set_revision_numbers(&revision_numbers);
        match repository.save(&Path::new(".zatsu")) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };

        println!("");
        println!("Commited as revision {}.", revision_number);
        println!("There are {} revision(s).", revision_numbers.len());

        Ok(())
    }
}

impl CommitCommand {
    pub fn new() -> Self {
        Self {}
    }
}

fn process_file(path: impl AsRef<Path>, repository: &Box<dyn Repository>) -> Result<String, ZatsuError> {
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => return Err(ZatsuError::new(error::CODE_READING_META_DATA_FAILED)),
    };
    let mut hex_string = String::new();
    if metadata.is_file() {
        let values = match fs::read(path) {
            Ok(values) => values,
            Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
        };
        hex_string = repository.object_hash(&values);

        let directory_name = hex_string[0..2].to_string();
        let path = format!(".zatsu/objects/{}", directory_name).to_string();
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };
        if !exists {
            match fs::create_dir(&path) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
            };
        }

        let path = format!("{}/{}", &path, hex_string);
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };
        if !exists {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            match encoder.write_all(&values) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
            }
            let compressed = match encoder.finish() {
                Ok(compressed) => compressed,
                Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
            };

            match fs::write(path, compressed) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
            };
        }
    }

    Ok(hex_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    use crate::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = CommitCommand::new();
    }

    #[test]
    fn is_executable() {
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(1);
        command.execute().unwrap();
        let command = CommitCommand::new();
        let result = command.execute();
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();

        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(2);
        command.execute().unwrap();
        let command = CommitCommand::new();
        let result = command.execute();
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }
}
