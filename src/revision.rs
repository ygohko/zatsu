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

use crate::entry::Entry;
use crate::error;
use crate::error::ZatsuError;

#[derive(Serialize, Deserialize)]
pub struct Revision {
    pub commited: i64,
    pub entries: Vec<Entry>,
    // TOOD: Use this reserved field.
    pub description: String,
}

impl Revision {
    pub fn load(path: impl AsRef<Path>) -> Result<Revision, ZatsuError> {
        let serialized = match fs::read_to_string(path) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(error::CODE_LOADING_FILE_FAILED)),
        };
        let revision = match serde_json::from_str(&serialized) {
            Ok(revision) => revision,
            Err(_) => return Err(ZatsuError::new(error::CODE_DESERIALIZATION_FAILED)),
        };

        Ok(revision)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ZatsuError> {
        let serialized = match serde_json::to_string(self) {
            Ok(serialized) => serialized,
            Err(_) => return Err(ZatsuError::new(error::CODE_SERIALIZATION_FAILED)),
        };

        let _ = match std::fs::write(path, serialized) {
            Ok(result) => result,
            Err(_) => return Err(ZatsuError::new(error::CODE_SAVING_FILE_FAILED)),
        };

        Ok(())
    }
}
