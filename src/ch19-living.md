# 19. Living With It

0.2.1 shipped on September 15. 0.3.0 shipped on November 11. Between them the
program gained a feature the plan had promised since slice three, its regex
engine was rewritten to work in characters, its test suite grew by half, and
its release machinery, three of its seven install channels, and its macOS
builds were removed. None of that was planned at 0.2.1. All of it is what
happens to a program after people can install it.

The code is `examples/ch19-living`, which is ved 0.3.0. Every earlier chapter
described 0.2.1, which is `examples/ved`. This chapter is about the difference,
and about the documents that decide what a program becomes once the plan that
built it has run out: the issue tracker, the backlog, and the list of what the
program will not do.

## 19.1 Where the findings went

Four of ved's issues were opened on the same day:

| Issue | Opened | Closed | Title |
|---|---|---|---|
| #7 | 2026-09-14 | 2026-09-27 | Support /pattern/ and ?pattern? search addresses |
| #8 | 2026-09-14 | 2026-09-15 | Panic on non-ASCII command character loses the buffer |
| #9 | 2026-09-14 | 2026-09-15 | Panic on a pattern with ten or more capturing groups |
| #10 | 2026-09-14 | 2026-09-27 | Byte-oriented regex corrupts multi-byte UTF-8 on substitute |

September 14 is the day chapters 1 through 9 of this book were written, and
each of those issues is a finding from one of them: chapter 4's search
addresses, chapter 5's panic, chapter 9's tenth group, and chapter 9's
corruption. The two panics were fixed the next day and shipped as 0.2.1, which
is why chapters 5 and 9 could say the bugs they found lasted until the chapter
was written. The other two took two weeks and shipped in 0.3.0.

Writing about a program in detail is a code review with a reader in mind, and
it finds what code review finds: the places where the code and the account of
it disagree. The useful habit is the one visible in the dates. A finding that
goes into the tracker the day it is made is a finding that gets fixed. A
finding left in prose is a paragraph.

## 19.2 The interim fix, then the real one

Chapter 9 found that `s/f./Z/` on `café` destroyed the `é` without a word, and
suggested two fixes: a cheap one that refuses any substitution whose match
boundaries fall inside a character, and an expensive one that teaches the
engine to work in characters. Exercise 9-2 asked for the cheap one. 0.3.0 did
both, a day apart.

The first commit, on September 26, is titled *Refuse a substitution that splits
a multi-byte UTF-8 character*. It checks the match boundaries against
`str::is_char_boundary` and returns an error instead of a damaged line. It
converts silent corruption into a refusal, which is the property chapter 9 said
an editor most needs, and with its tests it is under a hundred lines.

The second, the next day, is *Make the BRE engine character-aware, not
byte-oriented*, and its message is a small case study in why the cheap fix came
first. Making `.` and bracket expressions consume a whole character was the
obvious change, and it was not enough:

```text
That alone isn't sufficient: Regex::find's unanchored search tried
every byte offset, so a failed attempt starting at a real character
boundary could still fall through to trying the bytes inside that
character as further starting offsets, with Dot's byte fallback
happily accepting one — reopening the exact bug one level up, in the
search loop instead of a single atom.
```

The same gap was in `substitute_all`'s empty-match step, chapter 10's loop,
which advanced by one byte. With all three fixed, the commit says, *a
substitution can no longer produce a byte range that splits a character, so
the boundary check added in #10 as an interim safety net is now unreachable
and comes out*. The line chapter 10 showed in `run_substitute` became
`String::from_utf8(new_bytes).expect(...)`, an assertion that the invariant
holds rather than a repair for when it does not.

The order is the lesson. The interim fix stopped the data loss with a change
small enough to be obviously correct, and it made the real fix safe to attempt,
because a mistake in the rewrite would now produce an error rather than a
corrupted file. The real fix then deleted the interim one, which is how an
interim fix is supposed to end. The `$` corruption from chapter 10, a
substitution that appended a stray character, disappeared in the same commit,
because the loop now pushes the character at the match instead of the byte at
the old position.

## 19.3 Sixty tests

0.3.0 has 185 tests, against 125. Sorted by what they exercise:

| Area | Tests added |
|---|---|
| `parse_args` | 8 |
| Commands through `dispatch`: `w`, `r`, `e`, `d`, `g`, `v`, `H`, `Q`, and the setup for `a`, `i`, `c` | 29 |
| Search addresses | 9 |
| Character-aware matching and substitution | 14 |

