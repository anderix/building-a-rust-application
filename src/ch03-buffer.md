# 3. A Buffer and a Second Mode

Slice two gives the program something to remember. It adds a text buffer, a
command that puts text into it, two commands that print it back, and a second
mode, in which the loop that has been reading commands starts reading data
instead.

```text
$ cargo run -q -p ch03-buffer -- -p '* '
* a
hello
world
.
* ,p
hello
world
* p
world
* q
$
```

That is two hundred and fifty lines in two files. It cannot save, it cannot
delete, and the only line it can address is the last one. It is still an
editor: text goes in, text comes out, and the structure holding the text is the
one the shipped program is still using 2,800 lines later.

## 3.1 The second file

`main.rs` grows two lines at the top, and the program stops being a single
file:

```rust,ignore
mod buffer;

use buffer::Buffer;
```

Two hundred and fifty lines do not need splitting for length. The split is
here because `Buffer` has an invariant, and a module boundary is the only place
Rust lets you defend one.

The invariant is that `lines` and `current` agree: `current` is either 0,
meaning the buffer is empty or positioned before the first line, or a valid
1-indexed position in `lines`. Nothing else is allowed. With both fields
private and every mutation going through a method, the code that can violate
that rule is the sixty-two lines in `buffer.rs`, and it is sixty-two lines you
can read in one sitting. Had `Buffer` been a struct with `pub` fields declared
in `main.rs`, the rule would be a comment, and comments do not hold.

This is the general case for splitting a file, and it is not about size. A
module is worth making when there is something to keep out.

## 3.2 `Vec<String>`, and what it costs

```rust,ignore
pub struct Buffer {
    lines: Vec<String>,
    current: usize, // 0 means empty buffer / before-first-line
}
```

A vector of owned strings, one per line, with the newlines stripped. It is the
first representation anyone would think of, and it has a real cost:
`Vec::insert` at position *n* moves every element after it, so inserting at the
top of a hundred-thousand-line file moves a hundred thousand `String` headers.

That cost is the reason serious editors use something else. A gap buffer keeps
a movable hole at the cursor so that repeated edits in one place are free. A
piece table never moves the original text at all and stores an edit list over
it. A rope makes insertion logarithmic anywhere. Any of the three would be a
defensible choice, and all three are more code than the rest of slice two put
together.

ved uses the vector, and the shipped version still does. The reasoning is
worth being explicit about rather than leaving as an accident: a `String`
header is twenty-four bytes, so moving a hundred thousand of them is a single
`memmove` of 2.4 megabytes, which is faster than the terminal that requested
it. ed's editing pattern is mostly append and mostly sequential. And the
program's promise is to be small enough to read.

The general point is the one about representations, not the one about vectors.
The structure chosen on the second day of this project survived to release
unchanged, with four fields added around it and none of them touching `lines`.
That is the usual outcome of picking the simple thing, and it is a better bet
than the rope you write in week one against a profile you have not taken.
Chapter 15 is where the measuring happens.

## 3.3 One place where 1 becomes 0

ed's addresses are 1-indexed: line 1 is the first line, `$` is the last. Rust's
vectors are 0-indexed. Somewhere the two have to meet, and the decision is how
many places that happens in.

```rust,ignore
/// Fetch a line by 1-indexed position. Returns None if the
/// address is out of range.
pub fn line(&self, n: usize) -> Option<&str> {
    if n == 0 || n > self.lines.len() {
        None
    } else {
        Some(self.lines[n - 1].as_str())
    }
}
```

The `n - 1` appears once in the program. Every other piece of code works in
ed's coordinates and asks `Buffer` to do the conversion. Spread the subtraction
across a dozen command handlers instead and each new command is a fresh chance
to write it wrong, producing the off-by-one at a boundary that survives casual
testing and shows up in someone's file.

The `Option` is the second half of the same decision. Addresses come from
users, so `1000p` on a three-line buffer is a typo rather than a bug, and the
program owes it an answer rather than a crash. Indexing a `Vec` out of range
panics, and a panic in a text editor loses the buffer. Because the bounds check
and the conversion are the same function, there is no path through the program
that checks the range and then indexes separately, which is the arrangement
where the two drift apart.

## 3.4 An off-by-one that cancels

