use flate2::write::ZlibDecoder;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::Command;
use crate::ERROR_FILE_NOT_FOUND;
use crate::ERROR_LOADING_FILE_FAILED;
use crate::ERROR_LOADING_REVISION_FAILED;
use crate::ERROR_REVISION_NOT_FOUND;
use crate::ERROR_SAVING_FILE_FAILED;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub struct GetCommand {
    revision_number: i32,
    path: String,
}

impl Command for GetCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
	let repository = match Repository::load(".zatsu/repository.json") {
	    Ok(repository) => repository,
	    Err(_) => Repository {
		revision_numbers: Vec::new(),
	    },
	};
	let mut found = false;
	for a_revision_number in repository.revision_numbers {
	    if a_revision_number == self.revision_number {
		found = true;
	    } 
	}
	if !found {
	    return Err(ZatsuError::new("main".to_string(), ERROR_REVISION_NOT_FOUND));
	}

	let revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", self.revision_number & 0xFF, self.revision_number)) {
	    Ok(revision) => revision,
	    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_REVISION_FAILED)),
	};
	let mut hash = "".to_string();
	let mut found = false;
	for entry in revision.entries {
	    if entry.path == *self.path {
		found = true;
		hash = entry.hash;
	    }
	}
	if !found {
	    return Err(ZatsuError::new("main".to_string(), ERROR_FILE_NOT_FOUND));
	}

	let directory_name = hash[0..2].to_string();
	let values = match fs::read(&PathBuf::from(format!(".zatsu/objects/{}/{}", directory_name, hash))) {
	    Ok(values) => values,
	    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
	};
	let mut decoder = ZlibDecoder::new(Vec::new());
	match decoder.write_all(&values) {
	    Ok(()) => (),
	    Err(_) =>return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
	};
	let decoded = match decoder.finish() {
	    Ok(decoded) => decoded,
	    Err(_) =>return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
	};
	let split: Vec<_> = self.path.split("/").collect();
	let mut file_name = "out.dat".to_string();
	if split.len() >= 1 {
	    let original_file_name = split[split.len() - 1].to_string();
	    let split: Vec<_> = original_file_name.split(".").collect();
	    if split.len() > 1 {
		file_name = format!("{}-r{}.{}", split[0], self.revision_number, split[1]);
	    }
	}
	match fs::write(&PathBuf::from(file_name), decoded) {
	    Ok(()) => (),
	    Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};

	Ok(())
    }	
}

impl GetCommand {
    pub fn new(revision_number: i32, path: &str) -> Self {
	Self {
	    revision_number,
	    path: path.to_string(),
	}
    }
}
