use chrono::DateTime;
use chrono::Utc;

use crate::Command;
use crate::Entry;
use crate::ERROR_LOADING_FILE_FAILED;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub struct LogCommand {
}

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
            let revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", revision_number & 0xFF, revision_number)) {
		Ok(revision) => revision,
		Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
            };
            let entries = revision.entries;
            let mut previous_entries: Vec<Entry> = Vec::new();
            if i > 0 {
		let previous_revision_number = repository.revision_numbers[i - 1];
		let previous_revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", previous_revision_number & 0xFF, previous_revision_number)) {
                    Ok(revision) => revision,
                    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
		};
		previous_entries = previous_revision.entries;
            }

            // TODO: Apply time zone.
            let commited = match DateTime::from_timestamp_millis(revision.commited) {
		Some(commited) => commited,
		None => Utc::now(),
            };
            println!("Revision {}, commited at {}", revision_number, commited.format("%Y/%m/%d %H:%M"));

            let mut changes: Vec<String> = Vec::new();
            for entry in &entries {
		let mut found = false;
		let previous_hash = match find_hash(&previous_entries, &entry.path) {
                    Some(hash) => {
			found = true;
			hash
                    },
                    None => String::new(),
		};
		if found {
                    if previous_hash != entry.hash {
			changes.push(format!("M {}", entry.path));
                    }
		}
		else {
                    changes.push(format!("A {}", entry.path));
		}
            }
            for entry in previous_entries {
		let mut found = false;
		match find_hash(&entries, &entry.path) {
                    Some(_) => {
			found = true;
			()
                    },
                    None => (),
		}
		if !found {
                    changes.push(format!("D {}", entry.path));
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

fn find_hash(entries:&Vec<Entry>, path: &String) -> Option<String> {
    for entry in entries {
        if entry.path == *path {
            return Some(entry.hash.clone());
        }
    }

    None
}

impl LogCommand {
    pub fn new() -> Self {
	Self {}
    }
}
