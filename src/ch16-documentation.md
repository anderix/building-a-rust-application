# 16. Documentation

At 0.2.1, ved is described in seven places. The README, 226 lines on GitHub and
crates.io. A tutorial, `docs/index.html`, published on the project's website.
The `H` command and `ved --help`, inside the program. Module header comments
and 180 lines of doc comments in the source. Seventy-six commit messages. And a
`PLAN.md` and `BACKLOG.md` that live in the author's working copy and are no
longer committed.

Each of them has a different reader and is true for a different length of
time, and most of what goes wrong with documentation is putting a sentence in
the place whose reader or lifetime does not match it. This chapter goes through
the seven, checks what each one says against what the program does, and ends
with the rule chapter 1 introduced and promised this chapter would argue for:
where the reasoning goes.

## 16.1 The README

```text
# ved — the verbose ed
## Why
## Install
### Debian and Ubuntu
### Homebrew
### Prebuilt binary (Linux and macOS)
### Windows
### Cargo
### Build from source
## Quick start
## Commands
## Addresses
## Regular expressions
## As your default editor
## Implementation
### Limitations worth knowing
## License
```

Chapter 1 called the README *the primary interface for everyone who is not
you*, and the order of its sections is the order of that reader's questions.
What is it, why would I want it, how do I get it, how do I start, what are the
commands. The implementation comes near the end, because it is for the smaller
group who got that far, and the license is last.

The opening paragraph does the most work and is the most often read. It names
the program, says what it is a clone of, says what it adds, and states the
compatibility claim: *close compatibility so most scripts written for real ed
run against ved unchanged*. That sentence was rewritten from an earlier one,
*strict drop-in compatibility*, when its author listed what ved does not have:
search addresses, `u`, `!`, `=`, `k`, `f`, and the repurposed `H`. The
commit that made the change says so, and it is the README's best sentence,
because it is the one that was checked.

The quick start was not:

```text
$ ved myfile.txt           # open a file
135                        # ved prints the byte count
* ,p                      # print all lines
Hello world
This is a test file
* 1s/Hello/Goodbye/       # substitute on line 1
Goodbye world
* g/test/n                # print lines matching "test" with numbers
2	This is a test file
* w                        # write back to the file
37
* q                        # quit
```

Run it, with a file containing those two lines:

```text
$ ved myfile.txt
32
,p
Hello world
This is a test file
1s/Hello/Goodbye/
Goodbye world
g/test/n
2	This is a test file
w
wrote 34 bytes to myfile.txt
q
```