```rust,ignore
/// Insert `line` after 1-indexed position `after`. `after = 0`
/// means insert at the very start of the buffer (the case when
/// you run `a` on an empty buffer). Updates current to the
/// newly inserted line.
pub fn append_after(&mut self, after: usize, line: String) {
    self.lines.insert(after, line);
    self.current = after + 1;
}
```

`after` is 1-indexed, `Vec::insert` takes a 0-indexed position, and there is no
conversion between them. That looks like the bug the previous section was
about, and it is not.

Inserting *after* 1-indexed line *n* is inserting *at* 0-indexed position *n*.
Line 1 lives at index 0, so the space after it is index 1. The coordinate
systems differ by one, "after" and "at" differ by one, and the two differences
cancel.

The sentinel falls out of the same arithmetic. `after = 0` means "after the
position before the first line", which is insertion at index 0, the front of
the buffer, which is what `a` on an empty buffer has to do. The zero in
`current` is not a special case in this function; it is a value that already
behaves correctly.

Code where two errors cancel is code that gets helpfully broken by the next
person to read it, which is why both this version and the shipped one carry a
doc comment stating what `after = 0` means. A cancellation that is not written
down is a trap.

## 3.5 The current line

`current` is ed's least visible piece of state and its most consequential.
Nearly every command has a default address, the default is nearly always "the
current line", and nearly every command moves it to the last line it touched.
`p` with no address prints the current line. Bare Enter, below, advances it.
This is why ed sessions are short to type and impossible to read over someone's
shoulder.

At slice two there are two writers. `append_after` leaves it on the line just
inserted, so typing three lines into an empty buffer leaves it at 3. And `,p`
sets it to the last line printed:

```rust,ignore
buf.set_current(buf.len());
```

That single line is why `dispatch` takes `&mut Buffer` rather than `&Buffer`.
Printing is not a read-only operation in ed. It is the most surprising fact
about the command language, it is invisible until a command two slices later
defaults to the wrong line, and Chapter 1 told the story of discovering it.

## 3.6 The second mode

```rust,ignore
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
```

A modal editor is the thing ed is famous for being, by way of `vi`, and the
mechanism is a `bool` and an early `continue`.

**The mode belongs to the loop.** It is not a field on `Buffer`, because it is
not part of the text. It is not state inside `dispatch`, because `dispatch` is
a pure function from a command and a buffer to an `Action`, and chapter 2 spent
a section on why that is worth keeping. A `dispatch` that remembered whether it
was in input mode would have to be called in sequence to be tested, and ved's
command-language tests call it in any order they like.

**`a` describes, the loop acts.** `dispatch` returns `Action::EnterInputMode`
and touches nothing. The one line that changes the program's mode is in the
loop, next to the lines that read and write, where every other decision about
what the terminal is doing already lives.

**The early `continue` keeps the two paths apart.** Input mode is four lines
and then the iteration ends. The command path below it is not indented inside
an `else` and does not test the flag again. When slice 6 adds `i` and `c`, both
of which also enter input mode, nothing here changes.

**The prompt is suppressed.** ed prints no prompt while you are typing text,
because a prompt would end up interleaved with the document. The condition is
slice 1's prompt block with one more `if` wrapped around it, and the nesting is
what it looks like: three levels of indentation to write one short string. It
stays that way for most of the program's life. Chapter 17 is where continuous
integration arrives, runs clippy for the first time, and flattens it.

The terminator is a line that is the single character `.`, and nothing else is:

```text
$ cargo run -q -p ch03-buffer
a
.hidden
..
.
,p
.hidden
..
q
```

A line consisting of one period therefore cannot be typed into the buffer. That
is ed's hole and not ved's; real ed has it too, and there is no escape syntax.
Files containing such a line load without trouble once chapter 13 adds file
reading, so the limitation is on typing alone.

## 3.7 Two kinds of trimming

```rust,ignore
// Strip just the line terminator, not all trailing whitespace
// — a user typing "  hello   " into the buffer should keep
// their trailing spaces. trim_end() would eat them.
let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
```

Slice 1 called `line.trim_end()`, which was correct while every line was a
command. It becomes a data-loss bug the moment a line can be text.

Commands are syntax, and whitespace around syntax is noise. Buffer contents are
data, and whitespace inside data is content. An editor that quietly strips
trailing spaces from typed lines corrupts Makefiles, Markdown hard line breaks,
and any file whose tests compare bytes. The program cannot have one rule for
both, so it strips only the line terminator here and leaves `dispatch` to call
`.trim()` on what it receives. The leniency stays on the command path.

