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
mod commons;
mod entry;
mod error;
mod file_path_producer;
mod forget_command;
mod get_command;
mod init_command;
mod log_command;
mod repository;
mod revision;
mod upgrade_command;

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
use crate::upgrade_command::UpgradeCommand;

#[derive(Parser)]
struct Arguments {
    /// Command you want to do
    #[command(subcommand)]
    command: Option<CommandKind>,
}

#[derive(Parser, PartialEq)]
struct InitArguments {
    /// Repository version to be created.
    #[arg(short, long)]
    version: Option<i32>,
}

#[derive(Parser, PartialEq)]
struct GetArguments {
    /// Revision to get a file or directory
    revision: i32,
    /// Path to get a file or directory
    path: String,
}

#[derive(Parser, PartialEq)]
struct ForgetArguments {
    /// Revision count to keep
    count: i32,
}

#[derive(Subcommand, PartialEq)]
enum CommandKind {
    /// Initialize a repository into this directory
    Init(InitArguments),
    /// Commit current files into this direcrory's repository
    Commit,
    /// Show logs of this directory's repository
    Log,
    /// Get a file or direcrory that is specified
    Get(GetArguments),
    /// Remove stored revisions to shrink this directory's repository to specified size
    Forget(ForgetArguments),
    /// Upgrade this repository
    Upgrade,
}

fn main() -> Result<(), ZatsuError> {
    let arguments = Arguments::parse();
    let mut command = CommandKind::Commit;
    if arguments.command.is_some() {
        command = arguments.command.unwrap();
    }

    if command == CommandKind::Commit {
        let command = CommitCommand::new();
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    } else if command == CommandKind::Log {
        let command = LogCommand::new();
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    } else if let CommandKind::Get(arguments) = command {
        let command = GetCommand::new(arguments.revision, &arguments.path);
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    } else if let CommandKind::Forget(arguments) = command {
        let command = ForgetCommand::new(arguments.count);
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    } else if let CommandKind::Init(arguments) = command {
        let version: i32;
        if arguments.version.is_some() {
            version = arguments.version.unwrap();
        }
        else {
            version = 1
        }
        let command = InitCommand::new(version);
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    } else if command == CommandKind::Upgrade {
        let command = UpgradeCommand::new();
        match command.execute() {
            Ok(()) => (),
            Err(error) => return Err(error),
        };
    }

    Ok(())
}
