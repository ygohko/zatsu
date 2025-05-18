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

use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::commons;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::ZatsuError;
use crate::Revision;

pub const ERROR_ID: ErrorId = "repository";

const ERROR_CODE_LOADING_REVISION_FAILED: ErrorCode = 1;
const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 2;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 3;
const ERROR_CODE_DESERIALIZATION_FAILED: ErrorCode = 4;
const ERROR_CODE_SERIALIZATION_FAILED: ErrorCode = 5;

pub trait Repository {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<(), ZatsuError>;
    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError>;
    // ADHOC: Use given repository path.
    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
        path: &str,
    ) -> Result<(), ZatsuError>;
    fn revision_numbers(&self) -> Vec<i32>;
    fn set_revision_numbers(&mut self, revision_numbers: &Vec<i32>);
    fn version(&self) -> i32;
    fn latest_revision(&self) -> i32;
    fn to_serializable_v1(&self) -> SerializableRepositoryV1;
    fn object_hash(&self, values: &Vec<u8>) -> String;
}

struct RepositoryBase {
    // TODO: Store repository path.
    revision_numbers: Vec<i32>,
    version: i32,
}

impl Repository for RepositoryBase {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<(), ZatsuError> {
        let repository_v1 = self.to_serializable_v1();
        repository_v1.save(path)?;

        Ok(())
    }

    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError> {
        let revision = match Revision::load(format!(
            ".zatsu/revisions/{:02x}/{}.json",
            revision_number & 0xFF,
            revision_number
        )) {
            Ok(revision) => revision,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_LOADING_REVISION_FAILED,
                ))
            }
        };

        Ok(revision)
    }

    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
        path: &str,
    ) -> Result<(), ZatsuError> {
        // TODO: Use path stored in Repository.
        let mut revision_path = PathBuf::from(&path);
        revision_path.push(".zatsu");
        revision_path.push("revisions");
        revision_path.push(format!("{:02x}", revision_number & 0xFF));

        println!("revision_path: {}", revision_path.display());
        
        let exists = match revision_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };
        if !exists {
            match fs::create_dir(&revision_path) {
                Ok(()) => (),
                Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
            };
        }
        revision_path.push(format!("{}.json", revision_number));

        println!("revision_path: {}", revision_path.display());
        
        match revision.save(revision_path) {
            Ok(_) => (),
            // Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
            Err(error) => return Err(error),
        };
        let mut revision_numbers = self.revision_numbers();
        revision_numbers.push(revision_number);
        self.set_revision_numbers(&revision_numbers);
        match self.save(&Path::new(".zatsu")) {
            Ok(_) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }

    fn revision_numbers(&self) -> Vec<i32> {
        self.revision_numbers.clone()
    }

    fn set_revision_numbers(&mut self, revision_numbers: &Vec<i32>) {
        self.revision_numbers = revision_numbers.clone();
    }

    fn version(&self) -> i32 {
        self.version
    }

    fn latest_revision(&self) -> i32 {
        let count = self.revision_numbers.len();
        if count == 0 {
            return 0;
        }

        return self.revision_numbers[count - 1];
    }

    fn to_serializable_v1(&self) -> SerializableRepositoryV1 {
        SerializableRepositoryV1 {
            revision_numbers: self.revision_numbers.clone(),
        }
    }

    fn object_hash(&self, _values: &Vec<u8>) -> String {
        panic!("This method is not implemented.");
    }
}

impl RepositoryBase {
    fn from_serializable_v1(repository_v1: &SerializableRepositoryV1) -> Self {
        RepositoryBase {
            revision_numbers: repository_v1.revision_numbers.clone(),
            version: 1,
        }
    }
}

struct RepositoryV1 {
    base: RepositoryBase,
}

impl Repository for RepositoryV1 {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<(), ZatsuError> {
        self.base.save(path)
    }

    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError> {
        self.base.load_revision(revision_number)
    }

    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
        path: &str,
    ) -> Result<(), ZatsuError> {
        self.base.save_revision(revision, revision_number, path)
    }

    fn revision_numbers(&self) -> Vec<i32> {
        self.base.revision_numbers()
    }

    fn set_revision_numbers(&mut self, revision_numbers: &Vec<i32>) {
        self.base.set_revision_numbers(revision_numbers)
    }

    fn version(&self) -> i32 {
        self.base.version()
    }

    fn latest_revision(&self) -> i32 {
        self.base.latest_revision()
    }

    fn to_serializable_v1(&self) -> SerializableRepositoryV1 {
        self.base.to_serializable_v1()
    }

    fn object_hash(&self, values: &Vec<u8>) -> String {
        commons::object_hash(values, 1)
    }
}

struct RepositoryV2 {
    base: RepositoryBase,
}

