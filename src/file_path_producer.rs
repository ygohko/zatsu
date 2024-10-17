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

use std::env::consts;
use std::fs;
use std::path::Path;

use crate::error;
use crate::error::ZatsuError;

pub struct FilePathProducer {
    file_paths: Vec<String>,
    directory_paths: Vec<String>,
    prefix_length: usize,
}

impl FilePathProducer {
    pub fn new(path: String) -> FilePathProducer {
        let prefix_length = path.len() + 1;
        return FilePathProducer {
            file_paths: Vec::new(),
            directory_paths: vec![path],
            prefix_length: prefix_length,
        };
    }

    pub fn next(&mut self) -> Result<String, ZatsuError> {
        let done = false;
        while !done {
            if self.file_paths.len() > 0 {
                let mut path = self.file_paths.pop().unwrap();
                if consts::OS == "windows" {
                    path = path.replace("\\", "/");
                }

                return Ok(path);
            }

            if self.directory_paths.len() == 0 {
                return Err(ZatsuError::new(error::CODE_PRODUCING_FINISHED));
            }
            let directory_path = self.directory_paths.pop().unwrap();

            let mut scan = true;
            let option = Path::new(&directory_path).file_name();
            if option.is_some() {
                let file_name = option.unwrap().to_string_lossy().to_string();
                if file_name == ".zatsu".to_string()
                    || file_name == ".jj".to_string()
                    || file_name == ".git".to_string()
                {
                    scan = false;
                }
            }

            if scan {
                let read_dir = match fs::read_dir(directory_path) {
                    Ok(read_dir) => read_dir,
                    Err(_) => return Err(ZatsuError::new(error::CODE_READING_DIRECTORY_FAILED)),
                };
                for result in read_dir {
                    if result.is_ok() {
                        let entry = result.unwrap();

                        let metadata = match fs::metadata(entry.path()) {
                            Ok(metadata) => metadata,
                            Err(_) => {
                                return Err(ZatsuError::new(error::CODE_READING_META_DATA_FAILED))
                            }
                        };
                        let path = entry.path().to_string_lossy().to_string();
                        if metadata.is_file() {
                            let path = path[self.prefix_length..].to_string();
                            self.file_paths.push(path);
                        } else {
                            self.directory_paths.push(path);
                        }
                    }
                }
            }
        }

        Err(ZatsuError::new(error::CODE_PRODUCING_FINISHED))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_creatable() {
        let producer = FilePathProducer::new(".".to_string());
        assert_eq!(0, producer.file_paths.len());
        assert_eq!(1, producer.directory_paths.len());
        assert_eq!(2, producer.prefix_length);
    }
}
