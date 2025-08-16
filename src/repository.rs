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
use std::path::PathBuf;

use crate::Revision;
use crate::commons;
use crate::commons::ToString;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::ZatsuError;

pub const ERROR_ID: ErrorId = "repository";

const ERROR_CODE_LOADING_REVISION_FAILED: ErrorCode = 1;
const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 2;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 3;
const ERROR_CODE_DESERIALIZATION_FAILED: ErrorCode = 4;
const ERROR_CODE_SERIALIZATION_FAILED: ErrorCode = 5;

/// Defines the interface for a Zatsu repository.
pub trait Repository {
    /// Saves the repository metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the repository metadata should be saved.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    fn save(&self, path: &str) -> Result<(), ZatsuError>;
    /// Loads a specific revision from the repository.
    ///
    /// # Arguments
    ///
    /// * `revision_number` - The number of the revision to load.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Revision` or an error.
    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError>;
    /// Saves a revision to the repository.
    ///
    /// # Arguments
    ///
    /// * `revision` - The `Revision` to save.
    /// * `revision_number` - The number of the revision.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
    ) -> Result<(), ZatsuError>;
    /// Returns a vector of all revision numbers in the repository.
    fn revision_numbers(&self) -> Vec<i32>;
    /// Sets the revision numbers for the repository.
    ///
    /// # Arguments
    ///
    /// * `revision_numbers` - A vector of revision numbers.
    fn set_revision_numbers(&mut self, revision_numbers: &Vec<i32>);
    /// Returns the version of the repository.
    fn version(&self) -> i32;
    /// Returns the path to the repository.
    fn path(&self) -> String;
    /// Returns the latest revision number in the repository.
    fn latest_revision(&self) -> i32;
    /// Converts the repository to a serializable version 1 representation.
    fn to_serializable_v1(&self) -> SerializableRepositoryV1;
    /// Calculates the object hash for given values.
    ///
    /// # Arguments
    ///
    /// * `values` - The values to hash.
    ///
    /// # Returns
    ///
    /// The calculated hash as a `String`.
    fn object_hash(&self, values: &Vec<u8>) -> String;
}

/// Base implementation for the `Repository` trait.
struct RepositoryBase {
    revision_numbers: Vec<i32>,
    version: i32,
    path: String,
}

