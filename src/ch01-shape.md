# 1. The Shape of a Project

A program starts as a sentence. The sentence for this one is: *a clone of ed
that explains itself*.

That is enough to start, and it is not enough to build from. Between the
sentence and the first line of code there is a short piece of work that decides
most of what the program becomes, and it is the cheapest work in the project
because nothing has been written yet. This chapter is that work, and then the
mechanics of an empty Rust project.

## 1.1 Choosing something with an answer

`ed` is the original Unix line editor. It was written by Ken Thompson in 1969,
it is still in every POSIX system, and it is famous for being unusable: it
prints `?` when something goes wrong and says nothing at all when something goes
right. `grep` is named after one of its commands (`g/re/p`), `sed` is its
non-interactive descendant, and `vi` was built on top of it.

ved keeps ed's interface and replaces its silence. Where ed prints nothing, ved
prints `deleted 3 lines (2-4)`. Where ed prints `?`, ved prints `? no match`.
Where ed has no help, ved has an `H` command that prints the command reference
without leaving the editor. Scripts written for ed run against ved unchanged.

The useful property here is not the idea. It is that somebody else wrote the
specification. POSIX defines what `ed` does, GNU ed is sitting on the machine to
compare against, and every question of the form "what should happen if the user
types this?" has an answer that can be looked up or run. A first project that
invents its own semantics spends most of its time in design arguments with
itself. A clone spends that time writing code.

That is worth holding onto when picking something to build: a project with an
external specification is dramatically easier than a project without one, and
the specification does not have to be a document. It can be an existing program,
a file format, a protocol, or a wire API.

## 1.2 Deciding the constraints first

Before any code, ved's `PLAN.md` recorded a table of decisions. It is short, and
every row in it closed a question that would otherwise have been reopened
weekly:

| Decision | Choice |
|---|---|
| Language | Rust, edition 2024, pure stdlib (no dependencies, ever) |
| Interface | CLI, line-oriented. No TUI. |
| ed compatibility | Strict drop-in. Scripts written for real ed must work unchanged. |
| Long-form aliases | Long names for commands without inline delimited arguments |
| Errors | Full English replacing ed's bare `?` |
| Confirmations | Echo what each command did |
| Build output | Single static binary. No runtime, no shared libraries. |

The first row is the one that does the most work, so it is worth being precise
about what it is and is not.

**"No dependencies, ever" is not a recommendation.** For most programs it is the
wrong call. Reaching for `clap` to parse arguments and `regex` to match patterns
is the correct default, and a book that told you otherwise would be teaching you
to waste your life. ved adopted the constraint for specific reasons: it was a
learning project, and code you import teaches you nothing; a regex engine is one
of the good exercises in computer science and ed needs a dialect (POSIX Basic
Regular Expressions) that the standard `regex` crate does not implement anyway;
and a text editor that might be installed as `$EDITOR` on a server has a real
argument for a zero supply-chain footprint.

What matters more than the specific choice is that the constraint was *chosen*
and *written down*. A constraint you have decided on does work for you. It
answers questions in advance, and it answers them the same way each time. Twice
in ved's history a feature was proposed that would have required a dependency,
and the constraint decided both cases in a minute instead of a week. Chapter 19
covers what that cost.

Notice also the rows that are negative. "No TUI" and "no runtime" are not
features; they are the absence of features, recorded so that the absence is a
decision rather than a thing nobody got around to. A plan that lists only what
a program will do is half a plan.

## 1.3 Slicing

The build plan was twelve slices:

| # | Slice |
|---|---|
| 1 | Cargo project, REPL loop, `-p`/`--prompt` flag |
| 2 | Buffer + `a` (append) + `.` + `p` + `,p` |
| 3 | Addresses (`1p`, `$p`, `2,4p`) + current-line tracking + `n` |
| 4 | `w` (write) and `q` (quit), with the double-`q` dance |
| 5a | BRE regex engine (`src/bre.rs`, hand-written) |
| 5b | Substitute: `s/old/new/[g]` wired to the engine |
| 6 | `d` (delete) and `i` (insert) |
| 7 | `g/pattern/cmd` and `v/pattern/cmd` |
| 8 | Long-form command aliases |
| 9 | Friendly errors, confirmations, `Q` (force quit) |
| 10 | `H`/`help` command reference |
| 11 | Open existing files, `e` (edit), `r` (read) |
| 12 | `l` (list) — visible rendering of non-printing characters |

The rule that makes this list useful is stated in the plan itself: *each slice
is a complete working program with one more capability than the previous*.

This is not the same as a task list. "Write the buffer module", "write the
address parser", "write the command dispatcher" is a decomposition by component,
and the program does not run until the last one is finished. Slices are a
decomposition by capability: at the end of slice 2 you can type text into a
buffer and print it back, which is not an editor but is a program, and you can
tell whether it works by using it.

Two things follow from that, and both of them are why the technique is worth the
small awkwardness of building things you will replace.

You get feedback at the end of every slice instead of at the end of the project.
Slice 3 of ved discovered that the dispatcher needed `&mut Buffer` rather than
`&Buffer`, because printing a line changes ed's notion of the current line. The
plan written during slice 2 had predicted that this would first be needed in
slice 6. Being wrong about that in slice 3, with two hundred lines written, cost
ten minutes.

