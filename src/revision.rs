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

use crate::entry::Entry;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::ZatsuError;

pub const ERROR_ID: ErrorId = "revision";

const ERROR_CODE_LOADING_FILE_FAILED: ErrorCode = 1;
const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 2;
const ERROR_CODE_DESERIALIZATION_FAILED: ErrorCode = 3;
const ERROR_CODE_SERIALIZATION_FAILED: ErrorCode = 4;

#[derive(Serialize, Deserialize)]
pub struct Revision {
    pub commited: i64,
    pub entries: Vec<Entry>,
    // TOOD: Use this reserved field.
    pub description: String,
}

impl Revision {
    pub fn load(path: &str) -> Result<Revision, ZatsuError> {
        let serialized = match fs::read_to_string(path) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_LOADING_FILE_FAILED)),
        };
        let revision = match serde_json::from_str(&serialized) {
            Ok(revision) => revision,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_DESERIALIZATION_FAILED)),
        };

        Ok(revision)
    }

    pub fn save(&self, path: &str) -> Result<(), ZatsuError> {
        let serialized = match serde_json::to_string(self) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SERIALIZATION_FAILED)),
        };

        let _ = match std::fs::write(path, serialized) {
            Ok(result) => result,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempdir::TempDir;

    use crate::commons::ToString;
    use crate::Command;
    use crate::CommitCommand;
    use crate::InitCommand;

    #[test]
    fn is_loadable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut path = temp_path.clone();
        path.push(".zatsu");
        path.push("revisions");
        path.push("01");
        path.push("1.json");
        Revision::load(&path).unwrap();

        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut command = CommitCommand::new();
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut path = temp_path.clone();
        path.push(".zatsu");
        path.push("revisions");
        path.push("01");
        path.push("1.json");
        Revision::load(&path).unwrap();
    }

    #[test]
    fn is_savable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let mut path = temp_path.clone();
        path.push(".zatsu");
        path.push("revisions");
        path.push("01");
        fs::create_dir_all(&path).unwrap();
        let revision = Revision {
            commited: 123,
            entries: vec![],
            description: "".to_string(),
        };
        path.push("1.json");
        revision.save(&path).unwrap();
    }
}
