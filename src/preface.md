# Preface

This book is about the distance between a program that runs and software
someone installs.

It assumes you have read an introduction to Rust and can write a function, a
struct, an enum, and a `match` without looking them up. It does not re-teach
ownership. What it teaches is everything that comes after the language: how to
decide what to build, how to grow a program in stages that each work, how to
structure the parsing and the state, what to test and what not to, what a
program should say when it fails, and how a binary on your laptop becomes a
binary on someone else's.

Most Rust material stops at a working program. That is roughly halfway. The
second half has its own decisions, and they are made badly by default: the
program prints raw `Debug` output at the user, the README is a paragraph, the
only way to install it is `cargo install`, and the argument parser accepts
things the help text never mentions. None of that is hard. It is just unwritten
down.

## The example

The book builds one program the whole way through, and it is a real one.

`ved` is a clone of `ed`, the original Unix line editor. It is about 2,800
lines of Rust in five modules, it has no dependencies at all, it compiles in
half a second, and it ships on crates.io, in a Debian apt repository, and
through WinGet. The version in this book is the version people
install.

It was chosen for three reasons. It is small enough to show completely, so
nothing is hand-waved. It is old enough in design to have a specification
somebody else wrote, so the question "what should this do?" has an answer that
is not a matter of taste. And it contains, in the middle, a regular expression
engine — a genuinely hard piece of work that has to be built rather than
imported, which is where a book about building applications ought to spend
three chapters.

The constraint that shapes the whole program is that ved uses nothing but the
standard library. That is not advice. Chapter 1 explains why the constraint was
chosen and Chapter 19 explains the two features it has cost. But it means that
every line in this book is a line you can read, including the argument parser
and the regex engine, and there is no chapter where the answer is "add this
crate and move on".

## What is written so far

Chapters 1 to 17 are here: from an empty directory to a working editor with
addresses, a command loop, its own regular expression engine, the commands
that change text and move lines, the global command, files, what the program
says, how it is tested and documented, and how it is released. Chapters 18 and
19 and the appendices are still being written, and they show in the contents
without links until they are done. Where a chapter here refers ahead to one of
them, it is describing a chapter still to come.

## How the program is built

ved was built in twelve slices. A slice adds one capability and leaves a
complete, working program behind it: slice one is a REPL that can quit, slice
two can hold text, slice three understands addresses, and so on to slice twelve.
This is not a retelling. It is how the program was actually written, and the
slice list in the repository's `PLAN.md` was written before the code.

Each chapter here takes a slice or two and builds it. The code is the real
code.

## Running the code

The program at eleven points in its history lives under `examples/` in the
[book's repository](https://github.com/anderix/building-a-rust-application),
one crate per checkpoint, and each one builds and runs:

```text
cd examples
cargo run -q -p ch03-buffer
cargo run -q -p ch10-substitute -- -p '* '
cargo test
```

Not every chapter ends at a checkpoint, because some of the original commits
covered more than one slice. Where a chapter's code is not its own crate, the
text says which crate to look in. The last one, `ved`, is the shipped program.

Type your own version alongside if you want to; that is the best way to read
this. If yours stops compiling, diff it against the next checkpoint.

## Conventions

Code that is a fragment of a larger file, which is most of it, is tagged so
that `mdbook test` does not try to compile it on its own. Terminal sessions
show what the program actually printed. Where the book states a number — a
binary size, a test count, a build time — it was measured on the machine this
was written on, not remembered.

Every chapter ends with exercises. They are the ones worth doing: most of them
are open questions in ved's own backlog.

## The other two books

This is the third of three. [*Programming in Rust*](/rust/) teaches the language to
someone who already programs. [*The Rust Thought Process*](/rust/thought-process/) teaches the model
behind it — ownership as a design question rather than a compiler rule. Either
one is enough preparation for this book, as is any other careful introduction.

Where this one overlaps them it is deliberate: the same idea looks different
when it shows up in a program you are shipping than it does in a chapter about
the idea.
