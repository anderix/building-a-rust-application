# 15. Testing

ved 0.2.1 has 125 tests, they all pass, and they run in less time than `cargo
test` takes to print its header. Chapters 10 through 14 found, between them,
more than twenty ways in which the program does something other than what ed
does, at least five of which lose or misplace text. None of the 125 tests
fails on any of them.

That is not an argument that the tests are bad. Most of them are the right
tests, of the right shape, and they are why the program could be changed
without fear from its first release to 0.2.1. It is an argument that a passing
suite is a statement about the questions the suite asks, and this chapter is
about which questions ved's suite asks, which it does not, and three cheap ways
to ask the rest.

## 15.1 What there is

| Module | Lines | Tests |
|---|---|---|
| `bre.rs` | 1,064 | 90 |
| `main.rs` | 1,287 | 32 |
| `enc.rs` | 131 | 3 |
| `address.rs` | 233 | 0 |
| `buffer.rs` | 156 | 0 |

The regex engine is the most tested part of the program by a wide margin, and
that is the right allocation. It is the hardest code, it is the code with an
external specification precise enough to write expected values from, and its
tests are cheap, because a pattern and a string go in and a match comes out:

```rust,ignore
    #[test]
    fn backref_to_uncaptured_fails() {
        // \1 with no group should fail to match
        assert!(!has_match(b"^\\1$", b"anything"));
    }
```

`address.rs` and `buffer.rs` have no tests of their own. Both are exercised
through `main.rs`, since every command test parses an address and changes a
buffer, so they are covered in the sense that their code runs. They are not
covered in the sense that anyone has stated what they should do in a case the
commands do not happen to reach. `resolve_with` has four conditions for an
invalid address, and chapter 11's `0a` was the fifth case, one nobody had
written down.

## 15.2 What `dispatch` tests are worth

Chapter 2 made `dispatch` return a value instead of printing, and said this
chapter would show what that was worth. Here is what it was worth:

```rust,ignore
    #[test]
    fn move_into_self_errors() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"]);
        let act = dispatch("2,4m3", &mut buf);
        assert!(matches!(act, Action::Error(_)));
        assert_eq!(buf.line(3), Some("c"));
    }
```

Every command test in the program has that shape. Build a buffer from a list
of strings, call `dispatch` with what a user would type, and check two things:
the `Action` that came back, and the buffer afterwards. There is no process, no
terminal, and no captured output, and the whole suite finishes in what `cargo`
reports as 0.00 seconds.

The assertions are as important as the setup, and they share a habit. They
check the *kind* of `Action`, with `matches!(act, Action::Error(_))`, and they
check the buffer's contents line by line. They almost never check the text of
a message. That is why chapter 14 could propose rewriting `invalid address`
into four different messages without breaking a test: the tests say that `2,4m3`
must fail and must leave line 3 alone, which is the requirement, and they leave
the wording free, which is the part that should be free to improve.

A test that asserts on a message string is a test of the message. There are
cases where that is the point, and ved has one, which section 15.4 is about.
In most cases it is an accident that makes the suite fight every change to the
program's voice.

## 15.3 The layer nobody tested

`dispatch` tests stop at `dispatch`. Everything that happens after an `Action`
is returned is in `run_repl`, which turns the value into output and decides the
exit status, and in `main` and `parse_args`, which run before it. At 0.2.1, none
of that has a test.

Two bugs from earlier chapters live in that layer. Chapter 11's single-line
`j` returns `Action::Print(String::new())`, which a `dispatch` test sees as an
empty string and the loop prints as a blank line. Chapter 14's exit status is
`0` after a failed command, which no `dispatch` test can see because `dispatch`
does not return an exit status. Both are visible from outside the process, and
Cargo has direct support for testing from there. A file in `tests/` is an
integration test, compiled as a separate crate, and Cargo gives it the path to
the program's binary:

```rust,ignore
use std::io::Write;
use std::process::{Command, Stdio};

fn session(input: &str) -> (String, i32) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ved"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("ved runs");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.code().unwrap())
}

#[test]
fn single_line_join_prints_nothing() {
    let (out, _) = session("a\none\ntwo\n.\n2,2j\nQ\n");
    assert_eq!(out, "");
}

#[test]
fn failed_command_sets_exit_status() {
    let (_, status) = session("zz\nQ\n");
    assert_eq!(status, 1);
}
```

Added to a copy of `examples/ved`, both fail, with the bugs they were written
for:

```text
assertion `left == right` failed
  left: "\n"
 right: ""
assertion `left == right` failed
  left: 0
 right: 1
```

