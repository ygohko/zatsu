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
mod repository;
mod revision;

use clap::Parser;
use clap::Subcommand;

use crate::command::Command;
use crate::commit_command::CommitCommand;
use crate::entry::Entry;
use crate::error::ZatsuError;
use crate::file_path_producer::FilePathProducer;
use crate::forget_command::ForgetCommand;
use crate::get_command::GetCommand;
use crate::init_command::InitCommand;
use crate::log_command::LogCommand;
use crate::repository::Repository;
use crate::revision::Revision;

#[derive(Parser)]
struct Arguments {
    /// Command you want to do
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
struct GetArguments {
    /// Revision to get a file or directory
    revision: i32,
    /// Path to get a file or directory
    path: String,
}

#[derive(Parser)]
struct ForgetArguments {
    /// Revision count to keep
    count: i32,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Commit,
    Log,
    Get(GetArguments),
    Forget(ForgetArguments),
}

fn main() -> Result<(), ZatsuError> {
    let arguments1 = Arguments::parse();
    let mut command = "commit".to_string();
    let mut revision_number = 0;
    let mut path = "".to_string();
    let mut revision_count = 0;
    if arguments1.command.is_some() {
        command = match arguments1.command.unwrap() {
            Commands::Init => "init".to_string(),
            Commands::Commit => "commit".to_string(),
            Commands::Log => "log".to_string(),
            Commands::Get(get_arguments) => {
                revision_number = get_arguments.revision;
                path = get_arguments.path;
                "get".to_string()
            },
            Commands::Forget(forget_arguments) => {
                revision_count = forget_arguments.count;
                "forget".to_string()
            },
        };
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
        let command = GetCommand::new(revision_number, &path);
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }
    if command == "forget" {
        let command = ForgetCommand::new(revision_count);
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
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
