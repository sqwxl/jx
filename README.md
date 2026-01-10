# jx

[![crates.io](https://img.shields.io/crates/v/jx?logo=rust)](https://crates.io/crates/jx)

```sh
jx examples/reference.json                   # open a JSON file directly
curl example.com/some-json-endpoint | jx     # ...or pipe it in
```

<img width="2238" height="1198" alt="image" src="https://github.com/user-attachments/assets/74e95f1b-a125-4e37-b108-9df909cb8512" />

## Installation

### Homebrew

```sh
brew install sqwxl/tap/jx
```

### Linux / macOS

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/sqwxl/jx/releases/latest/download/jx-installer.sh | sh
```

### Install from source

```sh
cargo install --path .
```

## Usage

| Key                                                 | Action                              |
| --------------------------------------------------- | ----------------------------------- |
| <kbd>?</kbd>                                        | Show help                           |
| <kbd>q</kbd><kbd>C-c</kbd>                          | Quit                                |
| <kbd>j</kbd><kbd>↓</kbd> / <kbd>k</kbd><kbd>↑</kbd> | Next/Previous                       |
| <kbd>h</kbd><kbd>←</kbd> / <kbd>l</kbd><kbd>→</kbd> | In/Out                              |
| <kbd>C-y</kbd> / <kbd>C-e</kbd>                     | Scroll line up/down                 |
| <kbd>u</kbd> / <kbd>d</kbd>                         | Scroll half page up/down            |
| <kbd>b</kbd> / <kbd>f</kbd>                         | Scroll full page up/down            |
| <kbd>g</kbd> / <kbd>G</kbd>                         | Go to top/bottom                    |
| <kbd><</kbd> / <kbd>></kbd>                         | Scroll left/right                   |
| <kbd>Space</kbd><kbd>Enter</kbd>                    | Toggle a fold                       |
| <kbd>z</kbd>                                        | Toggle all folds                    |
| <kbd>/</kbd>                                        | Search                              |
| <kbd>n</kbd> / <kbd>N</kbd>                         | Go to next/previous search match    |
| <kbd>Esc</kbd>                                      | Clear search                        |
| <kbd>y</kbd> / <kbd>Y</kbd>                         | Copy the selection/value (pretty)   |
| <kbd>A-y</kbd> / <kbd>A-Y</kbd>                     | Copy the selection/value (raw)      |
| <kbd>o</kbd> / <kbd>O</kbd>                         | Output the selection/value (pretty) |
| <kbd>A-o</kbd> / <kbd>A-O</kbd>                     | Output the selection/value (raw)    |
| <kbd>#</kbd>                                        | Toggle line numbering               |
| <kbd>w</kbd>                                        | Toggle line wrapping                |
