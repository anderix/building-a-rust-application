# 5. Writing, Quitting, and the Unsaved Buffer

Slice four is the first time the program touches anything outside itself. Until
now every failure was a typo and every consequence was a line of output. From
here the program can create a file, overwrite one, and lose an afternoon of
somebody's work.

```text
$ cargo run -q -p ch05-write -- -p '* '
* a
alpha
beta
.
* w out.txt
wrote 11 bytes to out.txt
* q
$
```

It also becomes, for the first time, a program that argues with the user:

```text
* a
x
.
* q
? warning: buffer modified
* q
$
```

That exchange is ed's, it is one boolean, and it is the reason this chapter is
longer than the command deserves.

## 5.1 Commands that take arguments

Chapter 4's dispatcher matched the remainder of the line exactly, which works
while every command is a single letter. `w out.txt` is a letter and an
argument, so the match needs a second stage:

```rust,ignore
    // Stage two: commands with arguments. Match on the first byte
    // and pass the rest as the argument string.
    let first = rest.as_bytes()[0];
    let args = &rest[1..];
    match first {
        b'w' => run_write(&spec, buf, args),
        _ => Action::Error(format!("unknown command: {rest}")),
    }
```

Four lines, and every command from here to slice 12 that carries an argument
lands in that match: `s`, `g`, `v`, `m`, `t`, `r`, `e`.

The indexing is safe from the empty case, because `rest.is_empty()` was checked
seventeen lines earlier and returned. It is not safe from anything else:

```text
$ ved
a
x
.
é
thread 'main' panicked at src/main.rs:296:21:
start byte index 1 is not a char boundary; it is inside 'é' (bytes 0..2 of string)
```

`rest[1..]` slices a `str` at byte 1. When the first character is multi-byte,
byte 1 is inside it, and slicing a `str` at a position that is not a character
boundary panics. Any non-ASCII character typed as a command crashes the editor
and takes the buffer with it.

