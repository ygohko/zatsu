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

use crate::error;
use crate::Command;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub struct ForgetCommand {
    revision_count: i32,
}

impl Command for ForgetCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository = match Repository::load(".zatsu") {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(error::CODE_LOADING_REPOSITORY_FAILED));
            }
        };
        let current_count = repository.revision_numbers.len() as i32;
        let removed_count = current_count - self.revision_count;
        if removed_count <= 0 {
            return Ok(());
        }
        let index: usize = removed_count as usize;
        repository.revision_numbers = repository.revision_numbers.drain(index..).collect();
        repository.save(".zatsu")?;
        process_garbage_collection()?;

        Ok(())
    }
}

impl ForgetCommand {
    pub fn new(revision_count: i32) -> Self {
        Self { revision_count }
    }
}

fn process_garbage_collection() -> Result<(), ZatsuError> {
    let repository = match Repository::load(".zatsu") {
        Ok(repository) => repository,
        Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_REPOSITORY_FAILED)),
    };

    let read_dir = match fs::read_dir(".zatsu/revisions") {
        Ok(read_dir) => read_dir,
        Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
    };
    let mut revision_paths: Vec<PathBuf> = Vec::new();
    for result in read_dir {
        if result.is_ok() {
            let entry = result.unwrap();
            revision_paths.push(entry.path());
        }
    }
    let removed_revision_count = remove_unused_revisions(&repository, &revision_paths)?;

    let read_dir = match fs::read_dir(".zatsu/objects") {
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
    let removed_object_count = remove_unused_objects(&repository, &object_paths)?;

    println!("");
    println!(
        "{} revision(s) and {} object(s) removed.",
        removed_revision_count, removed_object_count
    );

    Ok(())
}

fn remove_unused_revisions(
    repository: &Repository,
    revision_paths: &Vec<PathBuf>,
) -> Result<i32, ZatsuError> {
    let mut removed_revision_count = 0;
    for path in revision_paths {
        let read_dir = match fs::read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
        };
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let mut found = false;

                let option = path.file_stem();
                if option.is_some() {
                    let file_stem = option.unwrap().to_string_lossy();
                    println!("Checking: revision {}", file_stem);

                    let result = file_stem.parse();
                    if result.is_ok() {
                        let revision_number: i32 = result.unwrap();
                        let option = repository
                            .revision_numbers
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
    repository: &Repository,
    object_paths: &Vec<PathBuf>,
) -> Result<i32, ZatsuError> {
    let mut removed_object_count = 0;

    // Mark used objects.
    for revision_number in &repository.revision_numbers {
        println!("Checking: revision {}", revision_number);

        let revision = match Revision::load(format!(
            ".zatsu/revisions/{:02x}/{}.json",
            revision_number & 0xFF,
            revision_number
        )) {
            Ok(revision) => revision,
            Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
        };

        for entry in revision.entries {
            let hash = entry.hash;
            let directory_name = hash[0..2].to_string();
            let mut path = format!(".zatsu/objects/{}/{}", directory_name, hash);
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
            Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
        };

        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let option = path.file_name();
                if option.is_some() {
                    let file_name = option.unwrap().to_string_lossy();
                    if !file_name.ends_with(".mark") {
                        let hash = file_name.clone();
                        println!("Checking: object {}", hash);
                        let directory_name = hash[0..2].to_string();
                        let mark_file_path =
                            format!(".zatsu/objects/{}/{}.mark", directory_name, hash);
                        let marked = Path::new(&mark_file_path).exists();
                        if !marked {
                            println!("Removing: object {}", hash);
                            let path = format!(".zatsu/objects/{}/{}", directory_name, hash);
                            match fs::remove_file(&path) {
                                Ok(_) => (),
                                Err(_) => {
                                    return Err(ZatsuError::new(error::CODE_REMOVING_FILE_FAILED))
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
            Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
        };

        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let option = path.file_name();
                if option.is_some() {
                    let file_name = option.unwrap().to_string_lossy();
                    if file_name.ends_with(".mark") {
                        let directory_name = file_name[0..2].to_string();
                        let path = format!(".zatsu/objects/{}/{}", directory_name, file_name);
                        match fs::remove_file(&path) {
                            Ok(_) => (),
                            Err(_) => {
                                return Err(ZatsuError::new(error::CODE_REMOVING_FILE_FAILED))
                            }
                        };
                    }
                }
            }
        }
    }

    Ok(removed_object_count)
}
