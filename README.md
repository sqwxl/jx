# jx

![Crates.io](https://img.shields.io/crates/v/jx?logo=rust)

```sh
jx example.json                              # open a JSON file directly
curl example.com/some-json-endpoint | jx     # ...or pipe it in
```

## Installation

`cargo install jx`

## Usage

| Key | Action ||
| --- | ------ | - |
| `hjkl` or Arrows | Navigate the JSON structure. | [x] |
| `Space` | Toggle a fold. | [ ] |
| `y` | Copy the selection (pretty). | [x] |
| `Shift+Y` | Copy the value (pretty). | [x] |
| `r` | Copy the selection (raw). | [x] |
| `Shift+R` | Copy the value (raw). | [x] |
| `Enter` | Output the selection (pretty) to the console and quit. | [x] |
| `Shift+Enter` | Output the value (pretty) to the console and quit. | [ ] |
| `o` | Output the selection (raw) to the console and quit. | [x] |
| `Shift+O` | Output the value (row) to the console and quit. | [x] |
| `u` `d` | Scroll half page up/down. | [ ] |
| `b` `f` | Scroll full page up/down. | [ ] |
| `q`, `Escape` or `^C` | Quit. | [x] |
