// ved - the verbose ed, at the end of slice 2.
//
// The REPL of slice 1, plus a text buffer, an append command, and a
// second mode. Commands: a (append, ended by a lone `.`), p (print
// the current line), ,p (print the whole buffer), q/quit.

mod buffer;

use buffer::Buffer;
use std::env;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let prompt = match parse_args(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ved: {e}");
            eprintln!("try 'ved --help' for usage information");
            return ExitCode::from(2);
        }
    };

    run_repl(prompt.as_deref())
}

/// Parse command line arguments.
/// Returns the prompt string (if any) on success.
fn parse_args(args: &[String]) -> Result<Option<String>, String> {
    let mut prompt: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-p" | "--prompt" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("{arg} requires a value"));
                }
                prompt = Some(args[i].clone());
            }
            s if s.starts_with("--prompt=") => {
                prompt = Some(s["--prompt=".len()..].to_string());
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("ved {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
        i += 1;
    }
    Ok(prompt)
}

fn print_usage() {
    println!("ved - the verbose ed");
    println!();
    println!("Usage: ved [OPTIONS] [FILE]");
    println!();
    println!("Options:");
    println!("  -p, --prompt <STRING>   Set the command prompt");
    println!("  -h, --help              Show this help message and exit");
    println!("  -V, --version           Show version information and exit");
}

/// The main read-eval-print loop.
///
/// The loop has two modes. In *command mode* each input line is
/// parsed by `dispatch` and the resulting `Action` is applied. In
/// *input mode* (entered by `a`) every input line goes verbatim
/// into the buffer until the user types a single `.` on a line.
fn run_repl(prompt: Option<&str>) -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut reader = stdin.lock();
    let mut line = String::new();
    let mut buffer = Buffer::new();
    let mut input_mode = false;

    loop {
        // ed shows the prompt only in command mode, never in input
        // mode — input mode is meant to feel like raw typing.
        if !input_mode {
            if let Some(p) = prompt {
                if write!(out, "{p}").is_err() || out.flush().is_err() {
                    return ExitCode::FAILURE;
                }
            }
        }

        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return ExitCode::SUCCESS, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("ved: read error: {e}");
                return ExitCode::FAILURE;
            }
        }

        // Strip just the line terminator, not all trailing whitespace
        // — a user typing "  hello   " into the buffer should keep
        // their trailing spaces. trim_end() would eat them.
        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

        if input_mode {
            if trimmed == "." {
                input_mode = false;
            } else {
                buffer.append_after(buffer.current(), trimmed.to_string());
            }
            continue;
        }

        match dispatch(trimmed, &mut buffer) {
            Action::Quit => return ExitCode::SUCCESS,
            Action::EnterInputMode => input_mode = true,
            Action::Print(msg) => {
                let _ = writeln!(out, "{msg}");
            }
            Action::Error(msg) => {
                let _ = writeln!(out, "? {msg}");
            }
        }
    }
}

/// What the REPL should do after handling one input line.
enum Action {
    Quit,
    EnterInputMode,
    Print(String),
    Error(String),
}

/// Dispatch a single command line. Slice 2 knows four commands and
/// two shorthands; slice 3 replaces this match with a real parser.
fn dispatch(cmd: &str, buf: &mut Buffer) -> Action {
    let cmd = cmd.trim();

    // Bare Enter is shorthand for "advance to the next line and
    // print it". This is how you walk through a file in ed.
    if cmd.is_empty() {
        let next = buf.current() + 1;
        if next > buf.len() {
            return Action::Error("invalid address".to_string());
        }
        buf.set_current(next);
        return match buf.line(next) {
            Some(s) => Action::Print(s.to_string()),
            None => Action::Error("invalid address".to_string()),
        };
    }

    match cmd {
        "q" | "quit" => Action::Quit,
        "a" => Action::EnterInputMode,
        "p" => match buf.line(buf.current()) {
            Some(s) => Action::Print(s.to_string()),
            None => Action::Error("invalid address".to_string()),
        },
        ",p" => {
            if buf.is_empty() {
                return Action::Error("invalid address".to_string());
            }
            let mut output = String::new();
            for n in 1..=buf.len() {
                if n > 1 {
                    output.push('\n');
                }
                if let Some(line) = buf.line(n) {
                    output.push_str(line);
                }
            }
            buf.set_current(buf.len());
            Action::Print(output)
        }
        other => Action::Error(format!("unknown command: {other}")),
    }
}
