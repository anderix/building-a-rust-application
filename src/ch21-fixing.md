# 21. Fixing Them

Chapter 20 ended with twenty-four issues in an order. This chapter fixes
them: the procedure, then the fixes that teach something, in the order they
were made. The code is `examples/ch21-fixed`, which is ved 0.4.0, and every
fix is one commit in ved's history, on a branch named `fix/book-findings`,
merged as pull request #37.

## 21.1 The procedure

Each fix followed the same steps, and the steps were the point:

1. Run the session from the issue against the current binary and watch it
   fail. An issue that does not reproduce is a different conversation.
2. Write the test first, with its expected value taken from a reference, not
   from the code. Section 20.5 showed what happens otherwise.
3. Make the smallest change that turns the test from failing to passing
   without breaking another.
4. Run the whole suite, `cargo fmt --check`, and `cargo clippy -- -D warnings`.
5. Run the session from the issue again, against the built binary.
6. Commit, with a message that says what was wrong, what changed, and which
   reference decided, ending in `Fixes #N`.

And after the last fix, two checks that no single fix can do: the
differential harness from chapter 15, to find what the fixes changed that
nobody asked them to, and the fuzzer, to find what they broke. Both found
something.

Two rules shaped the commits. One issue per commit, so that a commit can be
reverted without reverting anything else, and so that the history can be
read as the list of issues. And the reasoning goes in the commit message,
where it stays attached to the change, rather than in a comment, where it
describes a change that is no longer visible. A reader of `git log` for 0.4.0
gets twenty-three short essays on what was wrong with ved and why the fix is
the fix.

## 21.2 Two lines: `wq`

