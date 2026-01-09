# jx

[![crates.io](https://img.shields.io/crates/v/jx?logo=rust)](https://crates.io/crates/jx)

```sh
jx examples/reference.json                   # open a JSON file directly
curl example.com/some-json-endpoint | jx     # ...or pipe it in
```

<img width="1616" height="984" alt="image" src="https://github.com/user-attachments/assets/5bbd3376-a9fe-4d78-8903-4646b160eb81" />

## Installation

### Homebrew

```sh
brew install sqwxl/tap/jx
```

### Linux / macOS

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/sqwxl/jx/releases/latest/download/jx-installer.sh | sh
```

### Windows

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/sqwxl/jx/releases/latest/download/jx-installer.ps1 | iex"
```

### Install from source

```sh
cargo install --path .
```

## Usage

| Key                                  | Action                             |
| ------------------------------------ | ---------------------------------- |
| <kbd>?</kbd>                         | Show help.                         |
| <kbd>q</kbd> <kbd>Ctrl+c</kbd>       | Quit.                              |
| <kbd>j</kbd> <kbd>k</kbd>            | Next/Previous node.                |
| <kbd>h</kbd> <kbd>l</kbd>            | In/Out node.                       |
| <kbd>y</kbd> <kbd>e</kbd>            | Scroll one line up/down.           |
| <kbd>u</kbd> <kbd>d</kbd>            | Scroll half page up/down.          |
| <kbd>b</kbd> <kbd>f</kbd>            | Scroll full page up/down.          |
| <kbd>g</kbd>                         | Go to first line.                  |
| <kbd>G</kbd>                         | Go to last line.                   |
| <kbd><</kbd>                         | Scroll left.                       |
| <kbd>></kbd>                         | Scroll right.                      |
| <kbd>Space</kbd>                     | Toggle a fold.                     |
| <kbd>z</kbd>                         | Toggle all folds.                  |
| <kbd>/</kbd>                         | Search.                            |
| <kbd>n</kbd>                         | Repeat previous search.            |
| <kbd>N</kbd>                         | Repeat previous search in reverse. |
| <kbd>Esc</kbd>                       | Clear search.                      |
| <kbd>c</kbd> <kbd>Ctrl+Shift+C</kbd> | Copy the selection (pretty).       |
| <kbd>C</kbd>                         | Copy the value (pretty).           |
| <kbd>r</kbd>                         | Copy the selection (raw).          |
| <kbd>R</kbd>                         | Copy the value (raw).              |
| <kbd>Enter</kbd>                     | Output the selection (pretty).     |
| <kbd>Shift+Enter</kbd>               | Output the value (pretty).         |
| <kbd>o</kbd>                         | Output the selection (raw).        |
| <kbd>O</kbd>                         | Output the value (raw).            |
| <kbd>w</kbd>                         | Toggle line wrapping.              |
| <kbd>#</kbd>                         | Toggle line numbering.             |

## Features

- [x] Navigation.
- [x] Copy selection or value to clipboard.
- [x] Output selection or value to console.
- [x] Pretty-printing.
- [x] Searching.
- [x] Scrolling.
- [x] Horizontal scrolling.
- [x] Folding.
- [x] Syntax highlighting.
- [x] Line wrapping.
- [x] Line numbers.
- [ ] Multiple files.
- [ ] Broken files.
- [ ] Filtering.
- [ ] Sorting.
