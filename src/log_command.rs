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

use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;

use crate::error;
use crate::Command;
use crate::Entry;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub struct LogCommand {}

impl Command for LogCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
        let repository = match Repository::load(".zatsu/repository.json") {
            Ok(repository) => repository,
            Err(_) => Repository {
                revision_numbers: Vec::new(),
            },
        };

        let count = repository.revision_numbers.len();
        for i in (0..count).rev() {
            let revision_number = repository.revision_numbers[i];
            let revision = match Revision::load(format!(
                ".zatsu/revisions/{:02x}/{}.json",
                revision_number & 0xFF,
                revision_number
            )) {
                Ok(revision) => revision,
                Err(_) => {
                    return Err(ZatsuError::new(
                        error::CODE_LOADING_FILE_FAILED,
                    ))
                }
            };
            let entries = revision.entries;
            let mut previous_entries: Vec<Entry> = Vec::new();
            if i > 0 {
                let previous_revision_number = repository.revision_numbers[i - 1];
                let previous_revision = match Revision::load(format!(
                    ".zatsu/revisions/{:02x}/{}.json",
                    previous_revision_number & 0xFF,
                    previous_revision_number
                )) {
                    Ok(revision) => revision,
                    Err(_) => {
                        return Err(ZatsuError::new(
                            error::CODE_LOADING_FILE_FAILED,
                        ))
                    }
                };
                previous_entries = previous_revision.entries;
            }

            let divided = divided_entries(&entries);
            let previous_divided = divided_entries(&previous_entries);

            // TODO: Apply time zone.
            let commited = match DateTime::from_timestamp_millis(revision.commited) {
                Some(commited) => commited,
                None => Utc::now(),
            };
            println!(
                "Revision {}, commited at {}",
                revision_number,
                commited.format("%Y/%m/%d %H:%M")
            );

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
    pub fn new() -> Self {
        Self {}
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
