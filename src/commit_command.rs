use chrono::Utc;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use hex_string::HexString;
use std::io::Write;
use sha1::Digest;
use sha1::Sha1;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::Command;
use crate::Entry;
use crate::ERROR_LOADING_FILE_FAILED;
use crate::ERROR_READING_META_DATA_FAILED;
use crate::ERROR_PRODUCING_FINISHED;
use crate::ERROR_SAVING_FILE_FAILED;
use crate::FilePathProducer;
use crate::Repository;
use crate::Revision;
use crate::ZatsuError;

pub struct CommitCommand {
}

impl Command for CommitCommand {
    fn execute(&self) -> Result<(), ZatsuError> {
	let mut repository = match Repository::load(".zatsu/repository.json") {
            Ok(repository) => repository,
            Err(_) => Repository {
		revision_numbers: Vec::new(),
            },
	};
	let latest_revision = repository.latest_revision();
	let revision_number = latest_revision + 1;

	let mut producer = FilePathProducer::new(".".to_string());
	let now = Utc::now();
	let mut revision = Revision {
            commited: now.timestamp_millis(),
            entries: Vec::new(),
            description: "".to_string(),
	};
	let mut done = false;
	while !done {
            let result = producer.next();
            if result.is_ok() {
		let path = result.unwrap();
		println!("{}", path);
		let hash = match process_file(&PathBuf::from(path.clone())) {
                    Ok(hash) => hash,
                    Err(error) => return Err(error),
		};
		let entry = Entry{
                    path: path,
                    hash: hash,
                    permission: 0o644,
		};
		revision.entries.push(entry);
            }
            else {
		let error = result.unwrap_err();

		println!("error.code: {}", error.code);

		if error.code == ERROR_PRODUCING_FINISHED {
                    done = true;
		}
            }
	}

	let path = format!(".zatsu/revisions/{:02x}", revision_number & 0xFF).to_string();
	let a_path = Path::new(&path);
	let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};
	if !exists {
            match fs::create_dir(&path) {
		Ok(()) => (),
		Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };
	}
	match revision.save(format!("{}/{}.json", &path, revision_number)) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};
	repository.revision_numbers.push(revision_number);
	match repository.save(&PathBuf::from(".zatsu/repository.json")) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
	};

	Ok(())	
    }
}

impl CommitCommand {
    pub fn new() -> Self {
	Self{}
    }
}

fn process_file(path: impl AsRef<Path>) -> Result<String, ZatsuError> {
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_READING_META_DATA_FAILED)),
    };
    let mut hex_string = String::new();
    if metadata.is_file() {
        println!("This is file.");
        let values = match fs::read(path) {
            Ok(values) => values,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_LOADING_FILE_FAILED)),
        };
        println!("{} bytes read.", values.len());
        let mut sha1 = Sha1::new();
        sha1.update(values.clone());
        let hash = sha1.finalize();
        let hash_values = hash.to_vec();
        println!("{} bytes of hash generated.", hash_values.len());
        // println!("{}", hash_values);
        let hex = HexString::from_bytes(&hash_values);
        hex_string = hex.as_string();
        println!("{}", hex_string);

        let directory_name = hex_string[0..2].to_string();
        let path = format!(".zatsu/objects/{}", directory_name).to_string();
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
        };
        if !exists {
            match fs::create_dir(&path) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };
        }

        let path = format!("{}/{}", &path, hex_string);
        let a_path = Path::new(&path);
        let exists = match a_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
        };
        if !exists {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            match encoder.write_all(&values) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            }
            let compressed = match encoder.finish() {
                Ok(compressed) => compressed,
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };

            match fs::write(path, compressed) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
            };
        }
    } else {
        println!("This is not file.");
    }

    Ok(hex_string)
}