`session` is the whole harness, and it is the one piece of test infrastructure
this program lacked. It drives ved the way a user or a script does, through
standard input, and it sees what they see. The tests are slower than
`dispatch` tests, since each one starts a process, and they should be few: one
for each thing the loop is responsible for, which is a short list. Prompts,
input mode, the double `q`, the exit status, what goes to standard output and
what goes to standard error. The command language stays in `dispatch` tests,
where it is cheap.

The general rule is that a test suite should have at least one test at every
boundary the program has with the outside world, because the bugs that live at
a boundary are invisible from either side of it. ved's suite had tests on one
side of `dispatch` and none on the other.

## 15.4 Tests that agree with the code

Chapter 14 found a test that holds a deviation from POSIX in place:

```rust,ignore
    #[test]
    fn list_tab_becomes_octal() {
        assert_eq!(render_list_line("a\tb"), "a\\011b$");
    }
```

POSIX says `l` writes a tab as `\t`. GNU ed does. ved writes `\011`, and this
test makes sure it keeps doing so. Chapter 11 found another, which asserts that
a single-line join returns an empty `Print` and is named as though it asserted
silence.

Both tests were written at the same time as the code they test, by the same
author, and the expected value in each came from the same place as the code:
the author's idea of what the function should do. When that idea was wrong,
the test was wrong in the same way, and the two agreed. A test like that is not
useless, since it still catches the code changing by accident, but it cannot
catch the code being wrong, because it has no independent source of truth.

The repair is about where the expected value comes from.
`list_tab_becomes_octal` should have been written by reading the POSIX
description of `l`, or by running `printf 'a\tb\n' | ed -s` with an `l`
command, and copying what came out. Then the test disagrees with the code on
the day it is written, and the disagreement is the finding. A test that can
never disagree with the code it tests is documentation.

Chapter 14 asked how to write tests that can disagree. That is the answer: the
expected value comes from somewhere the code did not.

## 15.5 A second program as the specification

Chapter 7 described a bug that survived because it was not in any layer the
tests covered. `match_star` tried the shortest repetition first, the engine's
tests all passed, and the wrong answer appeared only in the output of `s`, two
slices later, as a disagreement with POSIX's leftmost-longest rule. Chapter 7
said this chapter had a name for that, and a way to test for it.

The name is differential testing. When a program has a reference
implementation, the reference is the oracle: run the same input through both
and compare. Any difference is either a bug in the program, a bug in the
reference, or a deliberate deviation, and all three are worth knowing about.
Chapter 1 chose ed partly because a specification existed. The most useful
form of that specification is not POSIX's text. It is `/usr/bin/ed`, which is
installed on the machine and answers any question in a millisecond.

Most of the bugs in chapters 10 through 14 were found that way, by hand: type a
command into both editors and compare. Automating it takes about forty lines.
The harness used here generates random scripts from a small grammar of
addresses and commands, runs each through ved and through GNU ed on the same
six-line file, ends each script with `w out.txt` and `Q`, and compares the
files the two editors wrote:

```text
CMDS = ["p", "n", "d", "j", "m{a}", "t{a}", "s/a/X/", "s/a/X/g", "s/^a/X/g",
        "s/$/!/g", "s/[ab]*/-/g", "s/\\(a\\)\\(.\\)/\\2\\1/", "s.a\\..X.", "s/a/X",
        "g/a/d", "g/a/j", "g/a/m0", "g/b/-1d", "v/a/d", "g/a/s/a/A/", "g/a/.,+1d",
        "a\nNEW\n.", "i\nNEW\n.", "c\nNEW\n.", "s"]
```

Comparing the written file rather than the output is the decision that makes
the harness usable. ved's messages differ from ed's on purpose, so comparing
everything either editor printed would report a difference on every line. The
file is the part that has to agree, and it is the part a user loses if it does
not.

Three thousand single-command scripts against ved 0.2.1, with a fixed random
seed, took twelve seconds. 516 of them left a different file:

| Command | Different | Run |
|---|---|---|
| `s.a\..X.` (escaped delimiter, ch. 10) | 87 | 132 |
| `s/a/X` (optional delimiter, ch. 10) | 86 | 116 |
| `s/$/!/g` (empty match, ch. 10) | 77 | 130 |
| `s/[ab]*/-/g` (empty match, ch. 10) | 69 | 103 |
| `g/a/.,+1d` (stale marks, ch. 12) | 54 | 132 |
| `g/a/j` (stale marks, ch. 12) | 46 | 121 |
| `g/b/-1d` (stale marks, ch. 12) | 42 | 107 |
| `a` with an address (ch. 11) | 37 | 123 |
| `i` with an address (ch. 11) | 18 | 104 |

