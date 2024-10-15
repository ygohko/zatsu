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

use crate::commons;
use crate::error;
use crate::Command;
use crate::Entry;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;
pub struct UpgradeCommand {}

impl Command for UpgradeCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let repository = match Repository::load(".zatsu") {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: Repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(error::CODE_LOADING_REPOSITORY_FAILED));
            }
        };
        if repository.version != 1 {
            println!("Error: Repository is already up to date. Do nothing.");
            return Err(ZatsuError::new(error::CODE_GENERAL));
        }

        // Move objects directory.
        println!("Moving current objects...");
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
        copy_objects()?;

        // Update hashes of entries.
        update_entries(&repository.revision_numbers)?;

        // Update version.txt.
        match fs::write(".zatsu/version.txt", "2") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };

        // Remove V1 objects.
        match fs::remove_dir_all(".zatsu/objects-v1") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_REMOVING_DIRECTORY_FAILED)),
        };

        println!("");
        println!("Repository successfully upgraded to V2.");

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
        let directory_path = path;
        let read_dir = match fs::read_dir(directory_path.clone()) {
            Ok(read_dir) => read_dir,
            Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
        };
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let file_path = entry.path();
                println!("Copying: {}", file_path.to_string_lossy());
                let values = match fs::read(file_path.clone()) {
                    Ok(values) => values,
                    Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
                };
                let mut decoder = ZlibDecoder::new(Vec::new());
                match decoder.write_all(&values) {
                    Ok(()) => (),
                    Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
                };
                let decoded = match decoder.finish() {
                    Ok(decoded) => decoded,
                    Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
                };

                let hash = commons::object_hash(&decoded, 2);
                commons::save_object(&decoded, &hash)?;

                // Write new object hash.
                let mut new_file_path = file_path.to_string_lossy().to_mut().clone();
                new_file_path.push_str(".new");
                match fs::write(&new_file_path, hash) {
                    Ok(()) => (),
                    Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
                };
            }
        }
    }

    Ok(())
}

fn update_entries(revision_numbers: &Vec<i32>) -> Result<(), ZatsuError> {
    for revision_number in revision_numbers {
        println!("Updating: Revision {}", revision_number);
        let path = format!(
            ".zatsu/revisions/{:02x}/{}.json",
            (revision_number & 0xFF),
            revision_number
        );
        let mut revision = Revision::load(&path)?;
        let mut new_entries: Vec<Entry> = Vec::new();
        for entry in revision.entries {
            let directory_name = entry.hash[0..2].to_string();
            let path = format!(".zatsu/objects-v1/{}/{}.new", directory_name, entry.hash);
            println!("Updating: {}", entry.path);
            let new_hash = match fs::read_to_string(&path) {
                Ok(new_hash) => new_hash,
                Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
            };

            let new_entry = Entry {
                path: entry.path,
                hash: new_hash,
                permission: entry.permission,
            };
            new_entries.push(new_entry);
        }

        revision.entries = new_entries;
        revision.save(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_creatable() {
        let _command = UpgradeCommand::new();
    }
}
