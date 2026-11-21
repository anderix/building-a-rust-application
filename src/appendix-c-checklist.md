# Appendix C: From Empty Directory to Installed

The book in one table per stage, for a program of your own. Each row is a step,
what it produces, and the chapter that explains it. The steps are in the order
ved took them, adjusted where ved's order turned out to be wrong.

## C.1 Before any code

| Step | Produces | Chapter |
|---|---|---|
| Write the one-sentence description | A sentence to test every later decision against | 1 |
| Find the external specification: a standard, a file format, an existing program | Answers to "what should this do?" that are not opinions | 1 |
| Write the decision table, including what the program will not do | Constraints that answer questions in advance | 1 |
| Slice by capability, each slice a working program | A plan that can stop at any point and leave something usable | 1 |
| `cargo new`, then fill in every metadata field in `Cargo.toml` | A manifest that can be published without edits | 1, 17 |
| Decide where reasoning goes: commit messages, not source | A repository that describes the present | 1, 16 |

## C.2 Building it

| Step | Produces | Chapter |
|---|---|---|
| Make `main` return an `ExitCode`, and parse arguments strictly | Exit statuses scripts can rely on | 2 |
| Make the command function return a value describing what to do | A command language testable without a terminal | 2 |
| Put each invariant behind a module boundary | State that only a few methods can change | 3 |
| Keep one place where 1-indexed meets 0-indexed, and return `Option` for user-supplied positions | No off-by-one at the boundaries, no panic on a typo | 3 |
| Separate parsing from resolving | Errors reported against the right cause | 4 |
| Let each command choose its own default | No flag parameters on a shared function | 4 |
| Index strings by character, never by a literal byte offset into input you did not inspect | No panic on the first non-ASCII character | 5 |
| Build replacements before swapping them in | A failed operation that leaves the old state intact | 5, 13 |
| Hard parts first in isolation, with their own tests, before wiring them in | An engine that can be trusted when it is connected | 7, 8, 9 |
| Keep marks, cursors, and positions attached to the things they refer to | Loops that survive the operations they perform | 12 |
| Treat input from files as foreign: line endings, encodings, missing final newlines | Files that come back the way they went in | 13 |

## C.3 Saying what happened

| Step | Produces | Chapter |
|---|---|---|
| Write every message in the user's terms as the command is written | No sweep at the end | 5, 14 |
| Route each kind of output through one function | A consistent voice, and a plural that is right everywhere | 14 |
| Distinguish requested output, confirmations, and errors in the types | A `-s` flag that is one line in the loop | 14 |
| Exit non-zero when a non-interactive run had an error | Scripts that can detect failure | 14 |
| Reject what you do not understand, in flags, suffixes, and arguments | Typos that fail instead of meaning something else | 10, 11, 14 |

## C.4 Checking it

| Step | Produces | Chapter |
|---|---|---|
| Unit tests at the command boundary, asserting on state, not wording | A suite that does not fight message changes | 15 |
| One integration test per responsibility of the outer loop | Coverage of the layer unit tests cannot see | 15 |
| Expected values from the specification, never from running the code | Tests that can disagree with the code | 15 |
| A differential harness against the reference implementation | The disagreements nobody thought to test | 15 |
| A fuzzer over the input language | The crashes nobody would type on purpose | 15 |
| A benchmark of the data structure at ten times the expected size | A measured boundary instead of a feeling | 15 |
| `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` in CI | A gate between a laptop and a release | 17 |
| Build on the declared minimum Rust version as well as stable | A `rust-version` that is true | 17 |

## C.5 Writing it down

| Step | Produces | Chapter |
|---|---|---|
| README in the order of the reader's questions: what, why, install, start, reference | The interface for everyone who is not you | 16 |
| Every terminal session captured from a real run, and re-run when the program changes | Documentation that cannot teach a command that does not exist | 16 |
| Module comments in the present tense; history in commit messages | Comments that stay true | 16 |
| Each fact stated once, and every other place pointing to it | Seven descriptions that cannot disagree | 16, 18 |
| A manual page | `man` working after `apt install` | 18 |
| A backlog sorted by why each item is not done | Decisions that are not re-argued | 19 |

## C.6 Releasing it

| Step | Produces | Chapter |
|---|---|---|
| Pick the version by what changed for users, including behavior and output | A number that means something | 17 |
| Run everything checkable before the tag | No release that needs an immediate patch | 17 |
| Write the release notes from typed commit subjects | A release page that says what changed | 16, 17 |
| Tag, build every target, publish checksums | A release anyone can download and verify | 17 |
| Publish to crates.io | `cargo install` | 17, 18 |

## C.7 Getting it to people

| Step | Produces | Chapter |
|---|---|---|
| A Makefile whose `install` does not build | `sudo make install` that works | 18 |
| A `.deb` with correct `depends`, and a signed apt repository | `apt install`, `apt upgrade`, and authenticity | 18 |
| A WinGet manifest generated from `Cargo.toml` | `winget install`, with a description that matches | 18 |
| Add each channel only when someone needs it, with a way to tell whether they still do | Channels you can afford at every release | 18 |
| When a channel ends, say so where its users will look | Users who know where to go next | 19 |

## C.8 After

| Step | Produces | Chapter |
|---|---|---|
| Read the program closely, as if writing a book about it | The findings the tests missed | 19 |
| Put each finding in the tracker the day it is made | Findings that get fixed | 19 |
| Stop data loss with a small fix first, then fix the design | A safe place to make the large change | 19 |
| Delete backlog items when they ship, and recheck the reasons on the rest | A backlog that describes what is left | 19 |
