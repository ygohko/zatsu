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

use crate::Command;
use crate::process_garbage_collection;
use crate::Repository;
use crate::ZatsuError;

pub struct ForgetCommand {
    revision_count: i32,
}

impl Command for ForgetCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
	let mut repository = match Repository::load(".zatsu/repository.json") {
            Ok(repository) => repository,
            // TODO: Ensure repository is created when zatsu init.
            Err(_) => Repository {
		revision_numbers: Vec::new(),
            },
	};
	let current_count = repository.revision_numbers.len() as i32;
	let removed_count = current_count - self.revision_count;
	if removed_count <= 0 {
            return Ok(());
	}
	let index: usize = removed_count as usize;
	repository.revision_numbers = repository.revision_numbers.drain(index..).collect();
	repository.save(".zatsu/repository.json")?;
	process_garbage_collection()?;

	Ok(())
    }
}

impl ForgetCommand {
    pub fn new(revision_count: i32) -> Self {
	Self {
	    revision_count,
	}
    }
}