Chapter 15 found that `parse_args` and half the commands had no tests at all,
and 0.3.0 closed most of that gap. The character tests are the best in the
suite: names like `four_dots_span_cafe_but_five_do_not` and
`class_member_match_is_whole_sequence_not_shared_lead_byte` state a fact about
the specification, and each one tests behavior that 0.2.1 got wrong.

Most of the others assert what the program already did. That is chapter 15's
pattern, and two of the names show it:

```rust,ignore
    fn edit_of_a_missing_file_wipes_the_buffer_as_a_new_file() {
    fn write_refuses_to_overwrite_a_non_utf8_source() {
```

The first makes `e` discarding the buffer a tested property, in a release where
`e` still discards a modified buffer with no warning, which chapter 13 found.
The second holds in place the guard that chapter 13 showed blocks only false
positives. Both are accurate descriptions of the code, and both will have to be
changed by whoever fixes the behavior they describe. A test written by running
the code protects the code's behavior, including the behavior that is a bug,
and that is a real cost of a suite that grows this way.

Run against GNU ed with chapter 15's harness and the same three thousand
single-command scripts, 0.3.0 leaves a different file in 439 cases, against
516 for 0.2.1. The 77 that went away are the `s/$/!/g` row, the corruption the
character rewrite fixed. Every other row in chapter 15's table is unchanged.

## 19.4 The list the book adds

Chapters 10 through 18 were written after 0.3.0 shipped, so their findings
have not been through the tracker. They are in the position chapter 9's were
in on September 14: written down, not yet scheduled. Checked against 0.3.0, the
ones that change what happens to a user's text:

| Finding | Chapter | In 0.3.0 |
|---|---|---|
| An escaped delimiter becomes a metacharacter: `s.a\..X.` | 10 | still there |
| `^` matches again on each `g` repetition | 10 | still there |
| An empty match next to a real one is replaced | 10 | still there |
| `s/a/b` without a closing delimiter is an error | 10 | still there |
| A count or unknown suffix after `s` is ignored | 10 | still there |
| `s/$/!/g` adds a stray character | 10 | fixed |
| `0a` and `5a` put text in the wrong place | 11 | still there |
| `g` marks numbers, not lines, so `g/a/j` and `g/a/-1d` hit the wrong lines | 12 | still there |
| CRLF line endings are rewritten as LF | 13 | still there |
| `e` discards a modified buffer, and a failed `e` loses it | 13 | still there |
| A UTF-8 file containing `+ACI-` cannot be saved in place | 13 | still there |
| `write`, `edit`, and `read` act on files named `rite`, `dit`, `ead` | 14 | still there |
| The tutorial teaches `write notes.txt` | 16 | still there |

And the ones that change what a script sees: no `-s` flag, exit status 0 after
an error, confirmations on standard output (chapter 14), and no declared
minimum Rust version (chapter 17). Fixes for most of the rows in the first
table, the escaped delimiter, the `g` loop, the append address, the marks,
CRLF, and `e`, were built and tested in a scratch copy as each chapter was
written. They are in those chapters, not in the program.

The list is long, and its length is the measure of something specific. ved's
first release was tested by its author against the cases its author thought of.
Its first close reading by someone comparing it line by line against ed found
the rest. That is the ordinary history of a program, and the part to copy is
what happens next: the list goes into the tracker, one issue per row, and the
rows that lose text go first.

## 19.5 The backlog

The other document that decides what a program becomes is the list of things
it does not do. ved's is `BACKLOG.md`, kept outside the repository since the
commit chapter 16 described, and it is organized by why each item is not done
rather than by when it will be:

```text
## Queued
## High-impact, unscheduled
## Smaller / conditional
## Gated on the pure-stdlib posture
## Rejected by project identity
```

That structure is the best idea in the file. "Not done" covers several
different states, and a list that does not distinguish them invites the same
argument about each item every time it is read. *Queued* means someone has
decided to do it. *Unscheduled* means it is wanted and nothing has made it
urgent. *Gated* means it would happen if a constraint changed. *Rejected* means
it will not happen while the program is what it is, and saying so in writing is
what stops it being re-proposed.

Chapter 4 said search addresses sat at the top of this file, and they did,
with a note about what they would cost:

```text
### Search addresses — `/pattern/`, `?pattern?`

The highest-remaining-impact item. The address parser handles line numbers, `.`,
`$`, and offsets, but not regex search. Adding it connects the existing BRE
engine (`src/bre.rs`) to the address resolver. If the pure-stdlib gate is ever
bent, this is the most likely reason.
```

