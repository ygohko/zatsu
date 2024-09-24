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

use crate::Command;
use crate::error;
use crate::Repository;
use crate::ZatsuError;

pub struct UpgradeCommand {
}

impl Command for UpgradeCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let repository = match Repository::load(".zatsu") {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(error::CODE_LOADING_REPOSITORY_FAILED));
            }
        };
        if repository.version != 1 {
            println!("Error: Repository is already up to date. Do nothing.");
            return Err(ZatsuError::new(error::CODE_GENERAL));
        }

        // Move objects directory.
        match fs::rename(".zatsu/objects", ".zatsu/objects-v1") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_CREATING_DIRECTORY_FAILED)),
        };

        // TODO: Copy objects into new new directory.
        

        // TODO: Update hashes of entries.


        Ok(())
    }
}

impl UpgradeCommand {
    pub fn new() -> Self {
        Self {}
    }
}
