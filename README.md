<div align="center">
  
# jx

![Crates.io](https://img.shields.io/crates/v/jx?logo=rust) 
![Downloads](https://img.shields.io/github/downloads/sqwxl/jx/total?logo=github)

jx is an interactive JSON explorer for the command line.

[Getting started](#getting-started) •
[Installation](#installation) •
[Usage](#usage) •
[Tests](#tests)

</div>

## Getting started

```sh
jx example.json                              # open a JSON file directly
cat example.json | jx                        # ...or pipe it in
curl example.com/some-json-endpoint | jx     # ...from anywhere
jx example.json -o selection.json             # ...write your selection to a file (can also be acheive via the ui)
```

## Installation

1. **Install binary**

   [ ] *TODO* release binary on various platforms

   `cargo install jx`
   
   For now you can manually download the latest version from the [releases](github.com/sqwxl/jx/releases) page.

## Usage

TODO

## Tests

`cargo test`

   
