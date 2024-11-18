# zatsu: Experimental general purpose basic version control system

## About

zatsu is an experimental general purpose basic version control system. It has these concepts:

* Tool for commiting files only
* Does not have branches
* Does not communicate with other repositories
* Can remove old revisions to reduce repository size

## Usage

zatsu has these commands:

* init ... Initialize a repository into this directory
* commit ... Commit current files into this directory's repository
* log ... Show logs of this directory's repository
* get ... Get a file or directory that is specified
* forget ... Remove stored revisions to shrink this directory's repository to specified size
* upgrade ... Upgrade this repository
* help ... Print this message or the help of the given subcommand(s)

## How to build

Run the following command in the root directory of this project:

```
$ cargo build
```
