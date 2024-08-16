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

#[allow(dead_code)]
pub const CODE_GENERAL: i32 = 0;
pub const CODE_READING_META_DATA_FAILED: i32 = 1;
pub const CODE_READING_DIRECTORY_FAILED: i32 = 2;
pub const CODE_CREATING_REPOSITORY_FAILED: i32 = 3;
pub const CODE_LOADING_REPOSITORY_FAILED: i32 = 4;
pub const CODE_REVISION_NOT_FOUND: i32 = 5;
pub const CODE_LOADING_REVISION_FAILED: i32 = 6;
pub const CODE_FILE_NOT_FOUND: i32 = 7;
pub const CODE_LOADING_FILE_FAILED: i32 = 8;
pub const CODE_SAVING_FILE_FAILED: i32 = 9;
pub const CODE_PRODUCING_FINISHED: i32 = 10;
pub const CODE_CREATING_DIRECTORY_FAILED: i32 = 11;

#[derive(Debug)]
pub struct ZatsuError {
    // TODO: Remove domain field.
    pub domain: String,
    pub code: i32,
    pub backtrace: String,
    pub details: String,
}

impl fmt::Display for ZatsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Zatsu error. domain: {}, code: {}, backtrace: {}, details: {}",
            self.domain, self.code, self.backtrace, self.details
        )
    }
}

impl Error for ZatsuError {}

impl ZatsuError {
    pub fn new(domain: String, code: i32) -> ZatsuError {
        let backtrace = Backtrace::capture();
        let string = format!("{}", backtrace);
        return ZatsuError {
            domain: domain,
            code: code,
            backtrace: string,
            details: "".to_string(),
        };
    }

    #[allow(dead_code)]
    pub fn new_with_details(domain: String, code: i32, details: String) -> ZatsuError {
        let backtrace = Backtrace::capture();
        let string = format!("{}", backtrace);
        return ZatsuError {
            domain: domain,
            code: code,
            backtrace: string,
            details: details,
        };
    }
}
