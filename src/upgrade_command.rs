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

use flate2::write::ZlibDecoder;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::Command;
use crate::Entry;
use crate::Revision;
use crate::ZatsuError;
use crate::commons;
use crate::commons::ToString;
use crate::error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::repository::factory;

pub const ERROR_ID: ErrorId = "upgrade_command";

const ERROR_CODE_READING_DIRECTORY_FAILED: ErrorCode = 1;
const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 2;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 3;
const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 4;
const ERROR_CODE_REMOVING_DIRECTORY_FAILED: ErrorCode = 5;

/// A command to upgrade the repository format.
pub struct UpgradeCommand {
    path: String,
}

impl Command for UpgradeCommand {
    /// Executes the upgrade command.
    ///
    /// This function upgrades the repository from version 1 to version 2.
    /// It involves moving object files, creating new object directories,
    /// copying objects with new hashes, updating entry hashes in revisions,
    /// updating the version file, and removing old object directories.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the upgrade fails.
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        let repository = match factory::load(&repository_path.to_string()) {
            Ok(repository) => repository,
            Err(error) => {
                println!("Error: Repository not found. To create repository, execute zatsu init.");
                return Err(error);
            }
        };
        if repository.version() != 1 {
            println!("Error: Repository is already up to date. Do nothing.");
            return Err(ZatsuError::new(ERROR_ID, error::ERROR_CODE_GENERAL));
        }

        // Move objects directory.
        println!("Moving current objects...");
        let mut from_path = repository_path.clone();
        from_path.push("objects");
        let mut to_path = repository_path.clone();
        to_path.push("objects-v1");
        match fs::rename(&from_path, &to_path) {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_DIRECTORY_FAILED,
                ));
            }
        };

        // Create new object direcrory.
        let mut object_path = repository_path.clone();
        object_path.push("objects");
        match fs::create_dir(&object_path) {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_CREATING_DIRECTORY_FAILED,
                ));
            }
        };

        // Copy objects into new new directory.
        self.copy_objects()?;

        // Update hashes of entries.
        self.update_entries(&repository.revision_numbers())?;

        // Update version.txt.
        let mut path = repository_path.clone();
        path.push("version.txt");
        match fs::write(&path, "2") {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        // Remove V1 objects.
        let mut path = repository_path.clone();
        path.push("objects-v1");
        match fs::remove_dir_all(&path) {
            Ok(()) => (),
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_REMOVING_DIRECTORY_FAILED,
                ));
            }
        };

        println!("");
        println!("Repository successfully upgraded to V2.");

        Ok(())
    }
}

impl UpgradeCommand {
    /// Creates a new `UpgradeCommand` instance.
    pub fn new() -> Self {
        Self {
            path: ".".to_string(),
        }
    }

    /// Copies objects from the old object store (`objects-v1`) to the new one (`objects`).
    ///
    /// During the copy, it recalculates the hash of each object using the new hashing algorithm
    /// (SHA-256) and saves the new hash to a temporary `.new` file.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if copying fails.
    fn copy_objects(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        let mut path = repository_path.clone();
        path.push("objects-v1");
        let read_dir = match fs::read_dir(&path) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_DIRECTORY_FAILED,
                ));
            }
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
                Err(_) => {
                    return Err(ZatsuError::new(
                        ERROR_ID,
                        ERROR_CODE_READING_DIRECTORY_FAILED,
                    ));
                }
            };
            for result in read_dir {
                if result.is_ok() {
                    let entry = result.unwrap();
                    let file_path = entry.path();
                    println!("Copying: {}", file_path.to_string());
                    let values = match fs::read(file_path.clone()) {
                        Ok(values) => values,
                        Err(_) => {
                            return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED));
                        }
                    };
                    let mut decoder = ZlibDecoder::new(Vec::new());
                    match decoder.write_all(&values) {
                        Ok(()) => (),
                        Err(_) => {
                            return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED));
                        }
                    };
                    let decoded = match decoder.finish() {
                        Ok(decoded) => decoded,
                        Err(_) => {
                            return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED));
                        }
                    };

                    let hash = commons::object_hash(&decoded, 2);
                    commons::save_object(&decoded, &hash, &repository_path.to_string())?;

                    // Write new object hash.
                    let mut new_file_path = file_path.to_string();
                    new_file_path.push_str(".new");
                    match fs::write(&new_file_path, hash) {
                        Ok(()) => (),
                        Err(_) => {
                            return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED));
                        }
                    };
                }
            }
        }

        Ok(())
    }

    /// Updates the entry hashes in all revisions to reflect the new hashing algorithm.
    ///
    /// This function reads each revision, updates the `hash` field of each `Entry`
    /// with the new SHA-256 hash (read from the temporary `.new` files),
    /// and then saves the updated revision.
    ///
    /// # Arguments
    ///
    /// * `revision_numbers` - A vector of revision numbers to update.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if updating entries fails.
    fn update_entries(&self, revision_numbers: &Vec<i32>) -> Result<(), ZatsuError> {
        for revision_number in revision_numbers {
            println!("Updating: Revision {}", revision_number);
            let path = format!(
                "{}/.zatsu/revisions/{:02x}/{}.json",
                self.path,
                (revision_number & 0xFF),
                revision_number
            );
            let mut revision = Revision::load(&path)?;
            let mut new_entries: Vec<Entry> = Vec::new();
            for entry in revision.entries {
                let directory_name = entry.hash[0..2].to_string();
                let path = format!(
                    "{}/.zatsu/objects-v1/{}/{}.new",
                    self.path, directory_name, entry.hash
                );
                println!("Updating: {}", entry.path);
                let new_hash = match fs::read_to_string(&path) {
                    Ok(new_hash) => new_hash,
                    Err(_) => {
                        return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED));
                    }
                };

                let new_entry = Entry {
                    path: entry.path,
                    hash: new_hash,
                    permission: entry.permission,
                };
                new_entries.push(new_entry);
            }

            revision.entries = new_entries;
            revision.save(&path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;
    use tempdir::TempDir;

    use crate::CommitCommand;
    use crate::InitCommand;
    use crate::commons::ToString;

    #[test]
    fn is_creatable() {
        let _command = UpgradeCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut path = temp_path.clone();
        path.push("a.txt");
        fs::write(&path, "Hello, World!").unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = UpgradeCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
    }
}
