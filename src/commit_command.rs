/*
 * Copyright (c) 2024 - 2025 Yasuaki Gohko
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
use std::path::PathBuf;

use crate::commons::ToString;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::file_path_producer;
use crate::repository::factory;
use crate::Command;
use crate::Entry;
use crate::FilePathProducer;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub const ERROR_ID: ErrorId = "commit_command";

const ERROR_CODE_READING_META_DATA_FAILED: ErrorCode = 1;
const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 2;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 3;

pub struct CommitCommand {
    pub path: String,
}

impl Command for CommitCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        let mut repository = match factory::load(&repository_path.to_string()) {
            Ok(repository) => repository,
            Err(error) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(error);
            }
        };
        let latest_revision = repository.latest_revision();
        let revision_number = latest_revision + 1;

        let mut producer = FilePathProducer::new(self.path.clone());
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
                let entry = self.process_file(&path, &repository)?;
                revision.entries.push(entry);
            } else {
                let error = result.unwrap_err();
                if error.id == file_path_producer::ERROR_ID
                    && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED
                {
                    done = true;
                }
            }
        }

        repository.save_revision(&revision, revision_number)?;

        println!("");
        println!("Commited as revision {}.", revision_number);
        println!(
            "There are {} revision(s).",
            repository.revision_numbers().len()
        );

        Ok(())
    }
}

impl CommitCommand {
    pub fn new() -> Self {
        Self {
            path: ".".to_string(),
        }
    }

    fn process_file(
        &self,
        path: &str,
        repository: &Box<dyn Repository>,
    ) -> Result<Entry, ZatsuError> {

        // TODO: Return permissions.
        // TODO: Check whether only file path is provided.
        
        let mut file_path = PathBuf::from(&self.path);
        file_path.push(&path);
        let metadata = match fs::metadata(&file_path) {
            Ok(metadata) => metadata,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_META_DATA_FAILED,
                ))
            }
        };
        let mut hex_string = String::new();
        let mut permission: i32 = 0o644;
        if metadata.is_file() {
            let values = match fs::read(&file_path) {
                Ok(values) => values,
                Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
            };
            hex_string = repository.object_hash(&values);

            // TODO: Get the permission.
            
            let directory_name = hex_string[0..2].to_string();
            let mut path = PathBuf::from(&self.path);
            path.push(".zatsu");
            path.push("objects");
            path.push(&directory_name);
            let exists = match path.try_exists() {
                Ok(exists) => exists,
                Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
            };
            if !exists {
                match fs::create_dir(&path) {
                    Ok(()) => (),
                    Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
                };
            }

            path.push(&hex_string);
            let exists = match path.try_exists() {
                Ok(exists) => exists,
                Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
            };
            if !exists {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                match encoder.write_all(&values) {
                    Ok(()) => (),
                    Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
                }
                let compressed = match encoder.finish() {
                    Ok(compressed) => compressed,
                    Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
                };

                match fs::write(path, compressed) {
                    Ok(()) => (),
                    Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
                };
            }
        }

        // TODO: Return error if directory path is provided?

        let entry = Entry {
            path: path.to_string(),
            hash: hex_string,
            permission: 0o644,
        };
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test whether what storing files works.

    use super::*;

    use tempdir::TempDir;

    use crate::InitCommand;
    use crate::commons::ToString;

    #[test]
    fn is_creatable() {
        let _command = CommitCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        let result = command.execute();
        assert!(result.is_ok());

        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        let result = command.execute();
        assert!(result.is_ok());
    }
}
