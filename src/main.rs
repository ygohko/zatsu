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

mod command;
mod commit_command;
mod entry;
mod error;
mod file_path_producer;
mod forget_command;
mod get_command;
mod log_command;
mod revision;
mod repository;

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::command::Command;
use crate::commit_command::CommitCommand;
use crate::entry::Entry;
use crate::error::ZatsuError;
use crate::file_path_producer::FilePathProducer;
use crate::forget_command::ForgetCommand;
use crate::get_command::GetCommand;
use crate::log_command::LogCommand;
use crate::revision::Revision;
use crate::repository::Repository;

#[allow(dead_code)]
const ERROR_GENERAL: i32 = 0;
const ERROR_READING_META_DATA_FAILED: i32 = 1;
const ERROR_READING_DIRECTORY_FAILED: i32 = 2;
const ERROR_CREATING_REPOSITORY_FAILED: i32 = 3;
const ERROR_LOADING_REPOSITORY_FAILED: i32 = 4;
const ERROR_REVISION_NOT_FOUND: i32 = 5;
const ERROR_LOADING_REVISION_FAILED: i32 = 6;
const ERROR_FILE_NOT_FOUND: i32 = 7;
const ERROR_LOADING_FILE_FAILED: i32 = 8;
const ERROR_SAVING_FILE_FAILED: i32 = 9;
const ERROR_PRODUCING_FINISHED: i32 = 10;

fn main() -> Result<(), ZatsuError> {
    println!("Hello, world!");

    let arguments: Vec<_> = env::args().collect();
    let count = arguments.len();
    println!("count: {}", count);
    if count > 0 {
        println!("arguments[0]: {}", arguments[0]);

    }
    if count > 1 {
        println!("arguments[1]: {}", arguments[1]);
    }

    let mut command = "commit".to_string();
    if count > 1 {
        command = arguments[1].clone();
    }

    if command == "commit" {
	let command = CommitCommand::new();
	match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }
    if command == "log" {
	let command = LogCommand::new();
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }
    if command == "get" {
        if count > 3 {
            // TODO: Do not panic is parse failed.
            let revision_number :i32 = arguments[2].parse().unwrap();
            let path = arguments[3].clone();
	    let command = GetCommand::new(revision_number, &path);
            match command.execute() {
                Ok(()) => (),
                Err(error) => return Err(error),
            };
        }
    }
    if command == "forget" {
        if count > 2 {
            // TODO: Do not panic is parse failed.
            let revision_count :i32 = arguments[2].parse().unwrap();
	    let command = ForgetCommand::new(revision_count);
            match command.execute() {
                Ok(()) => (),
                Err(error) => return Err(error),
            };
        }
    }
    if command == "init" {
        match process_init() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }

    Ok(())
}

fn process_init() -> Result<(), ZatsuError> {
    match fs::create_dir_all(".zatsu") {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    match fs::write(".zatsu/version.txt", "1") {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    match fs::create_dir_all(".zatsu/revisions") {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    match fs::create_dir_all(".zatsu/objects") {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_CREATING_REPOSITORY_FAILED)),
    };
    let repository = Repository {
        revision_numbers: Vec::new(),
    };
    match repository.save(&PathBuf::from(".zatsu/repository.json")) {
        Ok(()) => (),
        Err(_) => return Err(ZatsuError::new("main".to_string(), ERROR_SAVING_FILE_FAILED)),
    };

    Ok(())
}