The byte counts are wrong, which is minor. The prompt is shown although ved has
none without `-p`, which will confuse the first reader who tries it. And `w`
is shown printing `37`, ed's bare byte count, where ved prints `wrote 34 bytes
to myfile.txt`. The one line in the example that demonstrates ved's purpose,
saying what it did, shows ed's behavior instead. The transcript was written
from what the author expected rather than copied from a terminal.

The *Limitations worth knowing* section has the same problem in a different
form. It says lines are *delimited by `\n` (or `\r\n`)*, which is true on the
way in and leaves out that chapter 13's CRLF file comes out with `\n`. It says
`w` refuses to overwrite a UTF-7 file because *saving edits as UTF-8 would
silently corrupt the source*, which chapter 13 showed cannot happen on the path
that reaches the refusal. Chapter 9 found a third sentence there, *UTF-8 text
with multi-byte characters works*, which was false for any substitution that
split a character. Each was true of what the author intended, and none was
checked against the program.

## 16.2 The tutorial

The tutorial is the longest piece of documentation ved has, and it is the best
written: it explains what a line editor is, why a newcomer should care, and
walks through a session one command at a time, with the long-form names, so
that someone who has never seen ed can follow it. Then, under *Working with
files*:

```text
* write notes.txt
wrote 37 bytes to notes.txt
```

Chapter 14 found that `write` is not implemented. Typed into ved, that line
saves the buffer to a file named `rite notes.txt`:

```text
$ ved -p '* '
* append
The quick brown fox
.
* write notes.txt
wrote 20 bytes to rite notes.txt
```

The tutorial also teaches `edit filename`, which opens a file named `dit
filename` and, because of chapter 13's `e`, discards the buffer first, and
`read filename`, which reads from a file named `ead filename`. A reader who
follows the tutorial carefully, in order, loses the work they typed in the
earlier sections at the point where they were taught to save it.

Further down, the same page says *the commands that already use
delimiters — s/old/new/, g/pattern/cmd, v/pattern/cmd, w filename, e filename,
r filename — have no long form*. The tutorial contradicts itself, and the
second statement is the true one. The first was written from the plan, which
listed `write`, `read`, and `edit` as aliases; the second from the code. The
page is unchanged in the repository at 0.3.0.

That is the most expensive documentation bug in this book, and its cause is
simple. Every terminal session in the tutorial was typed rather than captured.
The preface of this book makes a promise its author found necessary for the
same reason: *terminal sessions show what the program actually printed*. A
transcript in documentation is a claim about the program's behavior, the same
kind of claim a test makes, and it needs the same treatment as a test: it must
be produced by running the program, and re-produced when the program changes.

Rust has this for code. `rustdoc` compiles and runs the examples in doc
comments, and `mdbook test` does the same for a book's code blocks, which is
why every fragment in this book is tagged `rust,ignore`: it marks the block as
not meant to compile on its own. There is no standard tool for terminal
sessions, but for a program that reads standard input the tool is short. Split
each session into the lines the user typed and the lines the program printed,
feed the first to the binary, and compare its output with the second. Chapter
15's `session` function is most of it. A documentation test that fails when
the tutorial is wrong is the same kind of check as a differential test against
ed, with the documentation as the reference.

## 16.3 Comments that describe a plan

Search the shipped source for the word *slice*:

```rust,ignore
// `filename` remembers the last filename used by `w`, so a bare `w`
// after `w foo.txt` writes back to foo.txt. Slice 11 will also set
// this when ved is invoked with a filename argument.
```

```rust,ignore
// Address forms recognized in slice 3:
//   5         absolute line number
// ...
// Search addresses (/foo/, ?foo?) wait for slice 5 when the regex
// engine arrives. Compound expressions like $-5 also wait.
```

```rust,ignore
            Action::Error(msg) => {
                // ved's friendly-error format. Slice 9 will route
                // every error path through this variant and may add
                // an H toggle to suppress the message like real ed.
                let _ = writeln!(out, "? {msg}");
```

Three comments in `buffer.rs`, `address.rs`, and `main.rs` describe the
program as it was during the slice in which they were written, and refer to
future slices that have since happened. Slice 11 did set the filename. Slice 5
did not bring search addresses, which chapter 4 discussed. Slice 9 did not
route anything, because every error already went through the variant, and `H`
became the help command rather than a toggle.

None of these comments was wrong when it was written. Each is a sentence in the
future tense, and the future arrived. That is the failure chapter 1 described:
a sentence about a change, sitting in a file that is read as a description of
the present. A reader opening `buffer.rs` today is told that ved cannot yet
open a file named on the command line.

The module header of `bre.rs` is the larger case:

```rust,ignore
// ved's BRE engine — compiled, with capturing groups and
// backreferences.
//
// Step 1 was a direct port of Pike's recursive matcher: three
// functions, no compile step, the pattern itself is the program.
// That original is preserved in git history (commit c53f626).
//
// Step 2 introduced a compile step (closer to Ken Thompson's
// original ed) and bracket expressions ([abc], [^abc], [a-z]).
//
// Step 3 wrapped the compiled form in a Regex struct, returning
// match positions via a Match struct.
//
// ...
```

Twenty lines of history, five steps of how the engine was built, ending with a
known limitation. It is interesting, and this book is in part a longer version
of it, but it is a commit log living in the source. And it has decayed in a way
that a commit log cannot: commit `c53f626` does not exist in ved's repository.
ved began inside another repository and was moved to its own, and that commit
did not come with it. The reference points at a history the reader cannot
reach.

What the header should say is what the module is now. A present-tense version
is shorter:

```rust,ignore
// ved's regex engine: POSIX Basic Regular Expressions, compiled to a
// flat list of elements and matched by recursive backtracking.
//
// Supports literals, `.`, `*`, `^`, `$`, bracket expressions with
// ranges and negation, `\(...\)` groups, `\1`-`\9` backreferences,
// and `\0NN` octal escapes. Matching is leftmost-longest. `*` applies
// to a single atom only; `\(...\)*` is not supported.
```

Every sentence there will stay true until the code changes, and when the code
changes, the comment is beside it to be changed too. The history went into the
commit messages when it happened, and the README says *the full history is in
the git log*, which is the right pointer.

## 16.4 Doc comments

The doc comments are, by count, most of ved's documentation: 180 lines of
`///` across five files. Most of them are good, and the good ones share a habit.
They say something the code does not:

```rust,ignore
    /// Called by `w` after a successful whole-buffer write. A
    /// partial-range write does NOT clear modified, since the
    /// buffer still differs from any single file on disk.
    pub fn mark_saved(&mut self) {
```

The function body is one assignment. The comment is the design, and it is what
stops a later reader from calling `mark_saved` after `1,2w`.

The bad ones share a different habit. They describe what the author intended
the function to do, and nothing compares the description with what it does.
`run_edit`'s doc comment says it *warns if the buffer has unsaved changes (like
q does)*, and it does not. `run_join`'s says a single-line range is *a silent
no-op*, and it prints a blank line. `enc.rs`'s header says its rules are
*byte-perfect signatures only, no heuristics*, and its UTF-7 rule is a
heuristic. In all three cases the comment states the requirement correctly and
the code does not meet it, which means the comment is useful: it is the bug
report, written in advance, by the person who knew what the code was for.

