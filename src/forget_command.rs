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

use crate::commons::ToString;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::repository::factory;
use crate::Command;
use crate::Repository;
use crate::ZatsuError;

pub const ERROR_ID: ErrorId = "forget_command";

const ERROR_CODE_READING_DIRECTORY_FAILED: ErrorCode = 1;
const ERROR_CODE_LOADING_REPOSITORY_FAILED: ErrorCode = 2;
const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 3;
const ERROR_CODE_REMOVING_FILE_FAILED: ErrorCode = 4;

pub struct ForgetCommand {
    revision_count: i32,
    path: String,
}

impl Command for ForgetCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        let mut repository = match factory::load(&repository_path) {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_LOADING_REPOSITORY_FAILED,
                ));
            }
        };
        let mut revision_numbers = repository.revision_numbers();
        let current_count = revision_numbers.len() as i32;
        let removed_count = current_count - self.revision_count;
        if removed_count <= 0 {
            return Ok(());
        }
        let index: usize = removed_count as usize;
        revision_numbers = revision_numbers.drain(index..).collect();
        repository.set_revision_numbers(&revision_numbers);
        repository.save(&repository_path)?;
        self.process_garbage_collection()?;

        Ok(())
    }
}

impl ForgetCommand {
    pub fn new(revision_count: i32) -> Self {
        Self {
            revision_count,
            path: ".".to_string(),
        }
    }

    fn process_garbage_collection(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        let repository = match factory::load(&repository_path) {
            Ok(repository) => repository,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_LOADING_REPOSITORY_FAILED,
                ))
            }
        };

        let mut revisions_path = repository_path.clone();
        revisions_path.push("revisions");
        let read_dir = match fs::read_dir(&revisions_path) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_DIRECTORY_FAILED,
                ))
            }
        };
        let mut revision_paths: Vec<PathBuf> = Vec::new();
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                revision_paths.push(entry.path());
            }
        }
        let removed_revision_count = remove_unused_revisions(&repository, &revision_paths)?;

        let mut objects_path = repository_path.clone();
        objects_path.push("objects");
        let read_dir = match fs::read_dir(&objects_path) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_DIRECTORY_FAILED,
                ))
            }
        };
        let mut object_paths: Vec<PathBuf> = Vec::new();
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                object_paths.push(entry.path());
            }
        }
        let removed_object_count = remove_unused_objects(&repository, &object_paths)?;

        println!("");
        println!(
            "{} revision(s) and {} object(s) removed.",
            removed_revision_count, removed_object_count
        );

        Ok(())
    }
}

fn remove_unused_revisions(
    repository: &Box<dyn Repository>,
    revision_paths: &Vec<PathBuf>,
) -> Result<i32, ZatsuError> {
    let mut removed_revision_count = 0;
    for path in revision_paths {
        let read_dir = match fs::read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_DIRECTORY_FAILED,
                ))
            }
        };
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let mut found = false;

                let option = path.file_stem();
                if option.is_some() {
                    let file_stem = option.unwrap().to_string();
                    println!("Checking: revision {}", file_stem);

                    let result = file_stem.parse();
                    if result.is_ok() {
                        let revision_number: i32 = result.unwrap();
                        let revision_numbers = repository.revision_numbers();
                        let option = revision_numbers
                            .iter()
                            .find(|&value| *value == revision_number);
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
                    removed_revision_count += 1;
                }
            }
        }
    }

    Ok(removed_revision_count)
}

fn remove_unused_objects(
    repository: &Box<dyn Repository>,
    object_paths: &Vec<PathBuf>,
) -> Result<i32, ZatsuError> {
    let mut removed_object_count = 0;

    // Mark used objects.
    for revision_number in &repository.revision_numbers() {
        println!("Checking: revision {}", revision_number);

        let revision = match repository.load_revision(*revision_number) {
            Ok(revision) => revision,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };

        for entry in revision.entries {
            let hash = entry.hash;
            let directory_name = hash[0..2].to_string();
            let mut path = format!("{}/objects/{}/{}", repository.path(), directory_name, hash);
            let exists = Path::new(&path).exists();
            if exists {
                path += ".mark";
                let exists = Path::new(&path).exists();
                if !exists {
                    let _ = fs::write(&path, b"marked");
                }
            }
        }
    }

    // Remove objects that are not marked.
    for path in object_paths {
        let read_dir = match fs::read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_DIRECTORY_FAILED,
                ))
            }
        };

        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let option = path.file_name();
                if option.is_some() {
                    let file_name = option.unwrap().to_string();
                    if !file_name.ends_with(".mark") {
                        let hash = file_name.clone();
                        println!("Checking: object {}", hash);
                        let directory_name = hash[0..2].to_string();
                        let mark_file_path =
                            format!("{}/objects/{}/{}.mark", repository.path(), directory_name, hash);
                        let marked = Path::new(&mark_file_path).exists();
                        if !marked {
                            println!("Removing: object {}", hash);
                            let path = format!("{}/objects/{}/{}", repository.path(), directory_name, hash);
                            match fs::remove_file(&path) {
                                Ok(_) => (),
                                Err(_) => {
                                    return Err(ZatsuError::new(
                                        ERROR_ID,
                                        ERROR_CODE_REMOVING_FILE_FAILED,
                                    ))
                                }
                            };
                            removed_object_count += 1;
                        }
                    }
                }
            }
        }
    }

    // Remove mark files.
    println!("Cleaning...");
    for path in object_paths {
        let read_dir = match fs::read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_READING_DIRECTORY_FAILED,
                ))
            }
        };

        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let option = path.file_name();
                if option.is_some() {
                    let file_name = option.unwrap().to_string();
                    if file_name.ends_with(".mark") {
                        let directory_name = file_name[0..2].to_string();
                        let path = format!("{}/objects/{}/{}", repository.path(), directory_name, file_name);
                        match fs::remove_file(&path) {
                            Ok(_) => (),
                            Err(_) => {
                                return Err(ZatsuError::new(
                                    ERROR_ID,
                                    ERROR_CODE_REMOVING_FILE_FAILED,
                                ))
                            }
                        };
                    }
                }
            }
        }
    }

    Ok(removed_object_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use tempdir::TempDir;

    use crate::commons::ToString;
    use crate::CommitCommand;
    use crate::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = ForgetCommand::new(1);
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        fs::write("a.txt", "Hello, World!").unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = ForgetCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();

        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        fs::write("a.txt", "Hello, World!").unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = ForgetCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap()
    }
}
