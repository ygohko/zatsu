# zatsu: Experimental general purpose tiny verision control system

## About

zatsu is experimental general purpose tiny version control system. This has these concepts:

* Tool for just commiting files
* Do not have branches
* Do not communicate with other repositories
* Can remove old revisions to reduce repository size

## Usage

zatsu has these commands:

* init ... Initialize a repository into this directory
* commit ... Commit current files into this direcrory's repository
* log ... Show logs of this directory's repository
* get ... Get a file or direcrory that is specified
* forget ... Remove stored revisions to shrink this directory's repository to specified size
* upgrade ... Upgrade this repository
* help ... Print this message or the help of the given subcommand(s)

## How to build

Execute a command below at root of this project:

```
$ cargo build
```
