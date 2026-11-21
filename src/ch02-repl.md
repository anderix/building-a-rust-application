# 2. Slice One: A Program That Runs

Slice one is a hundred and twenty lines and does almost nothing. It accepts a
flag, prints a prompt, reads lines, replies "unknown command" to all of them,
and quits when told to. It is the whole skeleton of the finished editor, and
every later slice hangs off it.

```text
$ cargo run -q -p ch02-repl -- -p '* '
* hello
? unknown command: hello
* q
$
```

## 2.1 `main` returns an exit code

```rust,ignore
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
```

A command-line program has two outputs: what it prints, and the number it
returns. The second one is invisible until something depends on it, and then it
is the only thing that matters — a shell `&&`, a `Makefile` rule, a CI job, and
`git` deciding whether your commit message editor succeeded all read the exit
status and nothing else.

Rust offers three ways to produce one. `main` can return `()` and the program
exits 0. It can return `Result<(), E>`, and on `Err` the runtime prints the
error with its `Debug` formatting and exits 1. Or it can return `ExitCode`,
which is the choice here, because ved wants three distinct statuses and does not
want `Debug` output shown to a user.

The convention this program follows is the usual Unix one: 0 for success, 2 for
a usage error, 1 for a runtime failure. `ExitCode::SUCCESS` and
`ExitCode::FAILURE` cover the first and last, and `ExitCode::from(2)` covers the
middle.

The usage error also demonstrates the pattern for the rest of the program.
Diagnostics go to standard error with `eprintln!`, prefixed with the program's
name, lower case, no trailing period. The second line points at `--help` rather
than reprinting the help, because a user who mistyped a flag wants to see their
mistake, not eighty lines of reference:

```text
$ cargo run -q -p ch02-repl -- -x
ved: unknown argument: -x
try 'ved --help' for usage information
$ echo $?
2
```

`env::args()` yields the program's own path as its first item, which `.skip(1)`
discards. Collecting into a `Vec<String>` costs an allocation per argument and
makes the parser below a plain loop over a slice, which is worth it.

## 2.2 Parsing arguments by hand

```rust,ignore
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
```

This is thirty lines that `clap` would do in six, and for most programs `clap`
is the right answer. Writing it by hand once is still worth an hour, because
every argument parser makes the same four decisions and a generated one hides
them from you.

**A flag that takes a value has two spellings.** `-p '* '` and `--prompt='* '`
are both idiomatic and users type both. The `while` loop with a manual index
exists so the `-p` arm can consume the *next* argument; an iterator works
equally well if you keep the iterator in scope, which is what Chapter 10 of
*Programming in Rust* does. The `--prompt=` form is a separate match arm with a
guard, because it arrives as one argument rather than two.

**The value might be missing.** `ved -p` with nothing after it has to produce
`-p requires a value` rather than a panic. The bounds check is the whole fix and
it is the single most commonly skipped line in hand-written parsers.

**`--help` and `--version` exit immediately and successfully.** They are not
errors, they do not fall through to the rest of the program, and they exit 0.
`std::process::exit(0)` inside the parser is a deliberate shortcut: it skips
every destructor between here and `main`, which at this point in the program is
nothing.

**The version comes from the manifest.** `env!("CARGO_PKG_VERSION")` reads
`Cargo.toml`'s `version` field at compile time. A version string typed into the
source is a version string that will be wrong after the next release, and the
only thing worse than no `--version` is one that lies.

The final `other =>` arm rejects everything unrecognized. That is deliberately
strict, and it is what the finished ved changes first: by slice 11 a bare word
is a filename, not an error, and the arm becomes a `s if s.starts_with('-')`
guard so that unknown *flags* still fail while unknown *words* are accepted.
Start strict; loosening a parser later is easy, and tightening one is a breaking
change.

```rust,ignore
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
```

Usage text goes to standard output, because the user asked for it, and it is
literally a series of `println!` calls. The `[FILE]` in the usage line is a
promise slice 11 will keep.

## 2.3 The loop

```rust,ignore
fn run_repl(prompt: Option<&str>) -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        if let Some(p) = prompt {
            if write!(out, "{p}").is_err() || out.flush().is_err() {
                return ExitCode::FAILURE;
            }
        }

        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return ExitCode::SUCCESS, // EOF
            Ok(_) => match dispatch(line.trim_end()) {
                Action::Continue => {}
                Action::Quit => return ExitCode::SUCCESS,
                Action::Print(msg) => {
                    let _ = writeln!(out, "{msg}");
                }
            },
            Err(e) => {
                eprintln!("ved: read error: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
}
```