impl Repository for RepositoryV2 {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<(), ZatsuError> {
        self.base.save(path)
    }

    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError> {
        self.base.load_revision(revision_number)
    }

    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
        path: &str,
    ) -> Result<(), ZatsuError> {
        self.base.save_revision(revision, revision_number, path)
    }

    fn revision_numbers(&self) -> Vec<i32> {
        self.base.revision_numbers()
    }

    fn set_revision_numbers(&mut self, revision_numbers: &Vec<i32>) {
        self.base.set_revision_numbers(revision_numbers)
    }

    fn version(&self) -> i32 {
        self.base.version()
    }

    fn latest_revision(&self) -> i32 {
        self.base.latest_revision()
    }

    fn to_serializable_v1(&self) -> SerializableRepositoryV1 {
        self.base.to_serializable_v1()
    }

    fn object_hash(&self, values: &Vec<u8>) -> String {
        commons::object_hash(values, 2)
    }
}

pub mod factory {
    use super::*;

    pub fn new(version: i32) -> Box<dyn Repository> {
        let base = RepositoryBase {
            revision_numbers: Vec::new(),
            version: version,
        };
        if version == 1 {
            Box::new(RepositoryV1 { base: base })
        } else {
            Box::new(RepositoryV2 { base: base })
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Box<dyn Repository>, ZatsuError> {
        let version_path = path.as_ref().join("version.txt");
        let mut string = match fs::read_to_string(version_path) {
            Ok(string) => string,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        string = string.replace("\n", "");

        let version: i32 = match string.parse() {
            Ok(version) => version,
            Err(_) => 1,
        };

        let repository_v1 = SerializableRepositoryV1::load(path)?;
        let mut base = RepositoryBase::from_serializable_v1(&repository_v1);
        base.version = version;
        if version == 1 {
            Ok(Box::new(RepositoryV1 { base: base }))
        } else {
            Ok(Box::new(RepositoryV2 { base: base }))
        }
    }

    #[allow(dead_code)]
    pub fn with_arguments(revision_numbers: &Vec<i32>, version: i32) -> Box<dyn Repository> {
        let base = RepositoryBase {
            revision_numbers: revision_numbers.to_vec(),
            version: version,
        };

        if version == 1 {
            Box::new(RepositoryV1 { base: base })
        } else {
            Box::new(RepositoryV2 { base: base })
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableRepositoryV1 {
    revision_numbers: Vec<i32>,
}

impl SerializableRepositoryV1 {
    fn save(&self, path: impl AsRef<Path>) -> Result<(), ZatsuError> {
        let serialized = match serde_json::to_string(self) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SERIALIZATION_FAILED)),
        };
        let json_path = path.as_ref().join("repository.json");
        let _ = match fs::write(json_path, serialized) {
            Ok(result) => result,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }

    fn load(path: impl AsRef<Path>) -> Result<Self, ZatsuError> {
        let json_path = path.as_ref().join("repository.json");
        let serialized = match fs::read_to_string(json_path) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        let repository: SerializableRepositoryV1 = match serde_json::from_str(&serialized) {
            Ok(repository) => repository,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_DESERIALIZATION_FAILED)),
        };

        Ok(repository)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;

    use crate::commons;
    use crate::Command;
    use crate::InitCommand;

    #[test]
    fn repository_is_savable() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let result = repository.save(&".");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();

        let repository = factory::with_arguments(&vec![1, 2, 3], 2);
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let result = repository.save(&".");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn repository_is_gettable_latest_revision() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        assert_eq!(3, repository.latest_revision());

        let repository = factory::with_arguments(&vec![1, 2, 3], 2);
        assert_eq!(3, repository.latest_revision());
    }

    #[test]
    fn repository_is_convertable_to_repository_v1() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let repository_v1 = repository.to_serializable_v1();
        assert_eq!(
            repository.revision_numbers(),
            repository_v1.revision_numbers
        );
    }

    #[test]
    fn repository_is_loadable() {
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(1);
        command.execute().unwrap();
        let result = factory::load(".zatsu");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();

        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(2);
        command.execute().unwrap();
        let result = factory::load(".zatsu");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn repository_is_convertable_from_repository_v1() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let repository_v1 = repository.to_serializable_v1();
        let repository = RepositoryBase::from_serializable_v1(&repository_v1);
        assert_eq!(repository_v1.revision_numbers, repository.revision_numbers);
    }

    #[test]
    fn repository_v1_is_savable() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let repository_v1 = repository.to_serializable_v1();
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let result = repository_v1.save(".");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn repository_v1_is_loadable() {
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(1);
        command.execute().unwrap();
        let result = SerializableRepositoryV1::load(".zatsu");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn repository_is_calculatable_object_hash() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let mut values: Vec<u8> = Vec::new();
        values.push(1);
        values.push(2);
        values.push(3);
        let hash1 = repository.object_hash(&values);
        let hash2 = commons::object_hash(&values, 1);
        assert_eq!(hash1, hash2);

        let repository = factory::with_arguments(&vec![1, 2, 3], 2);
        let mut values: Vec<u8> = Vec::new();
        values.push(1);
        values.push(2);
        values.push(3);
        let hash1 = repository.object_hash(&values);
        let hash2 = commons::object_hash(&values, 2);
        assert_eq!(hash1, hash2);
    }
}
