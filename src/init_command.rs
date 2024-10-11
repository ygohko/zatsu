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

use crate::error;
use crate::Command;
use crate::Repository;
use crate::ZatsuError;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub struct InitCommand {
    version: i32,
}

impl Command for InitCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        if Path::new(".zatsu").exists() {
            println!("Error: This directory already has a repository.");
            return Err(ZatsuError::new(error::CODE_CREATING_DIRECTORY_FAILED));
        }

        match fs::create_dir_all(".zatsu") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_CREATING_REPOSITORY_FAILED)),
        };
        match fs::write(".zatsu/version.txt", self.version.to_string()) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_CREATING_REPOSITORY_FAILED)),
        };
        match fs::create_dir_all(".zatsu/revisions") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_CREATING_REPOSITORY_FAILED)),
        };
        match fs::create_dir_all(".zatsu/objects") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_CREATING_REPOSITORY_FAILED)),
        };
        let repository = Repository {
            revision_numbers: Vec::new(),
            version: self.version,
        };
        match repository.save(&PathBuf::from(".zatsu")) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };

        println!("Repository initialized.");

        Ok(())
    }
}

impl InitCommand {
    pub fn new(version: i32) -> Self {
        Self { version }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    #[test]
    fn is_creatable() {
        let _command = InitCommand::new(1);
        let _command = InitCommand::new(2);
    }

    #[test]
    fn is_executable() {
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(1);
        let result = command.execute();
        assert!(result.is_ok());
        let exists = Path::new(".zatsu").exists();
        assert_eq!(true, exists);
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }
}
