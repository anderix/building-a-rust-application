# 6. The Command Loop

This chapter has no slice of its own. The next one starts a regex engine that
will take three chapters and add more code than the program currently contains,
and before that happens it is worth reading what four slices have built as a
single thing rather than as four additions.

The code is `examples/ch05-write` for the version this chapter describes, and
`examples/ved` for what it becomes.

The claim being made is narrow and checkable. At the end of slice 4 the program
is 609 lines, `run_repl` is 75 of them, and `dispatch` is 52. At release the
program is 2,809 lines, `run_repl` is 95, and `dispatch` is 110. Everything
else — the regex engine, eleven more commands, file loading, encoding
detection, the help system — arrived without the loop changing shape.

## 6.1 The loop, whole

```rust,ignore
fn run_repl(prompt: Option<&str>) -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut reader = stdin.lock();
    let mut line = String::new();
    let mut buffer = Buffer::new();
    let mut input_mode = false;
    let mut quit_warned = false;

    loop {
        // prompt, unless we are in input mode
        // read a line; EOF and I/O error return here
        // strip the terminator, keep the spaces

        if input_mode {
            // a lone `.` leaves input mode; anything else is text
            continue;
        }

        match dispatch(trimmed, &mut buffer) {
            Action::Quit => { /* the unsaved-buffer guard */ }
            Action::EnterInputMode => input_mode = true,
            Action::Print(msg) => { let _ = writeln!(out, "{msg}"); }
            Action::Error(msg) => { let _ = writeln!(out, "? {msg}"); }
        }
    }
}
```

Four sections: set up, read, one branch for the mode, one match over what the
last command decided. Everything the program owns that is not text lives in
those nine local variables, and three of them are locks and buffers.

Read the responsibilities rather than the code and it divides cleanly. The loop
owns the terminal, the modes, and the session. `dispatch` owns the command
language. `Buffer` owns the text. `Spec` owns addresses. Nothing owns two of
those, and the direction of the dependencies runs one way: the loop calls
`dispatch`, `dispatch` calls `Buffer` and `Spec`, and neither of them can see
the terminal or knows a mode exists.

That is the property worth naming, because it is what the rest of this chapter
is about and it did not happen by accident. It was the decision in chapter 2 to
have `dispatch` return an `Action` rather than print, made when the program had
one command and the decision looked like ceremony.

## 6.2 What twelve more commands did to it

By release the loop has gained twenty lines, and they are worth listing because
the list is short.

A file named on the command line is loaded before the loop starts. Two `Action`
variants arrive: `ForceQuit`, which is `Q`, quitting without the unsaved-buffer
warning, and `Done`, which is the variant for commands that change the buffer
and have nothing to say beyond their confirmation. And `quit_warned` gains
resets in the two new arms.

That is the difference. No new mode, no state threaded through, no flag
consulted in three places. `s`, `g`, `v`, `m`, `t`, `j`, `d`, `c`, `i`, `l`,
`e`, `r`, and `H` were added without touching `run_repl` at all, because each
of them is a command, and commands are `dispatch`'s business.

A loop that had printed its own output would have grown differently. Each new
command would have added a call to `writeln!`, the error format would live in
fourteen places, and the rewrite of every message that the plan scheduled for
slice 9 would have been a search rather than an edit.

## 6.3 One function per command

Eleven commands in the finished program are a function with nearly the same
signature:

```rust,ignore
fn run_delete(spec: &Spec, buf: &mut Buffer) -> Action
fn run_join(spec: &Spec, buf: &mut Buffer) -> Action
fn run_write(spec: &Spec, buf: &mut Buffer, args: &str) -> Action
fn run_move(spec: &Spec, buf: &mut Buffer, dest_arg: &str) -> Action
fn run_global(spec: &Spec, buf: &mut Buffer, args: &str, invert: bool) -> Action
```

An address, the buffer, whatever text followed the letter, and an `Action` back.
The uniformity is not aesthetic. It means every command is callable from a test
with two lines of setup, it means a new command has an obvious shape before it
has an implementation, and it means the difference between two commands is
visible in one place: `run_global` takes an `invert` flag because `g` and `v`
are the same command with the match sense reversed, and that fact is legible in
the signature rather than buried in two near-identical functions.

The naming convention is doing work too. `run_` is a prefix with no meaning
except "this is a command implementation", which is what a reader needs
from it. Grepping for `fn run_` produces the command set.

## 6.4 A match, not a table

The obvious next thought, somewhere around the sixth command, is that this
should be a table: a `HashMap<char, Box<dyn Fn(&Spec, &mut Buffer, &str) ->
Action>>`, registered at startup, so that adding a command is data rather than
code.

ved does not do this, and the reasoning generalizes past ed.

A match arm is checked at compile time. A misspelled command name in a table is
a runtime miss. The match is also faster, which does not matter here, and
shorter, which does: the table version needs a registration call, a boxed
closure per command, and a lookup that can fail, in exchange for deleting a
match arm. The extensibility it buys is the ability to add commands without
recompiling, which is worth a great deal to a program with a plugin system and
nothing at all to a program shipped as one static binary.

The argument for the table arrives when commands come from somewhere the
compiler cannot see, when there are hundreds of them, or when something other
than the dispatcher needs to enumerate them. ved's help text, which does need
to enumerate them, is a separate hand-written list, and chapter 14 admits what
that costs.