Every row is a bug this book had already found by hand, which is a check on
the harness as much as on the program. Then the same three thousand scripts
against a copy of `examples/ved` with the fixes from chapters 10 through 13
applied: 155 differences, all from two causes. 86 are `s/a/X`, the optional
closing delimiter, which no chapter fixed. 69 are `s/[ab]*/-/g`, where GNU ed
refuses with `? Infinite substitution loop` and ved, after chapter 10's fix,
gives the answer `sed` gives. That row is a disagreement with the reference
that the program is right to keep, and the harness has no way to know that.
A person has to read it.

The harness also found a bug in this book. Chapter 11's first version of
`resolve_target` checked only the second address of a range, so `9,3a` on a
six-line buffer appended after line 3 when ed reports an error, and tightening
it to reject a backwards range made `3,1a` an error when ed accepts it. Both
were caught on the first run after each change, and the version in chapter 11
is the one that passes. Neither case was among the eleven that chapter 11
checked by hand, which is the argument for generating cases rather than
choosing them: a person picks the cases they have thought of.

With scripts of one to three commands, where earlier commands move the current
line for later ones, the count goes from 607 of 2,000 on 0.2.1 to 195 after the
fixes. Most of the remainder is the same two causes. The rest are smaller
disagreements about where the current line ends up after a command that does
nothing, such as a single-line `j`, which leaves ed's current line where it was
and moves ved's.

A harness like this is not a unit test, since it needs GNU ed installed and
takes seconds rather than microseconds, and it should not run on every build.
It is a tool to run when the command language changes, and its output is a
list of things to read rather than a pass or a fail.

## 15.6 Input nobody typed on purpose

Chapter 5 found a panic that lived from slice four to the writing of the
chapter: a non-ASCII character typed as a command sliced a `str` off a
character boundary. Chapter 5 said the tests missed it because they drive the
program with the command language, which is ASCII, and that this chapter would
come back to what a suite built from valid input cannot tell you.

It cannot tell you what invalid input does. Every test in the suite was written
by someone thinking about a command, and so every test is a command. The panic
needed a string that is not a command at all, and nobody writes a test for a
string they would never type.

A fuzzer writes them. The full version is a crate, and the minimal version is a
test with a random number generator and `catch_unwind`:

```rust,ignore
    #[test]
    fn fuzz_dispatch_never_panics() {
        let alphabet: Vec<char> =
            "0123456789$.,;+-/\\[]()*^&gvsdpnljmtaciqQH \té×".chars().collect();
        let lines = ["a", "é", "", "aéb", "ab", "\\"];
        let mut seed: u64 = 0x9E3779B97F4A7C15;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        std::panic::set_hook(Box::new(|_| {}));
        for _ in 0..1_000_000 {
            // a random buffer of 0-4 lines from `lines`,
            // then 1-4 random commands of 1-10 characters each
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for c in &cmds {
                    dispatch(c, &mut buf);
                }
            }));
            // record each distinct panic message and an example
        }
    }
```

The generator is xorshift, three lines, and the alphabet is the command
language's characters plus three that are not ASCII. `w`, `r`, and `e` are left
out so that the fuzzer cannot write files. `set_hook` silences the default
panic message so that a million runs do not print a million backtraces, and
`catch_unwind` turns each panic back into a value the loop can record.

Run against `examples/ch10-substitute`, which still has chapter 5's bug, a
million sequences took 0.99 seconds and found it twice, once for each non-ASCII
command character that reached the dispatcher's slice:

```text
PANIC start byte index 1 is not a char boundary; it is inside 'é' (bytes 0..
   e.g. ["(/", "1é3 \\s9lé "] on ["", "\\", "ab", "ab"]
PANIC start byte index 1 is not a char boundary; it is inside '×' (bytes 0..
   e.g. ["×;×Hp9"] on ["a", "aéb", "\\"]
2 distinct panics in 1000000 runs
```

Run against 0.2.1, a million sequences found nothing, and the same against
0.3.0. That is a real result, and it should be read for what it measures:
within this alphabet, this length, and this buffer, `dispatch` does not panic.
It says nothing about wrong answers, and the contrast with the previous section
is the useful part. The differential harness found 516 disagreements in twelve
seconds and the fuzzer found no crashes in a second, on the same program. ved's
bugs are almost all of the first kind. A program that answers plausibly and
wrongly will pass a fuzzer indefinitely.

