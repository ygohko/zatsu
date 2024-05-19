use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct GeneralError {
}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "General error.")
    }
}

impl Error for GeneralError {
}

#[derive(Serialize, Deserialize)]
pub struct Repository {
    pub revisions: Vec<i32>,
}

impl Repository {
    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let serialized = match serde_json::to_string(self) {
	    Ok(serialized) => serialized,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};
	println!("serialized: {}", serialized);
	let _ = match fs::write(path, serialized) {
	    Ok(result) => result,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};

	Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
	let serialized = match fs::read_to_string(path) {
	    Ok(serialized) => serialized,
	    Err(_) => return Err(Box::new(GeneralError {})),
	};
	let repository: Repository = match serde_json::from_str(&serialized) {
	    Ok(repository) => repository,
	    Err(_) => return  Err(Box::new(GeneralError {})),
	};

	Ok(repository)
    }
}
