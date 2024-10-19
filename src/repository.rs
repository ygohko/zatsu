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

use crate::error;
use crate::error::ZatsuError;

pub struct Repository {
    pub revision_numbers: Vec<i32>,
    pub version: i32,
}

impl Repository {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ZatsuError> {
        let repository_v1 = self.to_v1();
        repository_v1.save(path)?;

        Ok(())
    }

    pub fn latest_revision(&self) -> i32 {
        let count = self.revision_numbers.len();
        if count == 0 {
            return 0;
        }

        return self.revision_numbers[count - 1];
    }

    pub fn to_v1(&self) -> RepositoryV1 {
        RepositoryV1 {
            revision_numbers: self.revision_numbers.clone(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ZatsuError> {
        let version_path = path.as_ref().join("version.txt");
        let mut string = match fs::read_to_string(version_path) {
            Ok(string) => string,
            Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
        };
        string = string.replace("\n", "");

        let version: i32 = match string.parse() {
            Ok(version) => version,
            Err(_) => 1,
        };

        let repository_v1 = RepositoryV1::load(path)?;
        let mut repository = Repository::from_v1(&repository_v1);
        repository.version = version;

        Ok(repository)
    }

    pub fn from_v1(repository_v1: &RepositoryV1) -> Self {
        Repository {
            revision_numbers: repository_v1.revision_numbers.clone(),
            version: 1,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryV1 {
    revision_numbers: Vec<i32>,
}

impl RepositoryV1 {
    fn save(&self, path: impl AsRef<Path>) -> Result<(), ZatsuError> {
        let serialized = match serde_json::to_string(self) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(error::CODE_SERIALIZATION_FAILED)),
        };
        let json_path = path.as_ref().join("repository.json");
        let _ = match fs::write(json_path, serialized) {
            Ok(result) => result,
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }

    fn load(path: impl AsRef<Path>) -> Result<Self, ZatsuError> {
        let json_path = path.as_ref().join("repository.json");
        let serialized = match fs::read_to_string(json_path) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
        };
        let repository: RepositoryV1 = match serde_json::from_str(&serialized) {
            Ok(repository) => repository,
            Err(_) => return Err(ZatsuError::new(error::CODE_DESERIALIZATION_FAILED)),
        };

        Ok(repository)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;

    use crate::Command;
    use crate::InitCommand;
    
    #[test]
    fn repository_is_savable() {
        let repository = Repository {
            revision_numbers: vec![1, 2, 3],
            version: 1,
        };
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let result = repository.save(".");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();

        let repository = Repository {
            revision_numbers: vec![1, 2, 3],
            version: 2,
        };
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let result = repository.save(".");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn repository_is_gettable_latest_revision() {
        let repository = Repository {
            revision_numbers: vec![1, 2, 3],
            version: 1,
        };
        assert_eq!(3, repository.latest_revision());

        let repository = Repository {
            revision_numbers: vec![1, 2, 3],
            version: 2,
        };
        assert_eq!(3, repository.latest_revision());
    }

    #[test]
    fn repository_is_convertable_to_repository_v1() {
        let repository = Repository {
            revision_numbers: vec![1, 2, 3],
            version: 1,
        };
        let repository_v1 = repository.to_v1();
        assert_eq!(repository.revision_numbers, repository_v1.revision_numbers);
    }

    #[test]
    fn repository_is_loadable() {
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(1);
        command.execute().unwrap();
        let result = Repository::load(".zatsu");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();

        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(2);
        command.execute().unwrap();
        let result = Repository::load(".zatsu");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn repository_is_convertable_from_repository_v1() {
        let repository = Repository {
            revision_numbers: vec![1, 2, 3],
            version: 1,
        };
        let repository_v1 = repository.to_v1();
        let repository = Repository::from_v1(&repository_v1);
        assert_eq!(repository_v1.revision_numbers, repository.revision_numbers);
    }
}
