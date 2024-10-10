# zatsu: Experimental general purpose tiny version control system

## About

zatsu is an experimental general purpose tiny version control system. It has these concepts:

* Tool for commiting files only
* Does not have branches
* Does not communicate with other repositories
* Can remove old revisions to reduce repository size

## Usage

zatsu has these commands:

* init ... Initialize a repository into this directory
* commit ... Commit current files into this direcory's repository
* log ... Show logs of this directory's repository
* get ... Get a file or direcory that is specified
* forget ... Remove stored revisions to shrink this directory's repository to specified size
* upgrade ... Upgrade this repository
* help ... Print this message or the help of the given subcommand(s)

## How to build

Run the following command in the root directory of this project:

```
$ cargo build
```
