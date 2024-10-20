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

use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex_string::HexString;
use sha1::Digest;
use sha1::Sha1;
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error;
use crate::error::ZatsuError;

pub fn save_object(values: &Vec<u8>, hash: &str) -> Result<(), ZatsuError> {
    let directory_name = hash[0..2].to_string();
    let path = format!(".zatsu/objects/{}", directory_name).to_string();
    let a_path = Path::new(&path);
    let exists = match a_path.try_exists() {
        Ok(exists) => exists,
        Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
    };
    if !exists {
        match fs::create_dir(&path) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };
    }

    let path = format!("{}/{}", &path, hash);
    let a_path = Path::new(&path);
    let exists = match a_path.try_exists() {
        Ok(exists) => exists,
        Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
    };
    if !exists {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        match encoder.write_all(&values) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        }
        let compressed = match encoder.finish() {
            Ok(compressed) => compressed,
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };

        match fs::write(path, compressed) {
            Ok(()) => (),
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };
    }

    Ok(())
}

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

    use std::env;

    use crate::Command;
    use crate::InitCommand;

    #[test]
    fn object_is_savable() {
        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(1);
        command.execute().unwrap();
        let string = "Hello, World!".to_string();
        let values = string.into_bytes();
        let result = save_object(&values, "12345678");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();

        fs::create_dir("tmp").unwrap();
        env::set_current_dir("tmp").unwrap();
        let command = InitCommand::new(2);
        command.execute().unwrap();
        let string = "Hello, World!".to_string();
        let values = string.into_bytes();
        let result = save_object(&values, "12345678");
        assert!(result.is_ok());
        env::set_current_dir("..").unwrap();
        fs::remove_dir_all("tmp").unwrap();
    }

    #[test]
    fn object_hash_is_calculatable() {
        let string = "Hello, World!".to_string();
        let values = string.into_bytes();
        let _hash = object_hash(&values, 1);
        let _hash = object_hash(&values, 2);
    }
}