Five things in that loop are doing specific work.

**The handles are locked once.** `io::stdin()` and `io::stdout()` return handles
wrapped in a mutex, and every `println!` acquires and releases it. Taking the
lock once, outside the loop, is both faster and — more usefully — gives a
`StdinLock` that implements `BufRead`, which is where `read_line` comes from.

**The `String` is reused.** `line.clear()` at the top of each iteration keeps the
allocation and drops the contents. `read_line` appends rather than replacing,
so forgetting the `clear()` produces a buffer that accumulates the entire
session, which is a genuinely confusing bug to read.

**`Ok(0)` is end of input, not an error.** `read_line` returns the number of
bytes read, and zero bytes means the stream closed: the user pressed Ctrl-D, or
the script being piped in ran out. Exiting successfully on EOF is what makes
`printf 'a\nhello\n.\nw out.txt\nq\n' | ved` work, and the ability to drive the
editor from a pipe is half of what makes it testable.

**The prompt is written, not printed, and then flushed.** Standard output is
line-buffered, and a prompt has no newline, so without the explicit `flush()`
the user stares at a blank line while their prompt sits in a buffer. This is the
most common bug in first REPLs. The `is_err()` checks turn a closed stdout into
a clean failure exit rather than a panic.

**Reading fails separately from reaching the end.** `Err(e)` is a real I/O
error, and it exits 1 with a message on stderr.

## 2.4 Why `dispatch` returns instead of acts

```rust,ignore
/// What the REPL should do after handling one input line.
/// Slice 2 will grow this to include buffer mutations.
enum Action {
    Continue,
    Quit,
    Print(String),
}

fn dispatch(cmd: &str) -> Action {
    match cmd {
        "" => Action::Continue,
        "q" | "quit" => Action::Quit,
        other => Action::Print(format!("? unknown command: {other}")),
    }
}
```

`dispatch` could have printed the message itself and returned a `bool` for
"should we keep going". Returning a description of what should happen instead
costs an allocation per message and is the single most consequential design
decision in the program.

It means `dispatch` is a pure function from a string to a value. It does not
touch the terminal, so it can be called from a test with no I/O setup at all,
and the test asserts on a returned value rather than on captured output.
Chapter 15 shows what that is worth: ved's entire command-language test suite
calls `dispatch` directly.

It also means there is exactly one place in the program that writes to the
terminal, so a decision about output — a prefix on errors, a quiet flag,
whether confirmations go to stdout or stderr — is made once, in the loop,
rather than in forty command handlers.

The enum will grow. By the end it has six variants, one of which switches the
loop into a second mode. The shape does not change.

Three things about this enum are worth noticing now. `Action::Print(String)`
carries an owned `String` rather than a `&str`, because the message is usually
constructed rather than borrowed from the input. There is no `Action::Error`
yet, so slice 1 formats the `?` prefix into the message by hand; the next slice
splits the prefix out into its own variant, and the plan gave slice 9 the job of
rewriting the text that goes inside it, which Chapter 14 comes back to. And `Continue` exists so that a bare
Enter is a command that does nothing, rather than a command that fails.

## 2.5 What slice one is for

A hundred and twenty lines that reply "unknown command" to everything is not an
achievement. What it is, is a program: it builds, it runs, it accepts input, it
has an exit status, and it can be piped into. Every subsequent slice is an edit
to something that already works.

Compare the alternative, where the first day produces a `Buffer` struct with no
program around it. Both are a day's work. Only one of them can be run.

---

**Exercise 2-1.** Add a `-q`/`--quiet` flag that suppresses the "unknown
command" reply. Decide where the flag's value lives: threaded into `dispatch`,
or held by the loop and consulted when an `Action` comes back. Argue for one.

**Exercise 2-2.** `ved -p` with no value prints `-p requires a value`. Delete
the bounds check and find out what happens instead. Now write the same bug in
the `--prompt=` arm, which has a different failure.

**Exercise 2-3.** The `other =>` arm rejects any unrecognized argument. Change
it to accept a bare word as a filename while still rejecting unknown flags, and
check how `ved -- -p` should behave. (The shipped ved does this; compare after
you have written yours.)

**Exercise 2-4.** Remove the `out.flush()` and run the program interactively.
Then run it with its input piped from `printf`. Explain why only one of the two
misbehaves.
