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

use flate2::write::ZlibDecoder;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::repository::factory;
use crate::Command;
use crate::Revision;
use crate::ZatsuError;

pub const ERROR_ID: ErrorId = "get_command";

const ERROR_CODE_LOADING_REPOSITORY_FAILED: ErrorCode = 1;
const ERROR_CODE_REVISION_NOT_FOUND: ErrorCode = 2;
const ERROR_CODE_LOADING_REVISION_FAILED: ErrorCode = 3;
const ERROR_CODE_FILE_NOT_FOUND: ErrorCode = 4;
const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 5;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 6;
const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 7;

pub struct GetCommand {
    revision_number: i32,
    getting_path: String,
    path: String,
}

impl Command for GetCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");

        println!("repository_path: {}", repository_path.to_string_lossy());
        
        let repository = match factory::load(&repository_path) {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_LOADING_REPOSITORY_FAILED,
                ));
            }
        };

        println!("len: {}", repository.revision_numbers().len());
        
        let mut found = false;
        for a_revision_number in repository.revision_numbers() {

            println!("a_revision_number: {}", a_revision_number);
            
            if a_revision_number == self.revision_number {
                found = true;
            }
        }
        if !found {
            return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_REVISION_NOT_FOUND));
        }
        let revision = match repository.load_revision(self.revision_number) {
            Ok(revision) => revision,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_LOADING_REVISION_FAILED,
                ))
            }
        };
        let mut hash = "".to_string();
        let mut file_found = false;
        let mut directory_found = false;

        println!("revision.entries.len(): {}", revision.entries.len());

        for entry in &revision.entries {

            println!("entry.path: {}, self.getting_path: {}", entry.path, self.getting_path);

            if entry.path == *self.getting_path {
                file_found = true;
                hash = entry.hash.clone();
            }

            if entry.path.contains("/") {
                if let Some(index) = entry.path.find(&self.getting_path) {
                    if index == 0 && self.getting_path.len() <= entry.path.len() - 2 {
                        directory_found = true;
                    }
                }
            }
        }

        if file_found {
            return self.save_file(&hash);
        }
        if directory_found {
            return self.save_directory(&revision);
        }

        Err(ZatsuError::new(ERROR_ID, ERROR_CODE_FILE_NOT_FOUND))
    }
}

impl GetCommand {
    pub fn new(revision_number: i32, path: &str) -> Self {
        Self {
            revision_number,
            getting_path: path.to_string(),
            path: ".".to_string(),
        }
    }

    fn save_file(&self, hash: &str) -> Result<(), ZatsuError> {
        println!("Processing: {}", self.getting_path);

        let directory_name = hash[0..2].to_string();
        let values = match fs::read(&PathBuf::from(format!(
            ".zatsu/objects/{}/{}",
            directory_name, hash
        ))) {
            Ok(values) => values,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        let mut decoder = ZlibDecoder::new(Vec::new());
        match decoder.write_all(&values) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        let decoded = match decoder.finish() {
            Ok(decoded) => decoded,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        let split: Vec<_> = self.getting_path.split("/").collect();
        let mut file_name = "out.dat".to_string();
        if split.len() >= 1 {
            let original_file_name = split[split.len() - 1].to_string();
            let split: Vec<_> = original_file_name.split(".").collect();
            if split.len() > 1 {
                file_name = format!("{}-r{}.{}", split[0], self.revision_number, split[1]);
            }
        }
        match fs::write(&PathBuf::from(file_name), decoded) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }

    fn save_directory(&self, revision: &Revision) -> Result<(), ZatsuError> {
        // Make root directory.
        let root_path: String;
        let split: Vec<_> = self.getting_path.split("/").collect();
        let count = split.len();
        if count >= 1 {
            root_path = format!("{}-r{}", split[count - 1], self.revision_number);
        } else {
            root_path = format!("{}-r{}", self.getting_path, self.revision_number);
        }
        match fs::create_dir(&root_path) {
            Ok(_) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_DIRECTORY_FAILED,
                ))
            }
        };

        let mut hash: String;
        for entry in &revision.entries {
            if let Some(_) = entry.path.find(&self.getting_path) {
                println!("Processing: {}", entry.path);

                hash = entry.hash.clone();
                let directory_name = hash[0..2].to_string();
                let values = match fs::read(&PathBuf::from(format!(
                    ".zatsu/objects/{}/{}",
                    directory_name, hash
                ))) {
                    Ok(values) => values,
                    Err(_) => {
                        return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED))
                    }
                };
                let mut decoder = ZlibDecoder::new(Vec::new());
                match decoder.write_all(&values) {
                    Ok(()) => (),
                    Err(_) => {
                        return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED))
                    }
                };
                let decoded = match decoder.finish() {
                    Ok(decoded) => decoded,
                    Err(_) => {
                        return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED))
                    }
                };

                let split: Vec<_> = entry.path.split("/").collect();
                let mut file_name = "out.dat".to_string();
                let count = split.len();
                if count >= 1 {
                    file_name = split[count - 1].to_string();
                }

                // Make sub directries.
                let mut path = root_path.clone();
                if count >= 3 {
                    for i in 0..(count - 2) {
                        path += &("/".to_string() + &split[i + 1]);
                    }
                }
                match fs::create_dir_all(&path) {
                    Ok(_) => (),
                    Err(_) => {
                        return Err(ZatsuError::new(
                            ERROR_ID,
                            ERROR_CODE_CREATING_DIRECTORY_FAILED,
                        ))
                    }
                };

                path += &("/".to_string() + &file_name);
                match fs::write(path, decoded) {
                    Ok(()) => (),
                    Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
                };
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;
    use tempdir::TempDir;

    use crate::CommitCommand;
    use crate::InitCommand;
    use crate::commons::ToString;

    #[test]
    fn is_creatable() {
        let _command = GetCommand::new(1, "a.txt");
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        // let temp_path = PathBuf::from("tmp");
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        fs::write("a.txt", "Hello, World!").unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = GetCommand::new(1, "a.txt");
        command.path = temp_path.to_string();
        let result = command.execute();
        result.unwrap();
        // assert!(result.is_ok());
        let string = fs::read_to_string("a-r1.txt").unwrap();
        assert_eq!("Hello, World!", string);

        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(2);
        command.execute().unwrap();
        fs::write("a.txt", "Hello, World!").unwrap();
        let command = CommitCommand::new();
        command.execute().unwrap();
        let command = GetCommand::new(1, "a.txt");
        let result = command.execute();
        assert!(result.is_ok());
        let string = fs::read_to_string("a-r1.txt").unwrap();
        assert_eq!("Hello, World!", string);
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }
}
