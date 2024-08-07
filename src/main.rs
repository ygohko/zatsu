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
mod init_command;
mod log_command;
mod revision;
mod repository;

use std::env;

use crate::command::Command;
use crate::commit_command::CommitCommand;
use crate::entry::Entry;
use crate::error::ZatsuError;
use crate::file_path_producer::FilePathProducer;
use crate::forget_command::ForgetCommand;
use crate::get_command::GetCommand;
use crate::init_command::InitCommand;
use crate::log_command::LogCommand;
use crate::revision::Revision;
use crate::repository::Repository;

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
	let command = InitCommand::new();
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }

    Ok(())
}
