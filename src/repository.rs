use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ZatsuError {
}

impl fmt::Display for ZatsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "Zatsu error.")
    }
}

impl Error for ZatsuError {
}

#[derive(Serialize, Deserialize)]
pub struct Repository {
    pub revisions: Vec<i32>,
}

impl Repository {
    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let serialized = match serde_json::to_string(self) {
	    Ok(serialized) => serialized,
	    Err(_) => return Err(Box::new(ZatsuError {})),
	};
	println!("serialized: {}", serialized);
	let _ = match fs::write(path, serialized) {
	    Ok(result) => result,
	    Err(_) => return Err(Box::new(ZatsuError {})),
	};

	Ok(())
    }

    pub fn latest_revision(&self) -> i32 {
	let count = self.revisions.len();
	if count == 0 {
	    return 0;
	}

	return self.revisions[count - 1];
    }

    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
	let serialized = match fs::read_to_string(path) {
	    Ok(serialized) => serialized,
	    Err(_) => return Err(Box::new(ZatsuError {})),
	};
	let repository: Repository = match serde_json::from_str(&serialized) {
	    Ok(repository) => repository,
	    Err(_) => return  Err(Box::new(ZatsuError {})),
	};

	Ok(repository)
    }
}
