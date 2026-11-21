# Appendix A: The Finished Program

This is a map of `examples/ved`, which is ved 0.2.1, the program chapters 1 to
18 describe. It is meant to be read with the source open beside it. Every
number here was measured from that crate.

## A.1 The files

| Module | Lines | Code | Tests | Owns |
|---|---|---|---|---|
| `main.rs` | 1,287 | 928 | 359 | The loop, the arguments, the dispatcher, and every command |
| `bre.rs` | 1,064 | 467 | 597 | The regex engine and the replacement expander |
| `address.rs` | 233 | 233 | 0 | Address parsing and resolution |
| `buffer.rs` | 156 | 156 | 0 | The text and the state that belongs to it |
| `enc.rs` | 131 | 94 | 37 | Encoding detection for files that are not UTF-8 |
| **Total** | **2,871** | **1,878** | **993** | |

"Tests" is everything from the `#[cfg(test)]` line to the end of the file.
There are 125 tests: 90 in `bre.rs`, 32 in `main.rs`, and 3 in `enc.rs`. The
crate has no dependencies, and a release build takes about half a second.

## A.2 The types

**`Buffer`** (`buffer.rs`, chapters 3 and 5). The lines as a `Vec<String>`, the
current line, whether the text differs from the file, the remembered filename,
the last substitution, and the record of a file loaded as non-UTF-8. Every
field is private. Lines change only through `append_after`, `delete_range`, and
`replace_line`.

**`Address`, `Spec`, `Range`** (`address.rs`, chapter 4). `Address` is one
address as typed: a number, `.`, `$`, or an offset from the current line.
`Spec` is what the user typed in front of a command, zero, one, or two
addresses. `Range` is what a `Spec` means against a particular buffer, a pair
of 1-indexed, inclusive line numbers.

**`Action`** (`main.rs`, chapters 2, 3, 5, and 6). What the loop should do
after a command: `Quit`, `ForceQuit`, `EnterInputMode`, `Done`, `Print(String)`,
or `Error(String)`. Every command returns one, and only the loop acts on it.

**`PrintMode`** (`main.rs`, chapter 14). `Plain`, `Numbered`, or `List`, for
`p`, `n`, and `l`, which share one function.

**`LoadOutcome`** (`main.rs`, chapter 13). `Loaded(bytes)` or `NewFile`, so that
the caller can tell an empty file from a missing one.

**`Regex`, `Match`, `Element`, `Atom`, `Caps`** (`bre.rs`, chapters 7 to 9). A
compiled pattern is a flat `Vec<Element>`. An `Element` is one atom, a starred
atom, an anchor, a group marker, or a backreference. An `Atom` is a literal
byte, `.`, or a bracket class. `Caps` holds nine group positions and is copied
on every recursive call, which is how backtracking discards failed captures. A
`Match` is a pair of byte offsets plus the captures.

**`Encoding`** (`enc.rs`, chapter 13). UTF-7, UTF-16 little-endian, or UTF-16
big-endian, as detected by a byte-order mark or the UTF-7 marker.

## A.3 How a line of input travels

`main` calls `parse_args`, which returns a prompt and a filename, and passes
both to `run_repl`. If there is a filename, `run_repl` loads it with
`load_file` and prints the byte count. Then it loops.

Each iteration writes the prompt, unless the loop is in input mode, and reads
one line. It strips the `\n` and any `\r` before it, and nothing else. In input
mode, a lone `.` leaves the mode and any other line is appended after the
current line with `Buffer::append_after`. In command mode, the line goes to
`dispatch`.

`dispatch` trims the line and handles bare Enter, which prints the next line.
It calls `Spec::parse`, which takes any addresses off the front and returns the
rest. If nothing is left, the command is a bare address and prints that line.
Otherwise the rest is matched in two stages. The first stage matches the whole
remainder exactly, for commands that take no argument and for their long names:
`q`, `a`, `i`, `c`, `d`, `p`, `n`, `l`, `j`, `H`, `Q`. The second takes the
first character as the command letter and passes everything after it as the
argument: `e`, `r`, `g`, `v`, `s`, `w`, `m`, `t`. Anything else is `unknown
command`.

Each command resolves its `Spec` with the default it needs, changes the buffer
through its methods, and returns an `Action`. `g` and `v` call `dispatch`
again, once per marked line. Back in the loop, the `Action` is printed, turned
into a `?` line, used to enter input mode, or used to quit, and every arm except
`Quit` resets the unsaved-buffer warning.

## A.4 Commands

| Command | Long form | Default address | Implemented by | Chapter |
|---|---|---|---|---|
| (address alone) | | | `run_print` | 4 |
| Enter | | `.+1` | `dispatch` | 3 |
| `a` | `append` | `.` | `dispatch`, then input mode | 3, 11 |
| `i` | `insert` | `.` | `dispatch`, then input mode | 11 |
| `c` | `change` | `.,.` | `dispatch`, then input mode | 11 |
| `d` | `delete` | `.,.` | `run_delete` | 11 |
| `p` | `print` | `.,.` | `run_print` | 3, 4 |
| `n` | `number` | `.,.` | `run_print` | 4 |
| `l` | `list` | `.,.` | `run_print`, `render_list_line` | 14 |
| `j` | `join` | `.,.+1` | `run_join` | 11 |
| `m` *dest* | | `.,.` | `run_move` | 11 |
| `t` *dest* | | `.,.` | `run_transfer` | 11 |
| `s/re/new/[g]` | | `.,.` | `run_substitute` | 10 |
| `s` | | `.,.` | `run_substitute`, repeating the last | 10 |
| `g/re/cmd` | | `1,$` | `run_global` | 12 |
| `v/re/cmd` | | `1,$` | `run_global` | 12 |
| `w` [*file*] | | `1,$` | `run_write` | 5, 13 |
| `e` [*file*] | | | `run_edit` | 13 |
| `r` [*file*] | | `$` | `run_read` | 13 |
| `q` | `quit` | | the loop | 5 |
| `Q` | | | the loop | 14 |
| `H` | `help` | | `run_help` | 14 |

## A.5 Addresses and patterns

Addresses, from chapter 4: a line number, `.`, `$`, `+`*n* and `-`*n* from the
current line, `+` and `-` alone for one line, two addresses separated by a
comma, `,` for `1,$`, `;` for `.,$`, and a trailing comma for "to the end".
`m` and `t` accept 0 as a destination.

Patterns, from chapters 7 to 9: literal characters, `.`, `*` after a single
atom, `^` and `$` anchors, bracket expressions with ranges and negation and
POSIX's rules for `]` and `-`, backslash escapes, `\(` and `\)` groups,
backreferences `\1` to `\9`, and octal escapes `\000` to `\077`. Matching is
leftmost-longest, by recursive backtracking, on bytes.

Replacements, from chapter 10: `&` for the whole match, `\1` to `\9` for
groups, `\&` and `\\` for literal characters, and octal escapes.

## A.6 What it does not do

GNU ed commands that ved does not implement: `u`, `=`, `k`, `f`, `x`, `z`, `!`,
and `h`, and `wq`, which ved reads as `w` with the filename `q` (chapter 14). Address forms it does not accept: search addresses, `'x` marks,
and offsets attached to an address, such as `$-5`. Pattern features it does not
support: `\(...\)*`, `\{n,m\}`, character classes such as `[[:alpha:]]`, and
word boundaries. Version 0.3.0 added search addresses and made matching work on
characters instead of bytes; chapter 19 has the rest of the list, including
what this book found that behaves differently from ed.
