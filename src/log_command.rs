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

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::Command;
use crate::Entry;
use crate::ZatsuError;
use crate::commons::ToString;
use crate::error::ErrorId;
use crate::repository::factory;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "log_command";

/// A command to display the revision history of the repository.
pub struct LogCommand {
    path: String,
}

impl Command for LogCommand {
    /// Executes the log command.
    ///
    /// This function iterates through the revisions in the repository, displays
    /// information about each revision (number, commit time, description), and
    /// lists the changes (added, modified, deleted files) between consecutive revisions.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the operation fails.
    fn execute(&self) -> Result<(), ZatsuError> {
        let mut repository_path = PathBuf::from(&self.path);
        repository_path.push(".zatsu");
        let repository = match factory::load(&repository_path.to_string()) {
            Ok(repository) => repository,
            Err(error) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(error);
            }
        };

        let utc_offset = Local::now().offset().local_minus_utc() as i64;
        let count = repository.revision_numbers().len();
        for i in (0..count).rev() {
            let revision_number = repository.revision_numbers()[i];
            let revision = repository.load_revision(revision_number)?;
            let entries = revision.entries;
            let mut previous_entries: Vec<Entry> = Vec::new();
            if i > 0 {
                let previous_revision_number = repository.revision_numbers()[i - 1];
                let previous_revision = repository.load_revision(previous_revision_number)?;
                previous_entries = previous_revision.entries;
            }

            let divided = divided_entries(&entries);
            let previous_divided = divided_entries(&previous_entries);

            let milliseconds = revision.commited + utc_offset * 1000;
            let commited = match DateTime::from_timestamp_millis(milliseconds) {
                Some(commited) => commited,
                None => Utc::now(),
            };
            println!(
                "Revision {}, commited at {}",
                revision_number,
                commited.format("%Y/%m/%d %H:%M")
            );
            if !revision.description.is_empty() {
                println!("{}", revision.description);
            }

            let mut changes: Vec<String> = Vec::new();

            let keys = divided.keys();
            for key in keys {
                if !previous_divided.contains_key(key) {
                    // All entries are appended.
                    let entries = &divided[&key];
                    for entry in entries {
                        changes.push(format!("A {}", entry.path));
                    }
                } else {
                    // Compare entries and add chaned.
                    let entries = &divided[&key];
                    let previous_entries = &previous_divided[&key];
                    update_changes(&mut changes, &entries, &previous_entries);
                }
            }
            let keys = previous_divided.keys();
            for key in keys {
                if !divided.contains_key(key) {
                    // All entries are deleted.
                    let entries = &previous_divided[&key];
                    for entry in entries {
                        changes.push(format!("D {}", entry.path));
                    }
                }
            }

            for change in changes {
                println!("{}", change);
            }
            println!("");
        }

        Ok(())
    }
}

impl LogCommand {
    /// Creates a new `LogCommand` instance.
    pub fn new() -> Self {
        Self {
            path: ".".to_string(),
        }
    }
}

fn find_hash(entries: &Vec<Entry>, path: &String) -> Option<String> {
    for entry in entries {
        if entry.path == *path {
            return Some(entry.hash.clone());
        }
    }

    None
}

/// Divides entries into a HashMap based on the first character of their paths.
///
/// This is an optimization to reduce the number of comparisons needed when checking for changes.
///
/// # Arguments
///
/// * `entries` - A vector of `Entry` structs.
///
/// # Returns
///
/// A `HashMap` where keys are the first characters of paths and values are vectors of `Entry`.
fn divided_entries(entries: &Vec<Entry>) -> HashMap<char, Vec<Entry>> {
    let mut result: HashMap<char, Vec<Entry>> = HashMap::new();

    for entry in entries {
        let key: char;
        if entry.path.len() > 0 {
            key = entry.path.chars().nth(0).unwrap();
        } else {
            key = char::from_u32(0).unwrap();
        }

        if !result.contains_key(&key) {
            result.insert(key, Vec::new());
        }
        let entries = result.get_mut(&key).unwrap();
        entries.push(entry.clone());
    }

    result
}

/// Updates a list of changes by comparing current entries with previous entries.
///
/// This function identifies added, modified, and deleted files between two sets of entries.
///
/// # Arguments
///
/// * `changes` - A mutable vector of strings to which change descriptions will be added.
/// * `entries` - The current set of `Entry` structs.
/// * `previous_entries` - The previous set of `Entry` structs.
fn update_changes(changes: &mut Vec<String>, entries: &Vec<Entry>, previous_entries: &Vec<Entry>) {
    for entry in entries {
        let mut found = false;
        let previous_hash = match find_hash(&previous_entries, &entry.path) {
            Some(hash) => {
                found = true;
                hash
            }
            None => String::new(),
        };
        if found {
            if previous_hash != entry.hash {
                changes.push(format!("M {}", entry.path));
            }
        } else {
            changes.push(format!("A {}", entry.path));
        }
    }
    for entry in previous_entries {
        let mut found = false;
        match find_hash(&entries, &entry.path) {
            Some(_) => {
                found = true;
                ()
            }
            None => (),
        }
        if !found {
            changes.push(format!("D {}", entry.path));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempdir::TempDir;

    use crate::InitCommand;
    use crate::commons::ToString;

    #[test]
    fn is_creatable() {
        let _command = LogCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = LogCommand::new();
        command.path = temp_path.to_string();
        let result = command.execute();
        assert!(result.is_ok());

        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = LogCommand::new();
        command.path = temp_path.to_string();
        let result = command.execute();
        assert!(result.is_ok());
    }
}
