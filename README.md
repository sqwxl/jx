# jx

![Crates.io](https://img.shields.io/crates/v/jx?logo=rust)

```sh
jx example.json                              # open a JSON file directly
curl example.com/some-json-endpoint | jx     # ...or pipe it in
```

## Installation

`cargo install jx`

## Usage

| Key | Action |
| --- | ------ |
| `hjkl` or Arrows | Navigate the JSON structure. |
| `Space` | Toggle a fold. |
| `y` | Copy the selection (pretty). |
| `r` | Copy the selection (raw). |
| `Enter` | Output the selection to the console and quit. |
| `u` `d` | Scroll half page up/down. |
| `b` `f` | Scroll full page up/down. |
| `q`, `Escape` or `^C` | Quit. |