The two tools ask different questions. The fuzzer asks whether the program
survives any input. The differential harness asks whether it gives the right
answer to valid input. A suite built only from hand-written cases asks whether
it gives the right answer to the input its author thought of. All three
questions are cheap to ask, and ved, at release, asked the third.

## 15.7 Measuring the representation

Chapter 3 chose `Vec<String>` for the buffer and said that this chapter was
where the measuring happens. Chapter 3's exercise 3-5 asked for the number,
and here it is. `Buffer` from `examples/ved`, compiled with `--release`, filled
with lines of fifty characters, on an 11th-generation Intel Core i7 laptop:

| Buffer | 1,000 inserts at line 1 | 1,000 inserts at the end | 1,000 deletes of line 1 |
|---|---|---|---|
| 100,000 lines | 51 ms | 0.04 ms | 48 ms |
| 1,000,000 lines | 1.37 s | 0.03 ms | 1.42 s |

An insert at the top of a hundred-thousand-line buffer costs about 51
microseconds, which is the memory move chapter 3 predicted and is invisible to
a person typing. At a million lines it is 1.4 milliseconds per line, so pasting
a thousand lines at the top of a million-line file takes over a second, and
that is the first number in this book where the representation shows.

The pathological case is a command that does an insert per line across the
whole buffer. `g/^/m0` reverses a file by moving every line to the top in turn,
which is quadratic with `Vec<String>`. On a hundred thousand lines it took 6.6
seconds in ved and 18.1 seconds in GNU ed.

So chapter 3's bet holds, with a measured boundary on it. For files people edit
with ed, the vector is faster than anything that would replace it, because it
is simple and cache-friendly. It stops being invisible around a million lines,
and only for operations near the top of the buffer. A program that expected to
edit files that size would want a gap buffer or a rope, and it would know so
from a benchmark rather than from a feeling.

There is no benchmarking crate in this measurement, because there is no crate
in this program. `std::time::Instant` around a loop, in a binary built with
`--release`, run twice to check the numbers repeat. That is enough to answer
the question chapter 3 asked, and the question was the hard part.

## 15.8 What not to test

The preface promised this chapter would say what not to test as well, and the
earlier sections imply most of it.

Do not assert on the wording of messages, except in the tests whose subject is
a message. Do not test the standard library: a test that `Vec::insert` puts a
line where it was asked is testing Rust. Do not write a test whose expected
value you got by running the code, because it will agree with the code forever,
including when the code is wrong. And do not write a test that cannot fail: a
test that loads a file it just wrote, with no assertion on the contents, adds
to the passing count and to nothing else.

The other side of that list is where to spend the effort. One test per
boundary with the outside world. Expected values from the specification or the
reference. Random input, both valid and not, generated by a program. And for
every bug found by any means, a test that fails before the fix and passes
after, so that the passing count grows by the number of things the program has
learned.

## 15.9 What the suite bought

A program that could be changed from release to release without breaking what
it already did, which is what a regression suite is for. 125 tests in
hundredths of a second, almost all of them through the `dispatch` boundary that
chapter 2 made for this purpose.

It did not buy correctness against ed, because nothing in it consulted ed. The
three additions in this chapter cost a script of about forty lines, a test of
about fifty, and an integration file of twenty-five, and between them they
reproduce most of what the five chapters before this one found about what the
program does to text. 0.3.0 has 185 tests. The next
chapter but three is about what those sixty added tests do and do not check.

---

**Exercise 15-1.** Add tests for `address.rs` directly: one for each condition
in `resolve_with` that produces `invalid address`, and one for each form in
chapter 4's grammar table. Write the expected values from the table, not from
running the code.

**Exercise 15-2.** Add the `session` harness to `examples/ved` and write one
test for each responsibility of the loop listed in section 15.3. Count how many
fail.

**Exercise 15-3.** Write the differential harness from section 15.5 in Rust, as
an ignored test that runs only when GNU ed is installed. Add `r`, `e`, and `w`
to the grammar with files in a temporary directory, and see what it finds in
chapter 13's territory.

**Exercise 15-4.** Extend the fuzzer to drive the binary through `session`
rather than `dispatch`, so that input mode and the loop are reachable. Decide
what to do about commands that never return, such as an input mode that never
sees its `.`.

**Exercise 15-5.** Fix `list_tab_becomes_octal` by writing what POSIX requires
and letting it fail. Then go through every test in `bre.rs` and mark each
expected value with where it came from: POSIX, GNU ed, or the code. Count each.
