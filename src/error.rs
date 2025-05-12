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

use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;

pub type ErrorId = &'static str;
pub type ErrorCode = i32;

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
// pub const CODE_READING_META_DATA_FAILED: ErrorCode = 1;
// pub const CODE_READING_DIRECTORY_FAILED: ErrorCode = 2;
// pub const CODE_CREATING_REPOSITORY_FAILED: ErrorCode = 3;
// pub const CODE_LOADING_REPOSITORY_FAILED: ErrorCode = 4;
// pub const CODE_REVISION_NOT_FOUND: ErrorCode = 5;
// pub const CODE_LOADING_REVISION_FAILED: ErrorCode = 6;
// pub const CODE_FILE_NOT_FOUND: ErrorCode = 7;
// pub const CODE_LOADING_FILE_FAILED: ErrorCode = 8;
pub const CODE_SAVING_FILE_FAILED: ErrorCode = 9;
pub const CODE_PRODUCING_FINISHED: ErrorCode = 10;
pub const CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 11;
pub const CODE_DESERIALIZATION_FAILED: ErrorCode = 12;
pub const CODE_SERIALIZATION_FAILED: ErrorCode = 13;
pub const CODE_REMOVING_FILE_FAILED: ErrorCode = 14;
pub const CODE_REMOVING_DIRECTORY_FAILED: ErrorCode = 15;

#[derive(Debug)]
pub struct ZatsuError {
    pub id: ErrorId,
    pub code: ErrorCode,
    pub backtrace: String,
    pub details: String,
}

impl fmt::Display for ZatsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Zatsu error. id: {}, code: {}, backtrace: {}, details: {}",
            self.id, self.code, self.backtrace, self.details
        )
    }
}

impl Error for ZatsuError {}

impl ZatsuError {
    pub fn new(id: ErrorId, code: ErrorCode) -> ZatsuError {
        let backtrace = Backtrace::capture();
        let string = format!("{}", backtrace);
        return ZatsuError {
            id: id,
            code: code,
            backtrace: string,
            details: "".to_string(),
        };
    }

    #[allow(dead_code)]
    pub fn with_details(id: ErrorId, code: ErrorCode, details: String) -> ZatsuError {
        let backtrace = Backtrace::capture();
        let string = format!("{}", backtrace);
        return ZatsuError {
            id: id,
            code: code,
            backtrace: string,
            details: details,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_creatable() {
        let error = ZatsuError::new("test", 123);
        assert_eq!("test", error.id);
        assert_eq!(123, error.code);
        assert_eq!("disabled backtrace".to_string(), error.backtrace);
        assert_eq!("".to_string(), error.details);

        let error = ZatsuError::with_details("test2", 456, "details".to_string());
        assert_eq!("test2", error.id);
        assert_eq!(456, error.code);
        assert_eq!("disabled backtrace".to_string(), error.backtrace);
        assert_eq!("details".to_string(), error.details);
    }

    #[test]
    fn is_formattable() {
        let error = ZatsuError::with_details("test3", 789, "details".to_string());
        let formatted = format!("{}", error);
        assert_eq!(
            "Zatsu error. id: test3, code: 789, backtrace: disabled backtrace, details: details",
            formatted
        );
    }
}
