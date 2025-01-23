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
curl example.com/some-json-endpoint | jx     # ...or pipe it in
```

## Installation

### Cargo

`cargo install jx`

## Usage

- Use the arrow keys or 'hjkl' to navigate the JSON structure.
- 'Space' to toggle a fold.
- 'Enter' to copy the selection to the clipboard.
- 'q', 'Escape' or '^C' to quit.

## Tests

`cargo test`
