# Appendix B: The Slices

Chapter 1 listed the twelve slices the plan divided ved into. This appendix
records how they were built: which commit holds each one in ved's history,
which crate under `examples/` holds it in this book, and which chapter covers
it. The hashes are from the `excelano/ved` repository.

## B.1 The twelve slices

| # | Slice | Commit | Date | Crate | Chapter |
|---|---|---|---|---|---|
| 1 | REPL loop, `-p`/`--prompt` | `0851663` | 2026-04-10 | `ch02-repl` | 2 |
| 2 | Buffer, `a`, `.`, `p`, `,p` | `d766818` | 2026-04-10 | `ch03-buffer` | 3 |
| 3 | Addresses, current line, `n` | `d766818` | 2026-04-10 | `ch04-addresses` | 4 |
| 4 | `w` and `q`, the double `q` | `d766818` | 2026-04-10 | `ch05-write` | 5 |
| 5a | The regex engine, step 1 | `515d4b9` | 2026-04-11 | `ch07-pike` | 7 |
| 5a | The regex engine, steps 2 to 5 | `62c33ad` | 2026-04-12 | `ch08-bre` | 8, 9 |
| 5b | `s` | `dec3f54` | 2026-04-12 | `ch10-substitute` | 10 |
| 6 | `d` and `i` | `7f3369b` | 2026-04-12 | `ch11-editing` | 11 |
| 7 | `g` and `v` | `c107e27` | 2026-04-12 | `ch12-global` | 12 |
| 8 | Long-form aliases | `a4539c4` | 2026-04-12 | `ch13-files` | 14 |
| 9 | Confirmations, `Q` | `a4539c4` | 2026-04-12 | `ch13-files` | 14 |
| 10 | `H` | `a4539c4` | 2026-04-12 | `ch13-files` | 14 |
| 11 | Files, `e`, `r` | `a4539c4` | 2026-04-12 | `ch13-files` | 13 |
| 12 | `l` | `d8d02cb` | 2026-06-11 | `ved` | 14 |

Slices two to four were one commit, and so were slices eight to eleven. The
crates for slices four, five, and onward match a commit file for file. The
crates for slices two and three, `ch03-buffer` and `ch04-addresses`, match no
commit: they were made for this book by taking the slice two-to-four commit
and removing the later slices from it, so that chapters 3 and 4 each have a
program that builds and runs at the point the chapter ends. Slices eight to
eleven have one crate between them, because the chapters that cover them, 13
and 14, describe that commit from two directions.

Slice 12 was planned with the others and built a month after the first
release, in 0.1.4, so its code is only in `examples/ved`.

The dates show the plan's slices were unequal, as chapter 1 said they would be.
Slices one to four took a day. The regex engine took two. Slices 5b to 11, seven
slices including files and the help system, were built on one day.

## B.2 After the plan

The slices ended at 0.1.0. Everything after was added in response to use:

| Version | Added | Chapter |
|---|---|---|
| 0.1.1 | `c` | 11 |
| 0.1.2 | A missing file is a new file | 13 |
| 0.1.3 | Greedy `*`, leftmost-longest | 7 |
| 0.1.4 | `l` (slice 12) | 14 |
| 0.1.5 | `j`, `m`, `t`, octal escapes, bare `s` | 10, 11 |
| 0.2.0 | Encoding warnings, the overwrite refusal | 13 |
| 0.2.1 | The command-letter panic, the tenth-group panic | 5, 9 |
| 0.3.0 | Search addresses, character-aware matching | 19 |

`examples/ved` is 0.2.1. `examples/ch19-living` is 0.3.0.

## B.3 The crates

| Crate | Lines | Tests | State of the program |
|---|---|---|---|
| `ch02-repl` | 123 | 0 | A loop that can quit |
| `ch03-buffer` | 252 | 0 | Text in and out |
| `ch04-addresses` | 487 | 0 | Addresses and ranges |
| `ch05-write` | 609 | 0 | Saving and the unsaved-buffer guard |
| `ch07-pike` | 832 | 23 | Pike's matcher, not yet connected |
| `ch08-bre` | 1,434 | 69 | The full engine, not yet connected |
| `ch10-substitute` | 1,711 | 76 | `s` |
| `ch11-editing` | 1,768 | 76 | `d` and `i` |
| `ch12-global` | 1,864 | 76 | `g` and `v` |
| `ch13-files` | 2,006 | 76 | Files, aliases, help, `Q` |
| `ved` | 2,871 | 125 | Release 0.2.1 |
| `ch19-living` | 3,625 | 185 | Release 0.3.0 |

Every crate builds and runs on its own, with `cargo run -p <crate>` from
`examples/`, and `cargo test` there runs all of them.
