use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::{Command, Stdio};

const REDACTION: &str = "<redacted>";
const USAGE: &str = "Usage: slimshady < input\n\nReads environment text from stdin and copies KEY=<redacted> lines to the clipboard.\n";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => {}
        [arg] if arg == "--help" || arg == "-h" => {
            print!("{USAGE}");
            return;
        }
        _ => {
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    }

    let interactive_input = io::stdin().is_terminal();
    let interactive_output = io::stdout().is_terminal();

    if interactive_input {
        eprintln!("Paste env text, then press Ctrl-D to copy redacted output to the clipboard.");
    }

    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("slimshady: failed to read stdin: {err}");
        std::process::exit(1);
    }

    let output = strip_env(&input);

    if interactive_output {
        if let Err(err) = copy_to_clipboard(&output) {
            eprintln!("slimshady: failed to copy to clipboard: {err}");
            std::process::exit(1);
        }

        eprintln!("Copied redacted env to clipboard.");
    } else {
        print!("{output}");
    }
}

fn strip_env(input: &str) -> String {
    parse_env_keys(input)
        .map(|key| format!("{key}={REDACTION}\n"))
        .collect()
}

fn parse_env_keys(input: &str) -> impl Iterator<Item = &str> {
    input.lines().filter_map(parse_env_key)
}

fn copy_to_clipboard(output: &str) -> io::Result<()> {
    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "failed to open pbcopy stdin"))?;
    stdin.write_all(output.as_bytes())?;
    drop(stdin);

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("pbcopy exited with {status}")))
    }
}

fn parse_env_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let assignment = trimmed
        .strip_prefix("export ")
        .map(str::trim_start)
        .unwrap_or(trimmed);
    let (key, _value) = assignment.split_once('=')?;

    if is_valid_env_key(key) {
        Some(key)
    } else {
        None
    }
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();

    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(input: &str) -> Vec<&str> {
        parse_env_keys(input).collect()
    }

    #[test]
    fn strips_env_values() {
        assert_eq!(
            strip_env("A=1\nB=two=three\n"),
            "A=<redacted>\nB=<redacted>\n"
        );
    }

    #[test]
    fn parses_plain_assignment() {
        assert_eq!(keys("OPENAI_API_KEY=sk-abc123\n"), vec!["OPENAI_API_KEY"]);
    }

    #[test]
    fn parses_export_assignment() {
        assert_eq!(
            keys("export DATABASE_URL=postgres://secret\n"),
            vec!["DATABASE_URL"]
        );
    }

    #[test]
    fn parses_quoted_values() {
        assert_eq!(
            keys("DOUBLE=\"secret\"\nSINGLE='secret'\n"),
            vec!["DOUBLE", "SINGLE"]
        );
    }

    #[test]
    fn parses_values_containing_equals() {
        assert_eq!(keys("B=two=three\n"), vec!["B"]);
    }

    #[test]
    fn ignores_blank_lines_and_comments() {
        assert_eq!(keys("\n  \n# comment\n   # comment\nA=1\n"), vec!["A"]);
    }

    #[test]
    fn ignores_invalid_keys() {
        assert_eq!(
            keys("1BAD=value\nBAD-KEY=value\nBAD.KEY=value\nGOOD_KEY=value\n"),
            vec!["GOOD_KEY"]
        );
    }

    #[test]
    fn skips_malformed_non_env_lines() {
        assert_eq!(keys("not an env line\nexport nope\nA=1\n"), vec!["A"]);
    }

    #[test]
    fn trims_space_around_supported_syntax() {
        assert_eq!(keys("  export   A=1\n"), vec!["A"]);
    }

    #[test]
    fn rejects_space_before_equals() {
        assert_eq!(keys("A =1\nexport B =2\n"), Vec::<&str>::new());
    }
}