That panic was in ved from this slice until the chapter you are reading was
written, which is the reason it is here rather than in a list of things to
watch for. It went into the tracker that day as [ved
#8](https://github.com/excelano/ved/issues/8), and the fix shipped the next day
in 0.2.1.

Chapter 3 argued that a panic in a text editor loses the buffer, as the reason
`Buffer::line` returns an `Option` instead of indexing. The argument was right,
and this is what it looks like when the same care is not taken twelve lines
further along in the same file.

Finding one of these is less work than it sounds, because the dangerous
operation is specific: a literal byte offset applied to a `str`. Most of the
program never performs one. `run_substitute` and `run_global` call `as_bytes()`
first and index the byte slice, where every offset is legal, and `bre.rs` works
in bytes from end to end. `address.rs` does slice a `str` at `[1..]`, in five
places, but each one is reached only after matching an ASCII byte, so byte 1 is
always a boundary. That left the dispatcher: the one site that took a byte
offset into a string it had not inspected, and the site holding whatever the
user had typed.

The fix removes the arithmetic rather than correcting it:

```rust,ignore
    let mut chars = rest.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return Action::Error(format!("unknown command: {rest}")),
    };
    let args = chars.as_str();
    match first {
        'w' => run_write(&spec, buf, args),
        // ...
    }
```

`Chars::as_str` returns the rest of the string after the character just taken,
so there is no index to get wrong and no `len_utf8` to add up. The arms become
character literals. The `None` arm is the part worth copying: it replaces an
invariant held seventeen lines away with a branch the compiler can see, and
costs three lines to stop needing to remember anything.

The bug survived for as long as it did because the tests drive the program with
ASCII, which is what the command language is made of, and because nobody types
`é` at an editor on purpose. That is the profile of most latent panics —
unreachable by intent, one paste away by accident — and chapter 15 comes back
to it as a question about what a suite built from valid input cannot tell you.

## 5.2 Where the filename lives

```rust,ignore
pub struct Buffer {
    lines: Vec<String>,
    current: usize,
    modified: bool,
    filename: Option<String>,
}
```

Two fields arrive together, and neither is text.

`filename` is what makes a bare `w` work after a `w out.txt`. ed remembers the
last file you wrote and reuses it, which is why an ed session reads as a series
of terse commands rather than a series of paths. `run_write` consults it only
when the argument is empty:

```rust,ignore
    let filename: String = if filename_arg.is_empty() {
        match buf.filename() {
            Some(f) => f.to_string(),
            None => return Action::Error("no current filename".to_string()),
        }
    } else {
        filename_arg.to_string()
    };
```

The failure case is the interesting one. A bare `w` with nothing remembered has
no sensible default, and the two wrong answers are both tempting: writing to a
name the program invents, and doing nothing quietly. ved says `no current
filename` and changes nothing. A command that cannot do what was asked should
say so; the cost of that sentence is one `Option` and one match arm.

Putting the filename on `Buffer` rather than in the REPL loop is the choice
that looks arbitrary and is not. By slice 11 the buffer can be loaded from a
file, and at that point "which file is this" is a property of the text, not of
the session. Had it lived in `run_repl`, slice 11 would have moved it.

## 5.3 What `modified` means

```rust,ignore
    pub fn append_after(&mut self, after: usize, line: String) {
        self.lines.insert(after, line);
        self.current = after + 1;
        self.modified = true;
    }
```

Every mutation sets it. One thing clears it:

```rust,ignore
    /// Called by `w` after a successful whole-buffer write. A
    /// partial-range write does NOT clear modified, since the
    /// buffer still differs from any single file on disk.
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
```

The comment is the design, and it is easy to get wrong. `1,2w part.txt` on
a three-line buffer has written a file successfully, and the buffer is still
unsaved, because what is on disk is not what is in memory. A flag that meant
"a write succeeded" would clear here and the program would then let you quit
over the top of unsaved work:

```text
* 1,2w part.txt
wrote 6 bytes to part.txt
* q
? warning: buffer modified
```

`run_write` computes the distinction rather than trusting the command form,
because `1,$w` and a bare `w` are the same write:

```rust,ignore
        let whole = range.start == 1 && range.end == buf.len();
```

The flag is named for the buffer's relationship to the file, not for what the
last command did. Naming it `saved` or `dirty` would invite the same bug from
the other direction.

There is one more case, and it is the reason `run_write` does not simply call
`resolve_or_whole` and be done:

```rust,ignore
    let (content, wrote_whole) = if buf.is_empty() && spec.is_empty() {
        (String::new(), true)
    } else {
```

An empty buffer resolves to an error, because chapter 4's `resolve_with` opens
with `if buf.is_empty()`. But `w empty.txt` on an empty buffer is a legitimate
request that should produce a zero-byte file, not `? invalid address`. Writing
nothing is a thing a user can want. This is the first appearance of a pattern
that recurs through the rest of the program: the general path is right for
every case except the empty one, and the empty one has to be spelled out.

## 5.4 The double-`q` dance

```rust,ignore
            Action::Quit => {
                if buffer.is_modified() && !quit_warned {
                    let _ = writeln!(out, "? warning: buffer modified");
                    quit_warned = true;
                } else {
                    return ExitCode::SUCCESS;
                }
            }
            Action::EnterInputMode => {
                input_mode = true;
                quit_warned = false;
            }
            Action::Print(msg) => {
                let _ = writeln!(out, "{msg}");
                quit_warned = false;
            }
            Action::Error(msg) => {
                let _ = writeln!(out, "? {msg}");
                quit_warned = false;
            }
```

The first `q` on a modified buffer warns. A second `q` immediately after quits.
Anything in between puts the guard back:

```text
* q
? warning: buffer modified
* p
x
* q
? warning: buffer modified
```

Three things about where this code is.

**The guard is in the loop, not in `dispatch`.** `quit_warned` is a fact about
the session, like `input_mode`, and it changes in response to every action
rather than in response to one command. `dispatch` still answers the question
"what does this command mean" and does not know that the program is arguing
with anybody.

**Every arm resets it, including the error arm.** That is deliberate and it is
the part people get wrong: a mistyped command between the two `q`s should cancel
the confirmation, because the user's attention moved. The arms are written out
one by one rather than reset once before the match, which is four lines instead
of one and is the version that survives a fifth variant being added, since a new
variant with no reset is a compile error only if the author is looking. Chapter
11 adds one.

**The warning goes through the same `?` channel as errors.** It is not an error
— nothing failed — and it is formatted as one, because ed formats it as one and
scripts grep for the `?`. Chapter 14 comes back to whether that was right.

The asymmetry worth noticing: a program that refuses to lose your work without
asking, but asks by printing one line and then obeying you a second later. ed's
answer to "are you sure" is to make you repeat yourself. It works because the
cost of being wrong is one keystroke and the cost of a dialog box is a program
that cannot be driven from a pipe.

## 5.5 Writing the file

```rust,ignore
    if let Err(e) = std::fs::write(&filename, &content) {
        return Action::Error(format!("cannot write {}: {}", filename, e));
    }
```

One line does the work and one line turns its failure into a sentence:

```text
* w /nope/x.txt
? cannot write /nope/x.txt: No such file or directory (os error 2)
```

The message includes the path, because the user may have typed it wrong, and
the operating system's own words, because `os error 2` is what the manual pages
and the search results are about. It does not include a stack trace, and the
program does not exit. A failed write is an ordinary event.

`std::fs::write` creates or truncates, writes the bytes, and closes. What it
does not do is make the replacement atomic. Between the truncate and the last
byte, the file on disk is neither the old contents nor the new ones, and a
crash, a full disk, or a power loss in that window leaves it that way. The
durable version writes to a temporary file in the same directory and renames it
over the target, because rename within a filesystem is atomic, so the file is
either entirely old or entirely new and never half of either.

ved does not do this. For a tool whose users are mostly writing small files
interactively it has never mattered, and saying so is more useful than
pretending the simple call was a considered choice. It is also the kind of
thing that becomes a bug report exactly once, from someone whose disk filled
up.

## 5.6 Saying what happened

```rust,ignore
    let msg = format!("wrote {bytes} bytes to {filename}");
```

Real ed prints `11`. Just the number, no context, no filename. ved prints
`wrote 11 bytes to out.txt`, and this is the first place the program's stated
purpose shows up as code rather than as a sentence in `PLAN.md`.

The byte count is kept because it is the part of ed's output that carries
information: it tells you the write happened and roughly what went out. The
rest is added because a terminal is not a quiz. Two things follow from having
made the decision once, here, at slice four rather than at slice nine: every
command added afterwards was written with a confirmation line from the start,
and the format stayed consistent because there was something to be consistent
with.

The comment above the message is worth keeping, too:

```rust,ignore
    // Build the confirmation message before we move `filename` into
    // the buffer's filename slot.
    let msg = format!("wrote {bytes} bytes to {filename}");
    buf.set_filename(filename);
```

`set_filename` takes ownership. Formatting afterwards would be a borrow of a
moved value, and the compiler would say so. The comment documents an ordering
that exists for the compiler's sake, which is the useful kind of comment about
borrowing: not what the rule is, but why these two lines are in this order.

## 5.7 What slice four bought

The program can now be used for the thing it is for. Type text, write it to a
file, quit. Six hundred lines, four commands, and everything from here is
addition rather than foundation.

It also acquired its first three ways to hurt somebody: it can overwrite a file
without asking, it can leave a truncated file behind if it dies mid-write, and
it could, until recently, be crashed by a character. The first is ed's design
and stays. The second is a known trade. The third was a bug, and it lasted from
this slice to the writing of this chapter.

---

**Exercise 5-1.** This chapter's crate still panics, because slice 4 did. Fix
it there without looking at section 5.1 again. Then grep the finished program
in `examples/ved` for every literal byte offset applied to a string, and for
each one write the single sentence that says why it is safe. One of them will
take longer than the others.

**Exercise 5-2.** Make the write atomic: write to `filename.tmp` in the same
directory, then `std::fs::rename` over the target. Then work out what your
version does that the simple one did not, when the target is a symlink, and
when the process lacks write permission on the directory but has it on the
file.

**Exercise 5-3.** `1,2w part.txt` leaves the buffer modified. Write the session
that convinces you this is right, then change `mark_saved` to clear
unconditionally and write the session that loses data.

**Exercise 5-4.** The `q` guard resets on every action. Remove the reset from
the `Error` arm only, then find the sequence of three commands that quits
without warning and should not have.

**Exercise 5-5.** ved will overwrite an existing file without comment. Decide
whether it should, given that ed does and that scripts depend on it, and if you
decide it should not, work out how the new behavior stays scriptable.