Search addresses shipped in 0.3.0 with no dependency at all. The engine was
already in the program, and connecting it to the address resolver was about
eighty lines in `address.rs`. The entry's prediction about the gate was wrong,
and it had been wrong since the engine existed. The item was waiting on nothing
except being picked up, and the note implied it was waiting on a decision. The
backlog still lists it, under a status line dated July.

That is two lessons about a file whose purpose is to be read later. The reason
an item is not done is the part that decides whether anyone does it, so the
reason has to be checked as carefully as the item. And a backlog is a list of
what is not done, so a done item is deleted from it, in the commit that does
it, rather than left for a status line to qualify.

Chapter 8 said the backlog recorded its non-failing compiler, the regex
compiler that accepts `[abc` without an error, as a debt. It does not. Neither
does the file mention chapter 8's `Star(Atom)` limitation except as `\(...\)*`
under *smaller*, or chapter 9's step limit for pathological patterns. Those
are the entries this chapter would add to it.

## 19.6 Undo

Chapter 11 said this chapter would say why ved has no `u`. The backlog lists it
under *high-impact, unscheduled*, and the author's separate list of
limitations files it with Ctrl-C handling as *waiting on the same deps gate*.

The second of those is wrong. Chapter 11 showed that every change to the buffer
goes through three methods, and that a single-level undo is a snapshot of
`lines` and `current` taken before each command that changes them, which is a
dozen lines of standard library code. There is no gate. The first is right, and
it is the same answer chapters 6 and 9 gave for their unfinished work: nothing
has forced it.

The two documents disagreeing about the same item is a small instance of
chapter 16's rule. A fact about the program that is stated in two places will
eventually be stated two ways, and the one filed behind a gate is the one less
likely to be picked up.

## 19.7 What the constraint cost

The preface said chapter 1 explains why ved has no dependencies and this
chapter explains the two features that has cost. They are both in the backlog,
and they are the two items that a dependency would make possible and the
standard library does not.

**Ctrl-C.** Traditional ed catches the interrupt signal and uses it to abandon
what is being typed: Ctrl-C in input mode discards the partial text and returns
to command mode. ved has no signal handler, so Ctrl-C kills the process and
takes the buffer with it, which is the opposite of the protection the
double-`q` gives. Rust's standard library has no portable way to install a
signal handler. The options are a crate, `libc` through FFI, or hand-written
`extern "C"` declarations per platform. The backlog files it under *gated*: it
would go through the day the constraint did.

**An editable input block.** A proposal to let the user move a cursor within
the lines being typed in input mode, editing them before the terminating `.`,
needed raw terminal mode, an escape-sequence parser, and window-size handling,
none of which the standard library has. This one is filed under *rejected*, and
the reason is not the dependency alone: it would change what ved is, from a
line editor to something on the way to a screen editor. The idea was built
instead as a separate program, `nved`, written in Go, where it is the point.

Those are the two features. The costs paid along the way were smaller and are
scattered through the book: the encoding sniffer duplicated from a crate the
author already maintained (chapter 13), the argument parser that does not
understand `--` (chapter 13), the benchmark done with `Instant` (chapter 15),
and a regex engine that took three chapters and backtracks exponentially, which
the `regex` crate would not, at the price of having no backreferences.

What it bought is in the same chapters. A release build in half a second, a
binary with no supply chain beyond the compiler, a `Cargo.lock` with one
package in it, and, for anyone packaging the program for a distribution, one
source package with nothing else to package first. And it bought the book:
every line of ved can be read, including the parts that would otherwise have
been someone else's crate, which is why chapters 7 through 9 could be written
at all.

The general version is the one chapter 1 gave. A constraint is worth having
when it answers questions in advance, and ved's answered these four cleanly:
inline the sniffer, park Ctrl-C, build `nved`, write the engine. It cost two
features that a user would notice, and it was chosen with that cost visible.

## 19.8 The procedure leaves the repository

At 0.2.1, ved's repository held its release procedure: `RELEASING.md`, the
`cargo-dist` workflow, the crates.io and Debian workflows, a parameter file for
the tools that ran them. At 0.3.0 it holds a CI workflow that runs by hand and
a small file naming what to build, which is not committed. The rest moved into
release tools shared by the author's projects and run from the author's own
machines.

The commit that deleted `RELEASING.md` gives the reason in two sentences:

```text
A document beside the tool is what a session on the Mac or the Windows
machine reads instead of running it, and what drifts from it.
```

That is chapter 16's argument applied to procedure. A document describing how
to release is a second copy of the procedure, and the copy is what someone
reads at the moment it matters, and it drifts. Make the procedure a program,
and the program is the only copy.

