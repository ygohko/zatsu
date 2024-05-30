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

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ZatsuError {
    pub domain: String,
    pub code: i32,
    pub details: String,
}

impl fmt::Display for ZatsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "Zatsu error. domain: {}, code: {}, details: {}", self.domain, self.code, self.details)
    }
}

impl Error for ZatsuError {
}

impl ZatsuError {
    pub fn new(domain: String, code: i32) -> ZatsuError {
	return ZatsuError {
	    domain: domain,
	    code: code,
	    details: "".to_string(),
	};
    }

    pub fn new_with_details(domain: String, code: i32, details: String) -> ZatsuError {
	return ZatsuError {
	    domain: domain,
	    code: code,
	    details: details,
	};
    }
}