And a slice that works can be abandoned. Not every project finishes. A project
that stopped after slice 7 would still be a usable text editor with a regex
engine, which is a far better outcome than seven finished components and no
program.

The slices were also, deliberately, unequal. Slice 5a is a regex engine and took
longer than slices 1 through 4 together. Slice 8 is a lookup table. Sizing
slices evenly is an instinct worth resisting; the unit is "one more thing the
program can do", not "one afternoon".

## 1.4 The empty project

```text
$ cargo new ved
    Creating binary (application) `ved` package
$ cd ved
```

That produces `Cargo.toml`, `src/main.rs` with a hello-world, and a git
repository with a `.gitignore` containing `/target`. Three of those four are
right.

`Cargo.toml` wants editing, and the fields matter more than they look:

```toml
[package]
name = "ved"
version = "0.2.0"
edition = "2024"
description = "The verbose ed — an ed clone with friendly errors and a built-in help system"
homepage = "https://excelano.com/ved/"
repository = "https://github.com/excelano/ved"
license = "MIT"
authors = ["David M. Anderson"]
keywords = ["ed", "editor", "line-editor", "regex", "bre"]
categories = ["command-line-utilities", "text-editors"]

[dependencies]
```

The `name` is the binary's name, the crate's name on crates.io, and the string
people will type forever. It is worth five minutes and a check that it is not
taken. `edition` selects a dialect of the language; 2024 is what this program
uses and it sets the minimum compiler version to 1.85.

Everything from `description` down is metadata, and none of it is needed to
build. It is needed to publish, it is read by `cargo search`, `apt show`, and
the crates.io page, and it is the only description of the program most people
will ever see. Chapter 17 comes back to it. Filling it in now costs nothing;
discovering at release time that `license` is missing costs a re-tag.

The empty `[dependencies]` section stays empty for the life of this program.

## 1.5 What goes in the repository

By the end, ved's repository looks like this:

```text
ved/
├── Cargo.toml          the manifest
├── Cargo.lock          committed, because this is a program
├── LICENSE             MIT
├── README.md           the primary interface for everyone who is not you
├── PLAN.md             the design table and the slice list
├── BACKLOG.md          what is not built, and why
├── RELEASING.md        what to do to cut a release
├── SECURITY.md         where to report a vulnerability
├── Makefile            build and install from source
├── install.sh          download the release binary
├── src/
│   ├── main.rs         REPL, dispatch, the commands
│   ├── buffer.rs       the text buffer
│   ├── address.rs      the address parser
│   ├── bre.rs          the regex engine
│   └── enc.rs          encoding detection
└── .github/workflows/  CI, release, packaging
```

Most of that arrives in later chapters. Two decisions about it are worth making
on day one.

**Commit `Cargo.lock`.** For a program, the lock file records the exact
dependency versions that were built and tested, so a build six months from now
is the same build. For a library, the convention is the opposite, because a
library's lock file is ignored by anything that depends on it. ved has no
dependencies, so its lock file is four lines, and it is committed anyway.

**Use git from the first commit**, and write commit messages as if someone will
read them, because in this book somebody does: the example crates were extracted
from ved's history, commit by commit. Chapter 16 makes the case that commit
messages are the right home for a particular kind of writing that otherwise
pollutes the source.

## 1.6 Where the reasoning goes

One habit is worth adopting before there is any code to apply it to, because it
is much harder to retrofit.

A repository documents what is *true*. A decision log records what *changed*.
The two read identically and behave completely differently once time passes.

"Compare modification time, not bytes, because the server rewrites the file on
upload" is a statement about how the world is. It belongs in the code or the
README, and it stays correct until the world changes.

"Switched from byte comparison to modification time in May, after the hash
proved unreliable" is a statement about a change. The moment a third approach
arrives, that sentence is describing a dead decision in the present tense, sitting
in a file where a reader — or a future you at two in the morning — will load it
as current guidance.

The test is tense, not topic. Naming a former state, an alternative that was
rejected, or the date a decision was taken is the tell. Those sentences go in
commit messages, in a decision log outside the repository, or nowhere. The
`PLAN.md` shown above sits right on the line: its design table is present tense
and stays, and its slice-by-slice architectural notes are a decision log that
was published by accident and later trimmed.

## 1.7 What this chapter bought

There is still no code. What there is: a program with a specification that
somebody else wrote, a constraint that will answer a dozen future questions
identically, twelve slices each of which leaves a working program, a manifest
that can be published without edits, and a rule about where sentences go.

That is a morning's work, and it is the difference between a project and a
directory.

---

**Exercise 1-1.** Pick something to build and write its one-sentence
description. Then find its external specification: the existing program, the
RFC, the file format document. If there isn't one, pick something else for your
first project.

**Exercise 1-2.** Write the decision table. Include at least two negative rows —
things the program will not do — and one constraint you are choosing rather than
being forced into.

**Exercise 1-3.** Slice it. The test for each slice is whether you can describe
what the program does at the end of it in a sentence that does not contain the
word "and then". Aim for the first slice to be one day's work and the second to
be usable.

**Exercise 1-4.** Look at the last project you abandoned. Was it sliced by
capability or by component? If it had been sliced by capability, what would have
been left behind?
