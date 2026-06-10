# slimshady

Tiny Rust CLI for stripping environment variable values before sharing env text with an LLM.

`slimshady` reads environment text from stdin and replaces every recognized value with `<redacted>`.

## Install

```sh
cargo install --path .
```

Or build a local release binary:

```sh
cargo build --release
```

## Usage

Run it, paste env text, then press `Ctrl-D`. The redacted result is copied to your clipboard.

```sh
slimshady
```

You can also pipe input in. When output is a terminal, the result is copied to the clipboard:

```sh
env | slimshady
pbpaste | slimshady
```

When output is redirected or piped, `slimshady` writes redacted text to stdout:

```sh
env | slimshady > redacted-env.txt
env | slimshady | cat
```

During development:

```sh
printf 'A=1\nB=two=three\n' | cargo run --quiet
```

Output:

```text
A=<redacted>
B=<redacted>
```

## Behavior

Recognized input:

- `KEY=value`
- `export KEY=value`
- `KEY="secret"` or `KEY='secret'`
- Values containing `=`

Only shell-style variable names are printed: `[A-Za-z_][A-Za-z0-9_]*`.

Blank lines, comment-only lines, malformed lines, and invalid variable names are skipped silently. Values are never preserved, hashed, truncated, typed, or length-revealed.
