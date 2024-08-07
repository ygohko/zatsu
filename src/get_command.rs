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

use crate::Command;
use crate::error;
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
	    return Err(ZatsuError::new("main".to_string(), error::CODE_REVISION_NOT_FOUND));
	}

	let revision = match Revision::load(format!(".zatsu/revisions/{:02x}/{}.json", self.revision_number & 0xFF, self.revision_number)) {
	    Ok(revision) => revision,
	    Err(_) => return Err(ZatsuError::new("main".to_string(), error::CODE_LOADING_REVISION_FAILED)),
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
	    return Err(ZatsuError::new("main".to_string(), error::CODE_FILE_NOT_FOUND));
	}

	let directory_name = hash[0..2].to_string();
	let values = match fs::read(&PathBuf::from(format!(".zatsu/objects/{}/{}", directory_name, hash))) {
	    Ok(values) => values,
	    Err(_) => return Err(ZatsuError::new("main".to_string(), error::CODE_LOADING_FILE_FAILED)),
	};
	let mut decoder = ZlibDecoder::new(Vec::new());
	match decoder.write_all(&values) {
	    Ok(()) => (),
	    Err(_) =>return Err(ZatsuError::new("main".to_string(), error::CODE_LOADING_FILE_FAILED)),
	};
	let decoded = match decoder.finish() {
	    Ok(decoded) => decoded,
	    Err(_) =>return Err(ZatsuError::new("main".to_string(), error::CODE_LOADING_FILE_FAILED)),
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
	    Err(_) => return Err(ZatsuError::new("main".to_string(), error::CODE_SAVING_FILE_FAILED)),
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
