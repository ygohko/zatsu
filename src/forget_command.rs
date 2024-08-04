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