The smallest fix is the most common case of the most damaging bug. `wq`
reached the `w` arm with the argument `q`, and the `w` arm wrote a file by
that name (#14). The fix is to look for the `q` first:

```rust,ignore
        'w' => match args.strip_prefix('q') {
            Some(file) if file.is_empty() || file.starts_with(char::is_whitespace) => {
                match run_write(&spec, buf, file) {
                    Action::Confirm(msg) => Action::ConfirmAndQuit(msg),
                    other => other,
                }
            }
            _ => run_write(&spec, buf, args),
        },
```

A `wq` that cannot write returns the write's error and does not quit, which
is what GNU ed does and the only safe choice. A new `Action`,
`ConfirmAndQuit`, tells the loop to print the byte count and leave.

Checking `wq` against the references found that it is not in POSIX. The
specification's rationale lists it among "the BSD ed commands W, wq, and z"
that it leaves out, and Seventh Edition ed answers it with `?`. ved keeps it,
because scripts written for GNU and BSD ed end with it, and because the
alternative, an error, would still be better than what 0.3.0 did. The commit
message says so, which is what section 20.7 asked for: a difference from the
reference that is written down.

## 21.3 The rule first, then the feature: long forms

`write`, `edit`, and `read` (#15) could have been fixed by adding three
strings to a match arm. That would have fixed the three words and left the
cause: `w` followed by anything at all is a write to a file named after the
anything, so `wnotes.txt` writes `notes.txt`. GNU ed and Seventh Edition ed
both reject that. ved now does too, and once a filename command needs
whitespace after its letter, the long forms cannot collide with a filename:

```rust,ignore
fn split_command(rest: &str) -> Option<(char, &str)> {
    for (long, letter) in [("edit", 'e'), ("read", 'r'), ("write", 'w')] {
        if let Some(args) = rest.strip_prefix(long)
            && (args.is_empty() || args.starts_with(char::is_whitespace))
        {
            return Some((letter, args));
        }
    }
    let mut chars = rest.chars();
    let first = chars.next()?;
    let args = chars.as_str();
    let takes_filename = matches!(first, 'e' | 'E' | 'r' | 'w');
    let is_wq = first == 'w'
        && args
            .strip_prefix('q')
            .is_some_and(|f| f.is_empty() || f.starts_with(char::is_whitespace));
    if takes_filename && !is_wq && !(args.is_empty() || args.starts_with(char::is_whitespace)) {
        return None;
    }
    Some((first, args))
}
```

The order of the two changes is the lesson. Fixing a symptom without the rule
that produced it leaves the next symptom in place, and here the next symptom
was already visible: a user who types `writenotes.txt`, without the space,
would have got a file named `ritenotes.txt` from the alias-only fix.

This fix also fixed the tutorial. The tutorial teaches `write notes.txt` and
`edit filename`; with the long forms implemented and `e` fixed (below), what
it teaches is now what the program does. Replaying every session in the
tutorial against the new binary, rather than reading it, found two more
errors in it, a capture-group example that searched for text an earlier step
had changed and a byte count that was never right. Both are now pasted from a
run. The issue (#16) stayed open until the evening, because the tutorial is
also published on two websites, and those copies change only when they are
deployed.

## 21.4 Build, then swap: `e`

`e` had two bugs and they needed different kinds of fix (#17). The first was
an order:

```rust,ignore
    let mut fresh = Buffer::new();
    let action = match load_file(&filename, &mut fresh) {
        Ok(LoadOutcome::Loaded(bytes)) => Action::Confirm(format!("{bytes}")),
        Ok(LoadOutcome::NewFile) => Action::Confirm(format!("{filename}: new file")),
        Err(e) => return Action::Error(e),
    };
    *buf = fresh;
    action
```

The old buffer is replaced only after the new one exists. A load that fails
returns before the assignment and leaves the old buffer, its modified flag,
and the user's afternoon of work where they were.

The second was missing behavior. POSIX says that `e` on a modified buffer
warns, sets a flag it calls *changed-and-warned*, and takes effect if it is
repeated with nothing in between, which is the same rule as `q`, and the same
flag. Seventh Edition ed behaves that way. Chapter 13's scratch fix added only
`E`, the form that skips the check, because the double-`e` needs the loop's
warning state, which `dispatch` cannot see. The shipped fix keeps `dispatch`
stateless by letting it ask:

```rust,ignore
        let action = match dispatch(trimmed, &mut buffer) {
            Action::EditModified(args) if modified_warned => run_edit(&mut buffer, &args),
            other => other,
        };
```

`dispatch` returns `EditModified` for an `e` on a modified buffer. The loop
warns the first time and runs the edit if the next command is another `e`.
The loop's `quit_warned` became `modified_warned` and serves both commands,
as POSIX's one flag does. Chapter 6 argued that `dispatch` should hold no
state, and this is what that argument costs and buys: one more variant in
`Action`, and a dispatcher that can still be tested without a loop.

## 21.5 A fix that changes the data structure: marks

The `g` bug (#19) could not be fixed inside `run_global`. The loop needs to
visit the lines it matched, and the only way to find a line after other lines
have moved is for the mark to move with it. Chapter 12 showed how the Seventh
Edition does it, with a mark bit in each line's entry in the buffer's table.
ved now does the same:

```rust,ignore
struct Line {
    text: String,
    marked: bool,
}
```

`Buffer` holds a `Vec<Line>` instead of a `Vec<String>`, and gains three
methods, `set_mark`, `take_mark`, and `clear_marks`. `run_global` sets a mark
on each matching line and then takes them one at a time; `take_mark` returns
the first marked line by its number at that moment and clears it.

What makes the fix hold is not in `run_global`. Chapter 3 put every change to
the lines behind three methods, `append_after`, `delete_range`, and
`replace_line`, and the mark now rides on those. `append_after` inserts an
unmarked line, so marks below it move down with their lines. `delete_range`
drains lines and their marks together. `replace_line` changes the text and
leaves the mark. No command needed to learn about marks, because no command
touches the vector. The module boundary from chapter 3 was paid for in that
chapter and collected on here.

One command did need to change. `m` moved lines by copying their text,
deleting them, and inserting the copies, which loses the marks on the lines
it moves. It now uses a fourth primitive, `move_range`, which moves the `Line`
values themselves, so a marked line that `m` moves keeps its mark. The
Seventh Edition's `m` moves table entries, mark bits included, for the same
effect.

`take_mark` searches for the first marked line, and searching from the top
each time would make `g/^/m0` on a large file quadratic in a new way. `Buffer`
keeps `first_mark`, a position no mark is above, which the primitives lower
when they could have moved a mark above it. Chapter 15's benchmark command,
`g/^/m0` on 100,000 lines, took 7.09 seconds on 0.3.0 and 7.06 on the fix.

The differential harness, run after this fix, found one more: `g/re/d` left
the current line after the first deleted line, where GNU ed and Seventh
Edition ed leave it after the last (#34). The fast path for a plain `d`
deletes from the bottom up, which keeps the numbers valid without marks and
is cheap, and so got the current line from the wrong end. It was in 0.3.0, it
had nothing to do with marks, and it became visible only when the harness had
fewer, louder differences to report.

## 21.6 A fix that changes an API: `find_at`

The anchor and empty-match bugs (#22) came from searching slices of a line as
if they were lines. The fix gives the regex engine a way to search the whole
line from a position:

```rust,ignore
    pub fn find_at(&self, text: &[u8], start: usize) -> Option<Match> {
        let elements = &self.elements;
        let anchored = !elements.is_empty() && matches!(elements[0], Element::Caret);
        let elems = if anchored { &elements[1..] } else { elements };
        if anchored && start > 0 {
            return None;
        }
        // ... the search loop from `find`, starting at `start`
    }
```

and `find` becomes `find_at(text, 0)`. With that, one function replaces the
two that substituted the first match and every match, and takes a count for
#24:

```rust,ignore
    while search <= text.len() {
        let Some(m) = re.find_at(text, search) else {
            break;
        };
        let step = if m.start == m.end {
            bre::char_width(&text[m.start..]).max(1)
        } else {
            0
        };
        if m.start == m.end && prev_end == Some(m.start) {
            search = m.start + step;
            continue;
        }
        seen += 1;
        if seen >= nth {
            result.extend_from_slice(&text[copied..m.start]);
            result.extend_from_slice(&bre::expand_replacement(replacement, &m, text));
            copied = m.end;
            replaced = true;
            if !global {
                break;
            }
        }
        prev_end = Some(m.end);
        search = m.end + step;
    }
```

The skip in the middle is the rule for an empty match where the previous
match ended, and it is the one case in this chapter that POSIX does not
decide. GNU ed refuses `s/a*/X/g` with "Infinite substitution loop". Seventh
Edition ed answers `XbXcX` on `baaac`, and so does `sed`, so ved follows the
Seventh Edition, as the rule in section 20.7 says.

Making the change broke one test,
`global_substitute_with_star_that_can_match_empty_preserves_characters`, which
expected `XXéX`. Section 20.5 explained why that value was wrong. When a fix
breaks a test, the first question is which of the two is wrong, and the answer
comes from the reference, not from the side that changed most recently. Here
`sed` gave `XéX`, and the test changed.

## 21.7 Fixes that change what someone sees

Some fixes change behavior that a user may have come to rely on, and those
need a reason that outweighs that risk, written where the user will see it.

**CRLF** (#18). Splitting on `\n` alone keeps the `\r` in the line, so a
Windows file is written back with its own line endings. The cost is that
`s/e$/E/` no longer matches a line that ends in `\r`, and `l` shows the
`\r`. That is also how GNU ed and Seventh Edition ed behave. The alternative,
detecting the style at load and restoring it at write, would hide the `\r`
from patterns, but it would normalize a file with mixed endings, which is a
choice neither ed makes.

**Exit status** (#26). Any `?` now makes the exit status 1, including the
modified-buffer warning. POSIX requires it and says outright that historical
implementations which exited 0 are not allowed. A script that ran ved and
ignored its errors used to succeed and will now fail. That is the fix.

**`-s`** (#27). The flag needed one condition in the loop, but the loop could
not tell a line the user asked to see from a line ved says about a command.
Both were `Print`. So the fix starts with the type:

```rust,ignore
enum Action {
    // ...
    /// Output the user asked for, such as the lines `p` prints.
    Print(String),
    /// What ved says about a command it carried out, such as a byte
    /// count. `-s` suppresses these.
    Confirm(String),
    /// The results of the commands `g` ran, in order.
    Batch(Vec<Action>),
    Error(String),
}
```

Every byte count and confirmation became `Confirm`. `s` confirms the line it
changed, and prints it when given `p`, `l`, or `n`, which is what POSIX's `s`
does. `g` used to join its commands' output into one string, which would have
lost the difference, so it returns a `Batch`. Then `-s` is one line in one
function:

```rust,ignore
        Action::Confirm(msg) => {
            if !silent {
                let _ = writeln!(out, "{msg}");
            }
            false
        }
```

Chapter 14 predicted that shape, and Appendix C lists it as a step: put the
distinction in the types, and the flag is trivial. POSIX's `-s` suppresses
only byte counts, because ed has nothing else to suppress. ved's
confirmations have no ed equivalent, so `-s` suppresses them too, and the
output matches `ed -s` byte for byte on the scripts it was checked with.

**`j` with one address** (#35). POSIX says it does nothing and leaves the
current line unchanged. GNU ed agrees. Seventh Edition ed moved the current
line to the address, as ved did. This is the one case where the original and
the standard disagree and the standard was followed against the original,
because POSIX speaks.

## 21.8 A fix that removes code

The UTF-7 refusal (#25) was fixed by deleting it: the `non_utf8_source`
field, the two `Buffer` methods that read and wrote it, the check in
`run_write`, and the test that held it in place. The commit added 22 lines and
removed 54. The warning and its `iconv` hint stayed, because a Scoutbook
export is still worth pointing out; the refusal went, because the only files
it could ever refuse were ones that were not UTF-7.

Deleting a safety check feels worse than adding one, and the feeling is the
reason to write down the argument. The commit message traces the two
detections through `load_file` and shows that the guarded case cannot reach
the guard. A later reader who wonders why ved does not refuse to overwrite a
UTF-7 file can find the answer in the history, with the reasoning attached.

## 21.9 The fix that broke something

Making the closing delimiter optional (#23) was twelve lines, and it had a
test, and the suite passed. The fuzzer, run after the last fix, found this:

```text
PANIC substitution result is valid UTF-8: FromUtf8Error { bytes: [151, 106,
   e.g. ["sé*×j\\"] on [""]
1 distinct panics in 1000000 runs
```

`s` takes the first byte of its argument as the delimiter, and `é` is two
bytes. In 0.3.0 the same input failed as "unterminated replacement", because
the second delimiter was never found. With the closing delimiter optional,
the replacement ran to the end of the line and began with the second half of
`é`, and rebuilding the line as UTF-8 panicked. A change that was correct for
every input anyone had thought about turned an error into a crash for one
nobody had, which is the profile chapter 5 gave for latent panics and chapter 15
gave for the fuzzer's reason to exist.

The fix is a rule about delimiters, in its own commit:

```rust,ignore
fn valid_delimiter(delim: u8) -> bool {
    delim.is_ascii() && !matches!(delim, b' ' | b'\n')
}
```

POSIX allows any character except space and newline. ved scans fields byte by
byte, so it also needs the delimiter to be one byte, and GNU ed rejects a
multi-byte delimiter too. The first version of this fix also rejected
backslash, on the theory that it would make every field an escape. GNU ed
accepts `s\a\b\`, POSIX does not exclude it, and the scanner checks for the
delimiter before it checks for an escape, so the theory was wrong and the
commit was amended before the branch was pushed. A million fuzzer runs on the
result find no panics.

The order of the checks mattered here, and it makes a rule: the
fuzzer runs after the last fix, not after the first. Each fix changes the
input space the others see, and a crash introduced by fix eleven is
invisible to a fuzzer that ran after fix three.

## 21.10 Fixing the documents

Three fixes were to documents, and each was made the way chapter 16 said
documentation should be: by running the program and pasting what it printed.

The help (#29) gained `e`, `E`, `r`, `Q`, `wq`, the long forms, the search
addresses, and the right default address for `g`, `v`, and `w`. It also
gained a test, which chapter 14 proposed and nobody had written:

```rust,ignore
    fn help_mentions_every_command_the_dispatcher_accepts() {
        let Action::Print(help) = run_help() else {
            panic!("help should print");
        };
        for cmd in [
            "a", "i", "c", "d", "p", "n", "l", "j", "m", "t", "s", "g", "v", "w", "wq", "r",
            "e", "E", "q", "Q", "H",
        ] {
            // ... assert that some line of the help starts with `cmd`
        }
    }
```

The list in the test is a third copy of the command set, which is the
objection chapter 14 raised. It stays, because when it drifts it fails, and
the help drifted silently for six months.

The README's quick start (#31) had been typed. It is now a session captured
through a terminal, so the prompt, the byte counts, and the confirmations are
what ved prints. And the install section now says what a macOS user does:
install with Cargo, and leave the retired Homebrew tap, whose users would
otherwise sit on 0.2.1 with no sign that anything had changed.

## 21.11 What it came to

| Commit | Issue | Lines added | Removed |
|---|---|---|---|
| `wq` writes the file and quits | #14 | 42 | 1 |
| `write`, `edit`, `read`, and no filename glued to `w`, `e`, `r` | #15 | 68 | 4 |
| `e` warns, and a failed `e` keeps the buffer | #17 | 68 | 20 |
| The help lists every command | #29 | 43 | 19 |
| The tutorial, captured from a run | #16 | 8 | 5 |
| CRLF line endings are kept | #18 | 29 | 3 |
| `g` and `v` mark lines | #19 | 160 | 36 |
| `0a`, and `a` past the end | #20 | 68 | 18 |
| An escaped delimiter is the literal character | #21 | 44 | 29 |
| `^` once, and no empty match after a real one | #22 | 84 | 55 |
| The closing delimiter is optional | #23 | 12 | 4 |
| The count, and `p`, `l`, `n` | #24 | 117 | 29 |
| `%` | #32 | 22 | 1 |
| Exit status | #26 | 96 | 7 |
| End of input warns | #33 | 26 | 1 |
| `-s` | #27 | 165 | 92 |
| The UTF-7 refusal removed | #25 | 22 | 54 |
| `rust-version` | #28 | 7 | 1 |
| The README | #31 | 16 | 10 |
| A delimiter must be one ASCII byte | none | 33 | 0 |
| The current line after `g/re/d` | #34 | 17 | 0 |
| `j` with one address | #35 | 2 | 3 |

Twenty-one issues, twenty-three commits counting the README's command table
and the delimiter fix, which had no issue because the bug it fixed never
shipped. The largest fix is `-s`, because it changed a type that every
command returns. The second largest is the marks, because it changed a data
structure. Everything else is under 120 lines, and most of the lines in every
commit are tests.

Measured on the result, against 0.3.0:

| | 0.3.0 | 0.4.0 |
|---|---|---|
| Lines of Rust | 3,625 | 4,367 |
| Unit tests | 185 | 215 |
| Tests that run the binary | 0 | 10 |
| Single-command scripts that differ from GNU ed, of 3,000 | 425 | 99 |
| Scripts of one to three commands that differ, of 3,000 | 787 | 180 |
| Fuzzer panics in a million runs | 0 | 0 |

All 99 remaining single-command differences are `s/[ab]*/-/g`, the case where
GNU ed refuses and ved follows the Seventh Edition. Of the 180, 84 are the same
case; the rest were filed as #36, and section 22.6 is about what they turned
out to be. The harness numbers use seed 1, so they are comparable with each
other and not with chapter 15's, which used a different one.

The test count rose by 40, and the bugs fixed number 21. That ratio is the
procedure in section 21.1 made visible: every fix began with a test that
failed, and most began with several, because a bug that was found by one
input usually has neighbors.

---

**Exercise 21-1.** Pick one issue and fix it in `examples/ch19-living`
without looking at this chapter's code: reproduce it, write the test with an
expected value from POSIX or the Seventh Edition, make the change. Then
compare your fix with ved's commit for it, and list what each does that the
other does not.

**Exercise 21-2.** The `first_mark` hint in section 21.5 is maintained by the
four primitives. Write a property test, with the xorshift generator from
chapter 15, that applies random sequences of `append_after`, `delete_range`,
`move_range`, and `set_mark` to a buffer, and checks after each step that no
marked line sits above `first_mark`.

**Exercise 21-3.** Run `tools/difftest.py` against `examples/ch21-fixed` with
`MAXC=3` and twelve examples. Take one difference that is not
`s/[ab]*/-/g`, decide which program is right using the order of authority in
section 20.7, and either fix ved or write down why it differs.

**Exercise 21-4.** The fuzzer's alphabet has no `w`, `e`, or `r`, so that it
cannot write files. Add `E` and `r` with filenames from a fixed set of paths
inside a temporary directory, run it, and report what it finds.

**Exercise 21-5.** Implement #38 in `examples/ch21-fixed`: `l` writes `\t`,
`\r`, and the other C escapes POSIX names, and folds long lines. Decide the
folding width and write down why, then compare your version with ved 0.4.1's,
which section 22.6 describes.
