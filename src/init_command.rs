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

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::repository::factory;
use crate::Command;
use crate::ZatsuError;

pub const ERROR_ID: ErrorId = "init_command";

const ERROR_CODE_CREATING_REPOSITORY_FAILED: ErrorCode = 1;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 2;
const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 3;

pub struct InitCommand {
    version: i32,
    pub path: String,
}

impl Command for InitCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        if Path::new(&repository_path).exists() {
            println!("Error: This directory already has a repository.");
            return Err(ZatsuError::new(
                ERROR_ID,
                ERROR_CODE_CREATING_DIRECTORY_FAILED,
            ));
        }

        match fs::create_dir_all(&repository_path) {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_REPOSITORY_FAILED,
                ))
            }
        };
        let mut path = repository_path.clone();
        path.push("version.txt");
        match fs::write(&path, self.version.to_string()) {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_REPOSITORY_FAILED,
                ))
            }
        };
        let mut path = repository_path.clone();
        path.push("revisions");
        match fs::create_dir_all(&path) {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_REPOSITORY_FAILED,
                ))
            }
        };
        match fs::create_dir_all(".zatsu/objects") {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_REPOSITORY_FAILED,
                ))
            }
        };
        let repository = factory::new(self.version);
        match repository.save(&repository_path) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        println!("Repository initialized.");

        Ok(())
    }
}

impl InitCommand {
    pub fn new(version: i32) -> Self {
        Self {
            version,
            path: ".".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempdir::TempDir;
    
    #[test]
    fn is_creatable() {
        let _command = InitCommand::new(1);
        let _command = InitCommand::new(2);
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string_lossy().to_string();
        let result = command.execute();
        assert!(result.is_ok());
        let mut repository_path = temp_path.clone();
        repository_path.push(".zatsu");
        let exists = Path::new(&repository_path).exists();
        assert_eq!(true, exists);
        
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string_lossy().to_string();
        let result = command.execute();
        assert!(result.is_ok());
        let mut repository_path = temp_path.clone();
        repository_path.push(".zatsu");
        let exists = Path::new(&repository_path).exists();
        assert_eq!(true, exists);
    }
}