The lesson is not that doc comments should be trusted less. It is that a doc
comment which states a requirement is a test that nobody wrote. *Warns if the
buffer has unsaved changes* is one assertion away from being checked, and if
the assertion had been written beside the comment, the function would have
failed it on the day it was written.

## 16.5 Commit messages

Chapter 1 said this chapter would make the case that commit messages are the
right home for a kind of writing that otherwise ends up in the source. Here is
one:

```text
Treat a missing file as a new file, matching ed

Opening ved on a path that doesn't exist no longer prints an error and
forgets the name. It now remembers the filename, leaves an empty
unmodified buffer, and prints "<name>: new file", so a later bare w
creates the file. This matches ed, which never refuses a missing file
and creates it on first write rather than at startup.

Only ErrorKind::NotFound gets this treatment; permission errors,
is-a-directory, and the like stay real errors. The same path covers the
e command, since both route through load_file. load_file now returns a
LoadOutcome enum (Loaded/NewFile) so callers can word the two cases.

Adds the first filesystem-touching tests, in pure stdlib: one asserts a
missing file loads as NewFile with the name remembered and nothing
written to disk, the other asserts a directory still errors.
```

It says what the program did before, what it does now, why, where the boundary
of the change is, what else it affects, and how it was tested. Every one of
those is a statement about a change, and every one would be wrong in the source
by the next release: "no longer prints an error" describes a former state,
"now returns" is relative to a version nobody will be looking at, and "the
first filesystem-touching tests" stops being true when the second set arrives.
In a commit message none of that decays, because a commit message is read as
what it is, a record of one change at one moment. `git blame` on the line in
`load_file` leads to it, and it is still accurate.

Another, from the commit that made `*` greedy:

```text
Updates two existing position tests (position_dot_star,
position_digits) that were pinned to the old behavior, and
adds four new tests covering the greedy + back-off cases.
```

That is chapter 15's point about tests that agree with the code, recorded at
the moment it was discovered: two tests had to change because they asserted
the bug. The source cannot say that, since after the commit the tests assert
the right thing and there is nothing to see. The commit is the only place the
fact can live.