impl Repository for RepositoryBase {
    fn save(&self, path: &str) -> Result<(), ZatsuError> {
        let repository_v1 = self.to_serializable_v1();
        repository_v1.save(path)?;

        Ok(())
    }

    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError> {
        let mut path = PathBuf::from(&self.path);
        path.push("revisions");
        path.push(format!("{:02x}", revision_number & 0xFF));
        path.push(format!("{}.json", revision_number));
        let revision = match Revision::load(&path.to_string()) {
            Ok(revision) => revision,
            Err(_) => {
                return Err(ZatsuError::new(
                    ERROR_ID,
                    ERROR_CODE_LOADING_REVISION_FAILED,
                ));
            }
        };

        Ok(revision)
    }

    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
    ) -> Result<(), ZatsuError> {
        let mut revision_path = PathBuf::from(&self.path);
        revision_path.push("revisions");
        revision_path.push(format!("{:02x}", revision_number & 0xFF));
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
        match revision.save(&revision_path.to_string()) {
            Ok(_) => (),
            Err(error) => return Err(error),
        };
        let mut revision_numbers = self.revision_numbers();
        revision_numbers.push(revision_number);
        self.set_revision_numbers(&revision_numbers);
        match self.save(&self.path) {
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

    fn path(&self) -> String {
        self.path.clone()
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
    /// Creates a `RepositoryBase` from a `SerializableRepositoryV1`.
    ///
    /// # Arguments
    ///
    /// * `repository_v1` - The serializable repository version 1.
    /// * `path` - The path to the repository.
    ///
    /// # Returns
    ///
    /// A new `RepositoryBase` instance.
    fn from_serializable_v1(repository_v1: &SerializableRepositoryV1, path: &str) -> Self {
        RepositoryBase {
            revision_numbers: repository_v1.revision_numbers.clone(),
            version: 1,
            path: path.to_string(),
        }
    }
}

/// Represents a version 1 repository.
struct RepositoryV1 {
    base: RepositoryBase,
}

impl Repository for RepositoryV1 {
    fn save(&self, path: &str) -> Result<(), ZatsuError> {
        self.base.save(path)
    }

    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError> {
        self.base.load_revision(revision_number)
    }

    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
    ) -> Result<(), ZatsuError> {
        self.base.save_revision(revision, revision_number)
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

    fn path(&self) -> String {
        self.base.path()
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

/// Represents a version 2 repository.
struct RepositoryV2 {
    base: RepositoryBase,
}

impl Repository for RepositoryV2 {
    fn save(&self, path: &str) -> Result<(), ZatsuError> {
        self.base.save(path)
    }

    fn load_revision(&self, revision_number: i32) -> Result<Revision, ZatsuError> {
        self.base.load_revision(revision_number)
    }

    fn save_revision(
        &mut self,
        revision: &Revision,
        revision_number: i32,
    ) -> Result<(), ZatsuError> {
        self.base.save_revision(revision, revision_number)
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

    fn path(&self) -> String {
        self.base.path()
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

    /// Creates a new repository instance based on the specified version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version of the repository to create.
    ///
    /// # Returns
    ///
    /// A `Box` containing a new `Repository` trait object.
    pub fn new(version: i32) -> Box<dyn Repository> {
        let base = RepositoryBase {
            revision_numbers: Vec::new(),
            version: version,
            path: ".zatsu".to_string(),
        };
        if version == 1 {
            Box::new(RepositoryV1 { base: base })
        } else {
            Box::new(RepositoryV2 { base: base })
        }
    }

    /// Loads a repository from the specified path.
    ///
    /// This function reads the repository version and loads the appropriate repository implementation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the repository.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Box` with the loaded `Repository` trait object or an error.
    pub fn load(path: &str) -> Result<Box<dyn Repository>, ZatsuError> {
        let mut version_path = PathBuf::from(path);
        version_path.push("version.txt");
        let mut string = match fs::read_to_string(version_path) {
            Ok(string) => string,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        string = string.replace("\n", "");

        let version: i32 = match string.parse() {
            Ok(version) => version,
            Err(_) => 1,
        };

        let repository_v1 = SerializableRepositoryV1::load(&path)?;
        let mut base = RepositoryBase::from_serializable_v1(&repository_v1, path);
        base.version = version;
        if version == 1 {
            Ok(Box::new(RepositoryV1 { base: base }))
        } else {
            Ok(Box::new(RepositoryV2 { base: base }))
        }
    }

    #[allow(dead_code)]
    /// Creates a new repository instance with specified revision numbers and version.
    ///
    /// This function is primarily used for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `revision_numbers` - A vector of revision numbers.
    /// * `version` - The version of the repository.
    ///
    /// # Returns
    ///
    /// A `Box` containing a new `Repository` trait object.
    pub fn with_arguments(revision_numbers: &Vec<i32>, version: i32) -> Box<dyn Repository> {

        let base = RepositoryBase {
            revision_numbers: revision_numbers.to_vec(),
            version: version,
            path: ".zatsu".to_string(),
        };

        if version == 1 {
            Box::new(RepositoryV1 { base: base })
        } else {
            Box::new(RepositoryV2 { base: base })
        }
    }
}

/// A serializable representation of the repository for version 1.
#[derive(Serialize, Deserialize)]
pub struct SerializableRepositoryV1 {
    revision_numbers: Vec<i32>,
}

impl SerializableRepositoryV1 {
    /// Saves the serializable repository to a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path where the `repository.json` file will be saved.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    fn save(&self, path: &str) -> Result<(), ZatsuError> {
        let serialized = match serde_json::to_string(self) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SERIALIZATION_FAILED)),
        };

        let mut json_path = PathBuf::from(path);
        json_path.push("repository.json");
        let _ = match fs::write(json_path, serialized) {
            Ok(result) => result,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }

    /// Loads a serializable repository from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path where the `repository.json` file is located.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `SerializableRepositoryV1` or an error.
    fn load(path: &str) -> Result<Self, ZatsuError> {
        let mut json_path = PathBuf::from(path);
        json_path.push("repository.json");
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

    use tempdir::TempDir;

    use crate::Command;
    use crate::InitCommand;
    use crate::commons;
    use crate::commons::ToString;

    #[test]
    fn repository_is_savable() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let result = repository.save(&temp_path.to_string());
        assert!(result.is_ok());

        let repository = factory::with_arguments(&vec![1, 2, 3], 2);
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let result = repository.save(&temp_path.to_string());
        assert!(result.is_ok());
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
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut repository_path = temp_path.clone();
        repository_path.push(".zatsu");
        let result = factory::load(&repository_path.to_string());
        assert!(result.is_ok());

        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut repository_path = temp_path.clone();
        repository_path.push(".zatsu");
        let result = factory::load(&repository_path.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn repository_is_convertable_from_repository_v1() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let repository_v1 = repository.to_serializable_v1();
        let repository = RepositoryBase::from_serializable_v1(&repository_v1, ".zatsu");
        assert_eq!(repository_v1.revision_numbers, repository.revision_numbers);
    }

    #[test]
    fn repository_v1_is_savable() {
        let repository = factory::with_arguments(&vec![1, 2, 3], 1);
        let repository_v1 = repository.to_serializable_v1();
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let result = repository_v1.save(&temp_path.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn repository_v1_is_loadable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut repository_path = temp_path.clone();
        repository_path.push(".zatsu");
        let result = SerializableRepositoryV1::load(&repository_path.to_string());
        assert!(result.is_ok());
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
