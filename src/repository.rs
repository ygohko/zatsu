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
        let string = match fs::read_to_string(version_path) {
            Ok(string) => string,
            Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
        };

        println!("string: {}", string);
        
        let version: i32 = match string.parse() {
            Ok(version) => version,
            Err(_) => 1,
        };

        println!("version: {}", version);
        
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