That is the argument in one sentence. A repository's files describe what is
true now, and its history describes how it got that way, and a sentence belongs
in whichever one its tense says. Chapter 1's test: a sentence that names a
former state, a rejected alternative, or a date is about a change. The three
comments in section 16.3 are in the future tense, which is the same failure
pointing the other way, a sentence about a change that has not happened yet.
They belonged in the plan.

The subject lines matter separately, because more people read them than read
the bodies. In ved's history they are imperative and specific: *Refuse to
overwrite a file loaded as non-UTF-8*, *Take the command letter as a character,
not a byte*, *Record only the nine groups a BRE can address*. Chapter 17 uses
them as release notes, which is a reason to write them for the person
installing the release rather than for the person who wrote the code.

## 16.6 The plan and the backlog

`PLAN.md` was the first document in the project, and chapter 1 quoted its
design table and its slice list. Its design table is present tense, a list of
decisions that are still in force. Its slice list is a status board, with each
slice marked done. And below that it accumulated something else: slice-by-slice
notes on where code should go, written before the code existed, such as the
note for `l` that names line numbers in `main.rs` where the new arm should be
added.

Those notes were committed until July, when a commit titled *stop publishing
local planning notes* removed `PLAN.md` and `BACKLOG.md` from the repository.
Chapter 1 described that as a decision log published by accident and later
trimmed, and the trimming was right. A note saying where to add code is useful
for the hour before the code is written and misleading forever after, because
the line numbers it cites now point at something else.

The status board had the other problem. It marks slice 8 done, and its
description of slice 8 is *long-form command aliases (`append`, `write`,
`quit`, etc.)*, and `write` is not implemented. The tutorial's `write notes.txt`
was written from the plan. A status board is a claim that something is true of
the code, and like the README's quick start, nobody checked it.

`BACKLOG.md` is the other document that lives outside the repository, and it is
chapter 19's subject: the list of what is not built, and why.

## 16.7 Seven places, one fact

The help text, the README's command table, the tutorial, `--help`, the doc
comments, the plan, and the code each state, in part, what commands ved has.
They disagree. The help omits `e`, `r`, and `Q`. The README's table has them.
The tutorial teaches three aliases that the code does not have. The plan
says they exist.

The rule that prevents this is to state each fact once and point to it from
everywhere else. For a command reference, that means one source, and chapter
14's command table is the code version of it. For a README, it means the
command table and the help come from the same list, or the README says *type
`H` in ved for the full reference* and does not repeat it. For a tutorial, it
means every session is a captured transcript. For the design and the status,
it means a plan describes intentions and the code describes facts, and when the
two disagree, the code is the one that is right about what is true.

## 16.8 What the documentation bought

A program that newcomers can learn, which is a large part of why ved exists. The
README answers the questions in the order people ask them. The tutorial
explains a line editor to someone who has never used one. The doc comments,
including the ones the code does not live up to, say what the functions are
for. The commit messages are a complete and accurate history of why the program
is the way it is.

And the parts that were never run against the program were wrong in ways that
the program's own output would have shown in a second. The quick start's byte
counts, the tutorial's `write`, the plan's status, the limitation that was not
a limitation. Documentation is code that nothing compiles. The parts of it that
can be executed should be, and the parts that cannot should be written in the
tense that keeps them true.

---

**Exercise 16-1.** Write the documentation test from section 16.2: parse the
terminal sessions out of the tutorial's HTML, run each one through ved, and
report every line that differs. Run it against the tutorial and count the
failures.

**Exercise 16-2.** Rewrite the three planning comments from section 16.3 in the
present tense, and the `bre.rs` header as in that section. Then grep the source
for *will*, *later*, *yet*, and *now*, and decide for each hit whether it
describes the present.

**Exercise 16-3.** For each doc comment in `main.rs` that states a requirement,
write the test that checks it. Count how many of the tests pass the first time.

**Exercise 16-4.** Rewrite the README's quick start from a captured session.
Then decide whether the README should keep its command table or point to `H`,
and what that choice does to the help test from chapter 14.

**Exercise 16-5.** Read the last twenty commit messages of a project you work
on. For each one, decide whether its body says something the diff does not, and
whether any sentence in it should have been a comment instead, or the other way
round.
