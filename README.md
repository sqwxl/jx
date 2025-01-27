# jx

![Crates.io](https://img.shields.io/crates/v/jx?logo=rust)

```sh
jx example.json                              # open a JSON file directly
curl example.com/some-json-endpoint | jx     # ...or pipe it in
```

## Installation

`cargo install jx`

## Usage

| Key                                  | Action                             |
| ------------------------------------ | ---------------------------------- |
| <kbd>H</kbd>                         | Show help.                         |
| <kbd>q</kbd> <kbd>Ctrl+c</kbd>       | Quit.                              |
| <kbd>j</kbd> <kbd>k</kbd>            | Next/Previous node.                |
| <kbd>h</kbd> <kbd>l</kbd>            | In/Out node.                       |
| <kbd>y</kbd> <kbd>e</kbd>            | Scroll one line up/down.           |
| <kbd>u</kbd> <kbd>d</kbd>            | Scroll half page up/down.          |
| <kbd>b</kbd> <kbd>f</kbd>            | Scroll full page up/down.          |
| <kbd>g</kbd>                         | Go to first line.                  |
| <kbd>G</kbd>                         | Go to last line.                   |
| <kbd>Space</kbd>                     | Toggle a fold.                     |
| <kbd>s</kbd>                         | Sort selected.                     |
| <kbd>S</kbd>                         | Sort selected reversed.            |
| <kbd>/</kbd>                         | Search.                            |
| <kbd>?</kbd>                         | Search backward.                   |
| <kbd>n</kbd>                         | Repeat previous search.            |
| <kbd>N</kbd>                         | Repeat previous search in reverse. |
| <kbd>&</kbd>                         | Filter.                            |
| <kbd>Esc</kbd>                       | Clear search/filter.               |
| <kbd>c</kbd> <kbd>Ctrl+Shift+C</kbd> | Copy the selection (pretty).       |
| <kbd>C</kbd>                         | Copy the value (pretty).           |
| <kbd>r</kbd>                         | Copy the selection (raw).          |
| <kbd>R</kbd>                         | Copy the value (raw).              |
| <kbd>Enter</kbd>                     | Output the selection (pretty).     |
| <kbd>Shift+Enter</kbd>               | Output the value (pretty).         |
| <kbd>o</kbd>                         | Output the selection (raw).        |
| <kbd>O</kbd>                         | Output the value (raw).            |
| <kbd>#</kbd>                         | Toggle line numbers.               |
| <kbd>w</kbd>                         | Toggle line wrapping.              |

## Features

- [x] Navigation.
- [x] Copy selection or value to clipboard.
- [x] Output selection or value to console.
- [x] Pretty-printing.
- [ ] Searching.
- [ ] Filtering.
- [ ] Sorting.
- [ ] Scrolling.
- [ ] Folding.
- [ ] Syntax highlighting.
- [ ] Multiple files.
- [ ] Broken files.
- [ ] Line numbers.
- [ ] Line wrapping.
