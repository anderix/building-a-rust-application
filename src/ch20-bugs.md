# 20. Bugs

Chapter 19 ended with a list of findings and the claim that they belong in a
tracker. This chapter is about what was on the list: what the bugs were, how
long they had been there, what caused them, how they were found, and how to
tell a bug from a difference that is meant. Chapter 21 fixes them and chapter
22 ships the fixes.

The program in this chapter is still 0.3.0, `examples/ch19-living`. Every
session below was run against it.

## 20.1 The list

On November 21, the day the first draft of this book was finished, each
finding from chapters 10 through 19 went into ved's tracker as its own issue,
with a session that reproduces it and the reference output it should match.
Five more were filed the same day, while the fixes were being made: two from
reading the POSIX specification of ed, two found by the differential harness,
and one for what the harness found that nobody had explained yet. One more
was found while this chapter was being written, which section 20.6 is about.

| Issue | Finding | Found in |
|---|---|---|
| [#14](https://github.com/excelano/ved/issues/14) | `wq` writes a file named `q` and does not quit | ch. 14 |
| [#15](https://github.com/excelano/ved/issues/15) | `write`, `edit`, and `read` act on files named `rite`, `dit`, `ead` | ch. 14 |
| [#16](https://github.com/excelano/ved/issues/16) | The tutorial teaches `write notes.txt` | ch. 16 |
| [#17](https://github.com/excelano/ved/issues/17) | `e` discards a modified buffer, and a failed `e` empties it | ch. 13 |
| [#18](https://github.com/excelano/ved/issues/18) | CRLF line endings are rewritten as LF | ch. 13 |
| [#19](https://github.com/excelano/ved/issues/19) | `g` marks numbers, not lines | ch. 12 |
| [#20](https://github.com/excelano/ved/issues/20) | `0a` and `5a` append after the current line | ch. 11 |
| [#21](https://github.com/excelano/ved/issues/21) | An escaped delimiter becomes a metacharacter | ch. 10 |
| [#22](https://github.com/excelano/ved/issues/22) | `^` matches again under `g`; an empty match after a real one is replaced | ch. 10 |
| [#23](https://github.com/excelano/ved/issues/23) | `s/a/b` without its closing delimiter is an error | ch. 10 |
| [#24](https://github.com/excelano/ved/issues/24) | A count or unknown suffix after `s` is ignored | ch. 10 |
| [#25](https://github.com/excelano/ved/issues/25) | A UTF-8 file containing `+ACI-` cannot be saved in place | ch. 13 |
| [#26](https://github.com/excelano/ved/issues/26) | Exit status 0 after an error | ch. 14 |
| [#27](https://github.com/excelano/ved/issues/27) | No `-s`, and confirmations mixed into a script's output | ch. 14 |
| [#28](https://github.com/excelano/ved/issues/28) | No declared minimum Rust version | ch. 17 |
| [#29](https://github.com/excelano/ved/issues/29) | The help omits `e`, `r`, and `Q`, and misstates three defaults | ch. 14 |
| [#30](https://github.com/excelano/ved/issues/30) | The WinGet manifest still says *drop-in* | ch. 18 |
| [#31](https://github.com/excelano/ved/issues/31) | Nothing tells a macOS user what happened to their channel | ch. 19 |
| [#32](https://github.com/excelano/ved/issues/32) | A replacement of `%` alone is not the previous replacement | POSIX |
| [#33](https://github.com/excelano/ved/issues/33) | End of input on a modified buffer quits without a warning | POSIX |
| [#34](https://github.com/excelano/ved/issues/34) | The current line after `g/re/d` is wrong | harness |
| [#35](https://github.com/excelano/ved/issues/35) | `j` with one address moves the current line | harness |
| [#36](https://github.com/excelano/ved/issues/36) | The differences the harness still finds | harness |
| [#38](https://github.com/excelano/ved/issues/38) | `l` writes a tab as `\011` instead of `\t` | ch. 14, again in ch. 20 |

Every issue in ved's tracker was opened by its author. No user has reported a
bug. That does not mean no user hit one. It
means that a user who saves a file with `wq` and finds it unchanged, or opens
a Windows file and saves it back with different line endings, has no reason to
think the editor is wrong, and less reason to go and say so.

## 20.2 How long they shipped

Each issue has a reproduction, so each can be run against every release. The
script that does it is `tools/bugs-by-release.sh`: for each issue it runs the
session against one binary and prints `BUG` if the binary still behaves the
way the issue describes. Every tagged release from 0.1.0 to 0.3.0 was built
from its tag and checked:

| Issue | 0.1.0 | 0.1.5 | 0.2.0 | 0.2.1 | 0.3.0 |
|---|---|---|---|---|---|
| #14 `wq` | BUG | BUG | BUG | BUG | BUG |
| #15 `write` | BUG | BUG | BUG | BUG | BUG |
| #17 `e` | BUG | BUG | BUG | BUG | BUG |
| #18 CRLF | BUG | BUG | BUG | BUG | BUG |
| #19 `g` marks | BUG | BUG | BUG | BUG | BUG |
| #20 `0a` | BUG | BUG | BUG | BUG | BUG |
| #21 escaped delimiter | BUG | BUG | BUG | BUG | BUG |
| #22 `^` under `g` | BUG | BUG | BUG | BUG | BUG |
| #23 closing delimiter | BUG | BUG | BUG | BUG | BUG |
| #24 count suffix | BUG | BUG | BUG | BUG | BUG |
| #25 UTF-7 refusal | ok | ok | BUG | BUG | BUG |
| #26 exit status | BUG | BUG | BUG | BUG | BUG |
| #32 `%` | BUG | BUG | BUG | BUG | BUG |
| #33 end of input | BUG | BUG | BUG | BUG | BUG |
| #34 after `g/re/d` | BUG | BUG | BUG | BUG | BUG |

Fourteen of the fifteen were in the first release, on May 14, and survived
every release after it: nine releases over six months, including 0.2.1, which
fixed two panics, and 0.3.0, which rewrote the regex engine. The fifteenth, the
UTF-7 refusal, was added in 0.2.0 by the feature that was meant to protect
users from a different problem. `j` did not exist before 0.1.5, so #35 starts
there.

The table says something about where bugs come from that is easy to
misread. None of these bugs was introduced by a careless change to working
code. Each was there from the day its feature was written, and each survived
because nothing after that day looked at the feature again. The slice plan
from chapter 1 built a feature, tested the cases its author had in mind, and
moved on, which is the right way to build a program and leaves exactly these
behind. A program's bugs are mostly the ones it was born with.

## 20.3 What caused them

Read as a list of symptoms, the issues look unrelated: a file named `q`, a
line in the wrong place, a stray `!`. Read as a list of causes, they fall into
a handful of patterns, each of which has a shape you can look for in code of
your own.

**A parser that interprets what it should pass on.** The scanner that finds
the end of each field in `s/re/new/` also turned `\` followed by the delimiter
into the bare delimiter (chapter 10). That is right when the delimiter is `/`
and wrong when it is `.`, because the backslash that made the dot literal was
gone before the regex compiler saw it. The tell was the scanner's own comment,
which said it preserved escapes for the next stage, and then didn't preserve
one of them (#21).

**Context thrown away.** The global substitution loop searched each remaining
piece of the line as if it were a whole line:

```rust,ignore
        match re.find(&text[pos..]) {
```

so `^` matched at the start of every piece, and the loop had no record of
where the previous match had ended (#22). Slicing is cheap and looks harmless.
What it costs is everything the callee could have known about where the slice
came from.

**A position that goes stale.** `g` recorded the line numbers of the matching
lines and visited them after earlier commands had changed the numbering
(chapter 12):

```rust,ignore
                marked.push(n);
```

A number is a position, and positions change when lines are inserted or
deleted above them (#19). The same mistake in a smaller place: the fast path
for `g/re/d` deleted from the bottom up, which keeps the numbers valid, and so
left the current line where the top-down order would not have (#34).

**An error that was dropped.** The `a` command needed a line to append after
and got it from a resolver that rejects address 0:

```rust,ignore
            if !buf.is_empty()
                && let Ok(r) = spec.resolve(buf)
            {
                buf.set_current(r.end);
            }
```

`if let Ok` is the most innocent-looking way there is to discard an error, and
here it turned `0a` into "append wherever the cursor is" (#20).

**A convenience that changes the data.** `str::lines` splits on `\n` and also
removes a `\r` before it. For a program that reads text and never writes it
back, that is a kindness. For an editor, it is a conversion applied to every
Windows file on the way in, undone by nothing on the way out (#18).

**A match that is too loose.** The dispatcher's second stage takes the first
character as the command and the rest as its argument. That made `write` into
`w` with the filename `rite`, and `wq` into `w` with the filename `q` (#14,
#15). The suffix parser asked whether a `g` appeared anywhere after the
replacement and ignored everything else, so a count and a typo both ran as a
plain `s` (#24). Each accepted input it did not understand and did something
plausible with it, which is worse than failing, because it looks like working.

**The wrong order.** `e` emptied the buffer and then loaded the file into it.
When the load failed, there was nothing left to go back to (#17). The repair
is a rule worth having in general: build the new state somewhere else, and
replace the old one only when the new one is complete. Chapter 5 said it about
writing files; it holds for anything that replaces something.

**A guard that guards nothing.** 0.2.0 refused to overwrite a file that had
been detected as UTF-7 or UTF-16. Following the detections through `load_file`
shows that a UTF-16 file never loads, so the only files the guard ever
stopped were ones that had loaded as valid UTF-8, which lose nothing by being
written back (#25). The guard was written for a danger, and nobody checked
that the danger could reach it.

**A document typed rather than captured.** The tutorial teaches `write`
(#16). The README's quick start showed a byte count that did not match its own
file. The help listed seventeen entries and left out three commands that the
dispatcher accepts (#29). Each was written by someone describing what the
program does, and none was produced by running it. Chapter 16 made this
argument; the list shows how many places it applies.

**Behavior learned from an implementation instead of the specification.**
ved was written by comparing it with GNU ed, command by command, and it
matched GNU ed on the commands that were compared. POSIX specifies things
nobody thought to compare: the optional closing delimiter (#23), the count
suffix (#24), `%` (#32), end of input acting as `q` (#33), a nonzero exit
status after an error (#26). None of them was a bug in what ved did. Each was
something ved did not know it was supposed to do.

## 20.4 What found them

| Method | Issues |
|---|---|
| Comparing ved with GNU ed by hand, command by command | #19, #20, #21, #22, #23, #24, #26, #27, #38 |
| Reading the code closely enough to explain it | #14, #17, #18, #25 |
| Checking a document against the binary | #15, #16, #29, #31 |
| Building on older toolchains | #28 |
| Reading the published package metadata | #30 |
| Reading the POSIX specification | #32, #33 |
| The differential harness | #34, #35, #36 |
| The test suite | none |

The last row is the one to think about. ved 0.3.0 had 185 tests, every one
of them passed, and every bug in the table above was present. The fuzzer is
not in the table either, for a different reason: by 0.3.0 the panics had been
fixed, and a million runs found none. It found a new one in chapter 21, which
is the reason to keep running it.

Writing the book was the most productive method, and it is worth being exact
about why. Explaining a function requires reading every line of it with a
question in mind: what does this do with an input its author did not think
of? That is a code review. A book is a code review that cannot skip the dull
parts, because a chapter has to cover the whole command. You do not need to
write a book to get the effect. You need a reader who will ask that question
of every line, and the cheapest one available is you, a week later, explaining
the code to someone in writing.

## 20.5 Why the tests did not

Three reasons, each visible in the suite.

**The inputs were the author's.** Every test buffer was ASCII, every line
ended in `\n`, and every test ran one command. CRLF (#18), the escaped
delimiter (#21), and the stale marks (#19), which need a second command to
show, could not appear.

**The layer that failed had no tests.** The exit status (#26), end of input
(#33), and what reaches standard output (#27) belong to the loop in
`run_repl`, and every test called `dispatch`. Chapter 15 named this layer as
the one nobody tested. It is where four of the issues were.

**Three tests recorded bugs.** `list_tab_becomes_octal` asserts that a tab
lists as `\011`, which POSIX says is `\t` (#38). `join_single_line_is_silent_noop`
asserted that a one-address `j` returned an empty `Print`, which the loop
printed as a blank line. And 0.3.0 added this one, while fixing chapter 9's
corruption:

```rust,ignore
        let mut buf = buf_with(&["café"]);
        let act = dispatch("s/[a-z]*/X/g", &mut buf);
        assert!(!matches!(act, Action::Error(_)));
        assert_eq!(buf.line(1), Some("XXéX"));
```

The test was written to prove that a multi-byte character survives a
substitution, and it does prove that. Its expected value came from running
the code, and the code had #22's bug, so the value has one `X` too many: `sed`
prints `XéX`. A test whose expected value comes from the code under test
cannot disagree with it. Chapter 15 said so in general. This is the
particular case, written into ved's suite two months before chapter 15 was.

## 20.6 The finding the book forgot

Chapter 19 said that a finding left in prose is a paragraph. Chapter 14 found
that `l` lists a tab as `\011` where POSIX and GNU ed write `\t`, said so in a
sentence, and chapter 15 used it as its example of a test that agrees with
the code. It was not in chapter 19's list, because that list was compiled
from the chapters' explicit findings and this one was phrased, both times, as
a point about something else. It was not filed on November 21
with the rest. It was found again while section 20.5 was being written, three
chapters later, and filed as #38.

That is the argument for the habit, made by the book against itself. The
findings that get lost are not the important ones that everybody remembers.
They are the ones that were noticed in passing, while making some other point,
by someone who meant to come back to them.

## 20.7 Which ed

A bug is a difference from what the program should do, so a list of bugs
depends on a reference. ved's has been GNU ed, which is installed on the
machine, answers any question in a millisecond, and is what chapter 1 meant by
"sitting on the machine to compare against." The same sentence in chapter 1
says that POSIX defines what ed does. Most of this book treated those as the
same thing. They are not, and neither is a third reference: the original.

Seventh Edition Unix, from 1979, runs today under the SIMH PDP-11 simulator,
and its `ed` is the one Ken Thompson's design became before anyone
standardized it. `tools/v7/` boots it and types ed commands at its console.
Running the disputed cases on all three gives this:

| Question | POSIX | Seventh Edition | GNU ed 1.21.1 |
|---|---|---|---|
| Exit status after an error | nonzero | 0 | 1 |
| The rest of a script after an error | not specified | dropped | dropped from a file, run from a pipe |
| `,` meaning `1,$` | yes | an error | yes |
| `wq` | not in POSIX | an error | yes |
| `s/a/b` with no closing delimiter | allowed, and prints | an error | allowed |
| A count, `s/a/X/2` | yes | an error | yes |
| `%` as the replacement | yes | a literal `%` | yes |
| `s/a*/X/g` on `baaac` | not specified | `XbXcX` | refuses: "Infinite substitution loop" |
| `0i` | not specified | an error | as `1i` |
| `3,1a` | appends after 1 | an error | appends after 1 |
| The current line after `2j` | unchanged | 2 | unchanged |
| A `\r` before the newline | not addressed | kept | kept |
| `l` of a carriage return | `\r` | `\15` | `\r` |

The table has a shape. Where POSIX speaks, GNU ed almost always agrees with
it, and the Seventh Edition often does not, because POSIX was written later
and fixed things. The exit status row is the clearest case: POSIX's rationale
says that "some historical implementations returned exit status zero even if
command errors had occurred; this is not allowed," and the Seventh Edition is
one of them. Where POSIX is silent, the Seventh Edition and GNU ed can
disagree with each other, and the empty-match row is a case where GNU ed is
the odd one out.

So ved now has a rule, written into its notes and its commit messages. POSIX
decides. Where POSIX says "unspecified," the Seventh Edition decides, because
it is the closest thing there is to what ed is. GNU ed is the comparison that
runs in a millisecond, never the definition, and where it differs from both,
the difference is written down rather than copied.

Some differences from all three are deliberate: ved's confirmations, its
messages, `H` printing a reference instead of toggling verbose errors, and the
long-form command names. A deliberate difference is not a bug, and the way to
keep it from being mistaken for one is to write it down once, where a reader
of the code or the documentation will find it. A difference nobody wrote down
is a bug until someone decides otherwise.

## 20.8 What order to fix them in

The issues were numbered in the order they were filed, and the order was a
rule. A change that loses the user's text without a message comes first,
ranked by how many users reach it: `wq` ends a great many ed scripts, and the
tutorial is the first thing a new user reads. After those come text that ends
up wrong but present, then what scripts see, then the packaging and
documentation that describe the program to people who have not installed it.

The rule is short because it has to be applied quickly, to a list that will
grow. Two refinements were needed in practice. A fix that other fixes depend
on goes first regardless of rank: the long forms (#15) changed the dispatcher
that the help text (#29) and the tutorial (#16) describe. And a finding that
changes behavior some user may rely on, like the exit status (#26), is ranked
by the harm of the current behavior, not by the risk of changing it; the risk
belongs to the release, which is chapter 22's subject.

## 20.9 A report that can be fixed

Here is #20, as filed:

```text
`a` ignores an address it cannot resolve and appends after the current
line. `0a`, the usual way to put text at the top of a file, puts it at
the bottom:

$ printf 'one\ntwo\n' > f.txt
$ printf '0a\nNEW\n.\nw\nq\n' | ved f.txt; echo "exit $?"
8
wrote 12 bytes to f.txt
exit 0
$ cat 'f.txt'
one
two
NEW

GNU ed puts NEW first. [...]

The `a` arm in `dispatch` calls `spec.resolve(buf)`, which rejects 0
because ranges start at 1, and discards the error with `if let Ok`.
```

It has four parts, and each one saves the person fixing it a step. A sentence
saying what is wrong, in the user's terms. A session that anyone can paste
into a shell, with the output it produced, so the first thing the fixer does
is see it fail. The reference's output for the same input, so there is no
argument about what right looks like. And the cause, where the reporter knows
it, which turns the fix from a search into an edit.

The sessions were not typed. Each was generated by a script that ran the
commands and pasted what came out, for the same reason chapter 16 gave for the
tutorial: a report that cannot be reproduced exactly is a report that starts
an argument. One issue was wrong anyway. #15 first claimed that GNU ed accepts
`wfoo`. Checking it while fixing #15 showed that GNU ed rejects it, as chapter
14 had said, and the issue was corrected before the fix was written. A report
is a claim, and it is checked like one.

---

**Exercise 20-1.** Run `tools/bugs-by-release.sh` against `examples/ved` and
`examples/ch19-living`. Then pick one issue and find the commit that
introduced it with `git bisect` in ved's history, using the reproduction from
the issue as the test. How many of the fourteen were born in the commit that
first wrote their feature?

**Exercise 20-2.** Take the causes in section 20.3 and search a program of
your own for each one: a `let Ok` or `.ok()` that discards an error from user
input, a `lines()` on data you write back, a slice passed to a function that
needs to know where it came from, a stored index into a collection that
changes. Count what you find, and file each one.

**Exercise 20-3.** Write the tests that would have caught #18, #19, and #26,
with expected values from POSIX or the Seventh Edition rather than from
running ved. Run them against `examples/ch19-living` and watch them fail.

**Exercise 20-4.** Find three more rows for the table in section 20.7: cases
where POSIX, the Seventh Edition, and GNU ed might disagree. Run them with
`tools/v7/` and `/usr/bin/ed`, and decide what ved should do for each.

**Exercise 20-5.** Find a test in a project of your own whose expected value
was produced by running the code. Replace the value with one from an
independent source and see whether the test still passes.
