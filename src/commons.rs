/*
 * Copyright (c) 2024 - 2025 Yasuaki Gohko
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

use camino::Utf8Path;
use camino::Utf8PathBuf;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use hex_string::HexString;
use sha1::Digest;
use sha1::Sha1;
use sha2::Sha256;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::ZatsuError;

pub const ERROR_ID: ErrorId = "commons";

pub const ERROR_CODE_SAVING_FILE_FAILED: ErrorCode = 1;

/// A trait for operating on file paths.
pub trait OperatePath {
    /// Returns the file name of the path, or an empty string if not present.
    ///
    /// # Returns
    ///
    /// * `String` - The file name.
    fn file_name_or_empty(&self) -> String;
    /// Returns the extension of the path, or an empty string if not present.
    ///
    /// # Returns
    ///
    /// * `String` - The extension.
    fn extension_or_empty(&self) -> String;
    /// Returns the parent directory of the path, or an empty string if not present.
    ///
    /// # Returns
    ///
    /// * `String` - The parent directory.
    fn parent_or_empty(&self) -> String;
    /// Converts the path to a `String`.
    ///
    /// # Returns
    ///
    /// * `String` - The path as a string.
    fn to_string_easy(&self) -> String;
}

impl OperatePath for Utf8PathBuf {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name,
            None => return "".to_string(),
        };

        file_name.to_string()
    }

    fn extension_or_empty(&self) -> String {
        let extension = match self.extension() {
            Some(extension) => extension,
            None => return "".to_string(),
        };

        extension.to_string()
    }

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.as_str().to_string()
    }
}

impl OperatePath for Utf8Path {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name,
            None => return "".to_string(),
        };

        file_name.to_string()
    }

    fn extension_or_empty(&self) -> String {
        let extension = match self.extension() {
            Some(extension) => extension,
            None => return "".to_string(),
        };

        extension.to_string()
    }

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.as_str().to_string()
    }
}

impl OperatePath for PathBuf {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => return "".to_string(),
        };

        file_name
    }

    fn extension_or_empty(&self) -> String {
        let extension = match self.extension() {
            Some(extension) => extension.to_string_lossy().to_string(),
            None => return "".to_string(),
        };

        extension
    }

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl OperatePath for Path {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => return "".to_string(),
        };

        file_name
    }

    fn extension_or_empty(&self) -> String {
        let extension = match self.extension() {
            Some(extension) => extension.to_string_lossy().to_string(),
            None => return "".to_string(),
        };

        extension
    }

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

/// A trait for converting types to `String`.
pub trait ToString {
    /// Converts the value to a `String`.
    fn to_string(&self) -> String;
}

impl ToString for OsStr {
    /// Converts an `OsStr` to a `String`.
    fn to_string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl ToString for Path {
    /// Converts a `Path` to a `String`.
    fn to_string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl ToString for PathBuf {
    /// Converts a `PathBuf` to a `String`.
    fn to_string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

/// Saves an object (file content) to the repository's object store.
///
/// This function compresses the provided `values` using Zlib and stores them
/// in a directory structure based on the `hash`.
///
/// # Arguments
///
/// * `values` - The content of the object as a vector of bytes.
/// * `hash` - The hash of the object, used for naming and directory structure.
/// * `repository_path` - The path to the repository.
///
/// # Returns
///
/// A `Result` indicating success or an error if saving fails.
pub fn save_object(values: &Vec<u8>, hash: &str, repository_path: &str) -> Result<(), ZatsuError> {
    // TODO: Move to repository.rs?

    let directory_name = hash[0..2].to_string();
    let path = format!("{}/objects/{}", repository_path, directory_name).to_string();
    let a_path = Path::new(&path);
    let exists = match a_path.try_exists() {
        Ok(exists) => exists,
        Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
    };
    if !exists {
        match fs::create_dir(&path) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };
    }

    let path = format!("{}/{}", &path, hash);
    let a_path = Path::new(&path);
    let exists = match a_path.try_exists() {
        Ok(exists) => exists,
        Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
    };
    if !exists {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        match encoder.write_all(&values) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        }
        let compressed = match encoder.finish() {
            Ok(compressed) => compressed,
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };

        match fs::write(path, compressed) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(ERROR_ID, ERROR_CODE_SAVING_FILE_FAILED)),
        };
    }

    Ok(())
}

/// Calculates the hash of the given values based on the specified version.
///
/// If `version` is 1 or less, SHA-1 is used. Otherwise, SHA-256 is used.
///
/// # Arguments
///
/// * `values` - The input data as a vector of bytes.
/// * `version` - The version of the hashing algorithm to use.
///
/// # Returns
///
/// The calculated hash as a `String`.
pub fn object_hash(values: &Vec<u8>, version: i32) -> String {
    let result: String;
    if version <= 1 {
        let mut sha1 = Sha1::new();
        sha1.update(values.clone());
        let hash = sha1.finalize();
        let hash_values = hash.to_vec();
        let hex = HexString::from_bytes(&hash_values);
        result = hex.as_string();
    } else {
        let mut sha256 = Sha256::new();
        sha256.update(values.clone());
        let hash = sha256.finalize();
        let hash_values = hash.to_vec();
        let hex = HexString::from_bytes(&hash_values);
        result = hex.as_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempdir::TempDir;

    use crate::Command;
    use crate::InitCommand;
    use crate::commons::ToString;

    #[test]
    fn object_is_savable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(1);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let string = "Hello, World!".to_string();
        let values = string.into_bytes();
        let mut path = temp_path.clone();
        path.push(".zatsu");
        save_object(&values, "12345678", &path.to_string()).unwrap();

        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut command = InitCommand::new(2);
        command.path = temp_path.to_string();
        command.execute().unwrap();
        let string = "Hello, World!".to_string();
        let values = string.into_bytes();
        let mut path = temp_path.clone();
        path.push(".zatsu");
        save_object(&values, "12345678", &path.to_string()).unwrap();
    }

    #[test]
    fn object_hash_is_calculatable() {
        let string = "Hello, World!".to_string();
        let values = string.into_bytes();
        let _hash = object_hash(&values, 1);
        let _hash = object_hash(&values, 2);
    }
}