The general form of this mistake is worth recognizing because it is expensive
and it feels like craftsmanship at the time: building the mechanism for a
variation the program does not have. The cost is not the mechanism. It is that
every future reader has to understand the mechanism before they can find out
what `d` does.

## 6.5 Errors are strings

```rust,ignore
enum Action {
    Quit,
    ForceQuit,
    EnterInputMode,
    Done,
    Print(String),
    Error(String),
}
```

The program defines no error type. `Action::Error` carries a `String`,
`address.rs` returns `Result<_, String>` in eight places, and `bre.rs` does not
use `Result` at all — a match either succeeds or returns `None`.

In a library this would be wrong, and straightforwardly so. A caller who
receives a `String` cannot branch on what went wrong without matching on
English, cannot translate it, and cannot tell a bad address from a full disk
except by reading it. Error types exist so that the caller can make decisions.

ved's errors have one destination: a terminal, formatted with a `?` in front,
read by a person. Nothing in the program branches on the *kind* of failure —
there is no retry, no fallback, no recovery path that differs by cause. Every
error does the same thing, which is to be printed and to leave the buffer
alone. A `enum VedError { InvalidAddress, NoMatch, ... }` would add a type, a
`Display` implementation that reproduces the strings that are already there,
and a conversion at each boundary, in exchange for nothing this program does.

The honest version of this position has a boundary on it, and the boundary is
reuse. The moment `bre.rs` is published as a crate — an idea that sits in ved's
backlog — its failures acquire callers who need to branch, and `String` stops
being adequate. The same is true if ved ever gains a mode where an error should
be recoverable rather than reported. Neither has happened. The rule that
follows is not "errors should be strings"; it is that an error type is a
decision about what callers need to do, and a program whose only caller is a
terminal has one caller with one need.

## 6.6 Three commands that never got a function

`dispatch` is 110 lines in the released program, and it should be about 70.
Three commands are implemented inline in the middle of it rather than as
`run_` functions:

```rust,ignore
        "i" | "insert" => {
            // Insert before the addressed line (default: current).
            // Implemented as "append after address - 1", so the
            // input-mode loop (which always does append_after) works
            // unchanged. On an empty buffer or address 1, current
            // becomes 0 (the before-first-line sentinel).
            if !buf.is_empty() {
                let addr = match spec.resolve(buf) {
                    Ok(r) => r.end,
                    Err(e) => return Action::Error(e),
                };
                buf.set_current(addr.saturating_sub(1));
            }
            return Action::EnterInputMode;
        }
```

`a`, `i`, and `c` all do the same two things: move the current line to where
the following text should land, then return `Action::EnterInputMode`. They are
inline because each was three lines when it was written, and because they
return a mode change rather than output, which made them feel unlike the other
commands.

They are now eleven, fifteen, and fifteen lines, sitting between `"q"` and
`"d"` in a match that a reader is scanning to find out which commands exist.
The extraction is mechanical and nothing prevents it. It has not been done
because nothing has forced it, which is the answer in most codebases and is
worth saying rather than dressing up as a design.

The lesson is about when to notice. The cost of inline code in a dispatcher is
not felt when it is written; it is felt when the dispatcher stops being
readable as a list, and by then it has stopped being obvious that a rule was
ever broken. The signal is structural rather than aesthetic: the arms of this
match no longer all look alike.

## 6.7 Why all of this is testable

```rust,ignore
    #[test]
    fn move_into_self_errors() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"]);
        let act = dispatch("2,4m3", &mut buf);
        assert!(matches!(act, Action::Error(_)));
        assert_eq!(buf.line(3), Some("c"));
    }
```

Build a buffer, call `dispatch` with a string, assert on the value that comes
back and on the buffer afterwards. No process is spawned, no terminal is
faked, no output is captured and parsed, and the test runs in microseconds.

Most of ved's 123 tests are that shape, and they are that shape because of the
decision in chapter 2 and nothing else. A `dispatch` that printed would need
its output captured, which in Rust means either threading a `Write` through
every command or replacing stdout globally, and the tests would assert on
formatted text, so changing a message would break tests that were not about
messages.

Chapter 15 is about the parts this does *not* cover, of which there are more
than the passing count suggests. The panic in chapter 5 lived under 122 green
tests.

## 6.8 What to take from it

Four things, and none of them are about ed.

The program's I/O happens in one function. Everything else returns values.

The command language is one function that reads like a list of what the
commands are, and the rule is that new commands are arms rather than
conditions.

The state is small enough to enumerate and it lives with whoever owns the
concept: the text in `Buffer`, the session in `run_repl`, nothing in
`dispatch`.

And the shape stopped changing after four slices. That is the return on the
decisions in chapters 2 through 5, and it is why the next three chapters can
build a regex engine without once mentioning the REPL.

---

**Exercise 6-1.** Extract `a`, `i`, and `c` into `run_append`, `run_insert`,
and `run_change`. Then decide what their return type should be, given that they
change the buffer's current line and return a mode change rather than output.

**Exercise 6-2.** Write the table-driven dispatcher from section 6.4 for the
five commands in `examples/ch05-write`. Count the lines, then decide what would
have to be true about the program for the answer to flip.

**Exercise 6-3.** Define an error enum for `address.rs` with the three cases it
actually produces, convert the eight `Result<_, String>` signatures, and see
what happens at the call sites in `dispatch`. Keep it only if they got better.

**Exercise 6-4.** `run_repl` grew from 75 lines to 95 across eight more slices.
Pick a program you have written with a main loop, measure the same two numbers,
and if the answer is worse, find the command that should have been a value.