The trade is real in both directions. One procedure for every project means a
correction reaches all of them, and a release does not depend on remembering
what this repository's notes said. It also means the repository no longer says
how it is released, so a contributor who is not the author cannot cut one, and
a reader of the repository cannot learn from it what chapter 17 described. For
a project with one maintainer and a dozen siblings, the first side wins. For a
project that hopes for a second maintainer, it is a cost to count.

Continuous integration changed the same way. The workflow that chapter 17
described running on every push now runs when it is started by hand, before a
release or after a change worth checking. A check that runs on every push and
is red for reasons nobody fixes stops being read, and its author decided that a
check run deliberately and read every time was more useful than one run
constantly and ignored. The cost is that a broken commit is no longer flagged
the moment it is pushed, and the release is the backstop.

## 19.9 The channels, counted

Chapter 18 ended by saying the cost of seven channels had not been counted at
0.2.1. By 0.3.0 it had:

| Channel or artifact | Removed | Commit subject |
|---|---|---|
| curl installer | 2026-09-15 | *Retire the curl installer: apt, Homebrew, winget and cargo are the paths* |
| Defender scan before WinGet | 2026-09-15 | *Delete the Defender scan caller: the scan never predicted winget's validation* |
| Homebrew formula | 2026-11-11 | *build: stop publishing a Homebrew formula* |
| macOS builds, `cargo-dist` | 2026-11-11 | *build: release without cargo-dist or macOS builds* |

The 0.3.0 README lists four ways to install: Debian and Ubuntu, Windows, Cargo,
and building from source. The release builds for x86-64 and ARM Linux and for
x86-64 Windows, and the release page carries two `.deb` files and a Windows
zip.

Each removal has a one-line reason, and the reasons share a shape: the channel
cost something at every release and returned less than it cost. The curl
installer duplicated what the package managers did. The Defender scan did not
predict what it was meant to predict. The Homebrew formula pointed at macOS
archives, and the macOS archives, the PowerShell installer, and the Linux
tarballs went with `cargo-dist`.

Removing a channel has a cost the commits do not state, and it falls on the
people who used it. The Homebrew tap now holds only a README. A user who
installed ved 0.2.1 with `brew install` keeps 0.2.1, and `brew upgrade` has no
newer version to offer them and no way to say why. A macOS user who installed
from the release archives finds no macOS archive for 0.3.0. Neither was told,
because the channels have no way to reach their users except by delivering an
update, and the update is what stopped.

That is the reason chapter 18 said to know how you would measure a channel
before adding it, and the remedy for a project that did not is modest: when a
channel ends, say so in the place its users will look. A last release on that
channel whose notes say where to go next, or a line in the README's install
section saying the channel was retired and naming its replacement. `cargo
install ved` works on macOS, and a sentence saying so would have reached the
people the removal affected.

## 19.10 What living with it bought

A program that is better than it was when it shipped, in the ways a user
notices: search addresses, a regex engine that respects characters, and two
fewer crashes. A test suite half again as large. A release procedure that is
the same for every project its author maintains. And a set of channels that
matches what the author can support.

The book ends where a program's life continues. ved 0.3.0 is in use, and it has
a list of findings that this book produced and its tracker has not yet seen,
a backlog with a structure to copy and entries to recheck, and a constraint
that has cost two features and paid for them. The work of a
program after its first release is that loop: someone reads it closely, writes
down what is wrong, and the list goes into the place where lists get done.

---

**Exercise 19-1.** Turn the table in section 19.4 into issues, one per row,
each with the session that reproduces it and the GNU ed output it should
match. Then order them, and write down the rule you used.

**Exercise 19-2.** Rewrite ved's backlog for 0.3.0: delete what shipped, add
what chapters 8 through 18 found, and check the stated reason on every item
that remains. Count how many reasons changed.

**Exercise 19-3.** Implement Ctrl-C in input mode for Linux only, with
`extern "C"` declarations for `signal` and an `AtomicBool`, and no crate.
Measure what it cost in lines and in `unsafe` blocks, and decide whether the
constraint should bend for it.

**Exercise 19-4.** Write the README paragraph that tells a Homebrew user what
happened to ved's formula and how to keep getting updates. Then decide where
else it needs to appear for that user to see it.

**Exercise 19-5.** Pick a program you shipped more than six months ago. List
what changed since, sorted the way section 19.9 sorts removals: what it cost
each release, what it returned, and who was told when it stopped.