The two calls in sequence handle CRLF: strip the `\n`, then strip the `\r`
underneath it. The order is not interchangeable, since the reverse pair leaves
a CRLF line ending in `\r`. `read_line` returns at most one terminator, so the
fact that `trim_end_matches` would strip a repeated run never comes up.

## 3.8 The enum grows an error

```rust,ignore
enum Action {
    Quit,
    EnterInputMode,
    Print(String),
    Error(String),
}
```

`Continue` is gone and two variants have arrived.

`Error` is the split chapter 2 promised. Slice 1 wrote
`format!("? unknown command: {other}")`, with the marker baked into the text.
Now `dispatch` returns the message and the loop writes the `?`. One place in
the program decides what an error looks like, which is what would make the
rewrite of every error message that the plan scheduled for slice 9 a contained
piece of work rather than a sweep. Chapter 14 is about what became of it.

`Continue` is gone because the empty line stopped being a command that does
nothing. In ed, bare Enter is shorthand for `+1p`: advance to the next line and
print it, which is how you walk through a file.

```rust,ignore
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
```

At slice two that branch can never print anything. `append_after` always
leaves `current` on the line it inserted, `p` does not move it, and `,p`
sets it to the last line, so `current` is always the final line of the
buffer and `next` is always past the end. Every bare Enter in slice two
answers `? invalid address`.

It is written now for two reasons. It is what ed does, and slice 3 is what
makes it reachable: once `1p` can move the current line back to the top, Enter
walks down from there without another edit. The alternative was leaving Enter
out and letting it fall through to `unknown command`, which would have been a
regression from slice 1, where a bare Enter at least did nothing rude.

The rest of `dispatch` includes one piece of scaffolding worth labeling as
such:

```rust,ignore
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
```

`,p` is matched as a literal two-character string. In ed, `,` is shorthand for
the range `1,$` and `,p` is an address followed by a command, so the general
form should work too; at slice two, `1,2p` is an unknown command. The arm is a
placeholder sized to keep the program usable until slice 3 arrives, and slice 3
deletes it along with the `p` arm, replacing both with an address parser and
one print routine.

Most slices contain some of this. The part worth doing deliberately is marking
it, which here is the doc comment on `dispatch` saying that slice 3 replaces
the match. A placeholder nobody labelled is indistinguishable from a design
decision six months later.

Note also that the whole buffer is assembled into one `String` and returned as
a single `Print`, rather than printed line by line from inside `dispatch`. That
is the chapter 2 rule holding under its first real pressure: `dispatch` does
not get to touch the terminal, even when it has a thousand lines to emit. The
cost is a second copy of the buffer in memory for the duration of the command.
Chapter 13 revisits it when files get large enough for that to matter.

## 3.9 What slice two is for

Slice one was a program that ran. Slice two is a program that holds state, and
it puts in place the three things every later slice rearranges: a data
structure with an invariant behind a module boundary, a dispatcher that
describes what should happen rather than doing it, and a loop that owns the
terminal, the modes, and every byte that reaches the user.

The remaining ten slices add commands. None of them change that shape.

---

**Exercise 3-1.** Add `i` (insert before the current line) using the existing
`append_after`. Decide what it should do on an empty buffer, then check your
answer against GNU ed, which is almost certainly installed on the machine you
are reading this on.

**Exercise 3-2.** `printf 'a\nstranded\n' | cargo run -q -p ch03-buffer` exits 0
and the typed line is gone: EOF in input mode discards the buffer silently.
Decide what should happen instead, and find out what real ed does before you
implement it. (ved's answer arrives in chapter 5, with `w` and the unsaved-buffer
guard.)

**Exercise 3-3.** Change the terminator handling so that a lone `.` can be
entered into the buffer. Then write down what your escape syntax costs: every
ed script ever written assumes the current rule.

**Exercise 3-4.** Put `trim_end()` back in place of the two
`trim_end_matches` calls and write the shortest session that demonstrates the
bug. Then decide whether the buffer should keep a trailing `\r` when the input
is a CRLF file, and what that means for a program that will one day write the
buffer back out.

**Exercise 3-5.** The chapter claims `Vec<String>` is fast enough. Measure it:
build a buffer of 100,000 lines, time 1,000 insertions at position 1, and time
1,000 at the end. Report both. If the first number does not bother you, the
representation is right.
