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
use std::path::PathBuf;

use crate::Command;
use crate::commons;
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

        // Create new object direcrory.
        match fs::create_dir(".zatsu/objects") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_CREATING_DIRECTORY_FAILED)),
        };
        
        // Copy objects into new new directory.
        copy_objects();

        // TODO: Update hashes of entries.


        Ok(())
    }
}

impl UpgradeCommand {
    pub fn new() -> Self {
        Self {}
    }
}

fn copy_objects() -> Result<(), ZatsuError> {
    let read_dir = match fs::read_dir(".zatsu/objects-v1") {
        Ok(read_dir) => read_dir,
        Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
    };
    let mut object_paths: Vec<PathBuf> = Vec::new();
    for result in read_dir {
        if result.is_ok() {
            let entry = result.unwrap();
            object_paths.push(entry.path());
        }
    }

    for path in object_paths {
        let mut directory_path = PathBuf::from(".zatsu/objects-v1");
        directory_path = directory_path.join(path);
        let read_dir = match fs::read_dir(directory_path.clone()) {
            Ok(read_dir) => read_dir,
            Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
        };
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let mut file_path = directory_path.clone();
                file_path = file_path.join(entry.path());
                let values = match fs::read(file_path) {
                    Ok(values) => values,
                    Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
                };

                let hash = commons::object_hash(&values, 2);
                commons::save_object(&values, &hash, 2);

                // TODO: Write new object hash.
                // file_path = directory_path.clone();
                // 0let file_name = format!("{}.new", entry.path());
            }
        }
    }

    Ok(())
}
