# 14. The Voice of the Program

ed has two things to say. When something goes wrong it prints `?`, and when
something goes right it prints nothing, except a byte count when it reads or
writes a file. Everything else is left to the user's memory of what they typed.

ved exists to replace that, and chapter 1 put the replacement in its design
table as two rows: *full English replacing ed's bare `?`*, and *echo what each
command did*. This chapter is about how those two rows turned out, which means
it is about every string the program prints, who it is printing them for, and
what happens when the reader is not a person.

The slices are eight, nine, ten, and twelve: long-form aliases, friendly errors
and confirmations, the `H` help command, and `l`. The first three are in
`examples/ch13-files`; `l` arrived in 0.1.4 and is in `examples/ved`.

## 14.1 The slice that had nothing to do

The plan gave slice nine a large job: *friendly errors, confirmations, `Q`
(force quit)*. Chapters 2 and 3 described it the same way, as the slice that
would rewrite the program's messages, and arranged the code so that the rewrite
would be contained: errors went through one `Action` variant, and one place in
the loop wrote the `?`.

When slice nine arrived, there was almost nothing to rewrite. Compare every
error string in `examples/ch12-global` with every error string in
`examples/ch13-files`, which contains slice nine, and one string was added,
`cannot open {filename}: {e}`, which belongs to slice eleven's file loading. No
existing message changed. `invalid address`, `no match`, `unterminated
pattern`, and the rest had been written in English as each command was built,
because chapter 5 had made the decision, at slice four, that every command
would say what it did in a sentence, and every command after it was written
that way.

So slice nine shrank to two items. `d` gained its confirmation, `deleted 3
lines (2-4)`, replacing the silence that slice six had copied from ed. And `Q`
arrived, which quits without the unsaved-buffer warning and is one match arm:

```rust,ignore
        "Q" => return Action::ForceQuit,
```

The general point is about plans. A slice scheduled to do something across the
whole program is a slice that assumes the rest of the program will be built
without it. If the decision it represents is made earlier, and applied as the
program grows, the slice has nothing to do, and that is the good outcome: a
sweep that never needs to happen is cheaper than one that does, by the size of
the sweep. The slice plan said when the messages would be finished. The code
decided it was every time a message was written.

## 14.2 An inventory

Run every command that changes the buffer and record what it says. The result
is the program's voice, measured rather than intended:

| Command | ved prints | GNU ed prints |
|---|---|---|
| `w out.txt` | `wrote 14 bytes to out.txt` | `14` |
| `e`, `r`, opening a file | `14` | `14` |
| opening a missing file | `notes.txt: new file` | `notes.txt: No such file or directory` |
| `2d` | `deleted line 2` | nothing |
| `2,3d` | `deleted 2 lines (2-3)` | nothing |
| `2,3m1` | `moved 2 lines (2-3) to after 1` | nothing |
| `1m2` | `moved 1 lines (1-1) to after 2` | nothing |
| `1,2t$` | `copied 2 lines (1-2) to after 6` | nothing |
| `1,2j` | `joined 2 lines (1-2) into line 1` | nothing |
| `s/a/b/` | the last changed line | nothing |
| `a`, `i`, `c` | nothing | nothing |

Three voices are in that table. `w`, `d`, `m`, `t`, and `j` speak in sentences.
`e` and `r` print ed's bare byte count, which is the one piece of output ed
already had, kept unchanged when every other command gained a sentence. And
`a`, `i`, and `c` say nothing, which is arguably right, since what they did is
on the screen in the lines the user just typed.

Each confirmation is a `format!` call in the command that produces it, with no
shared code. That is why `deleted line 2` handles the singular and
`moved 1 lines` does not: `run_delete` was written with an `if count == 1`, and
`run_move` and `run_transfer`, written later, were not. Chapter 2 put the `?`
prefix in one place so that the error format could not drift, and nothing did
the same for confirmations. A helper would:

```rust,ignore
/// "line 3" or "3 lines (2-4)", for confirmations.
fn describe_lines(start: usize, end: usize) -> String {
    if start == end {
        format!("line {start}")
    } else {
        format!("{} lines ({start}-{end})", end - start + 1)
    }
}
```

and `run_delete`, `run_move`, `run_transfer`, and `run_join` would say `deleted
{}`, `moved {} to after {dest}`, and so on, with the plural decided once. It is
the same move as chapter 2's, applied to the half of the program's voice that
chapter 2 did not cover.

## 14.3 What the errors say

The errors are better than the confirmations, and the reason is structural.
They pass through one `Action` variant to one `writeln!`, they all have the
same shape, a lower-case phrase after `? `, and most of them say what the
user needs to know: `unterminated replacement`, `no previous substitute`,
`cannot write /nope/x.txt: No such file or directory (os error 2)`.

The weak one is the most common. `invalid address` appears in nine places in
the source, and it is the answer to all of these:

```text
1000p
? invalid address
0p
? invalid address
3,2p
? invalid address
+10p
? invalid address
```

Four different mistakes, one message. The first is a line past the end, the
second is the line that does not exist, the third is a backwards range, and the
fourth is an offset past the end. The code that rejects them, in `resolve_with`
and `resolve_one`, knows which case it is in, because each is a separate
condition, and it knows the buffer's length. `line 1000 is past the end (the
buffer has 3 lines)` costs one `format!` per case and is what a user who typed
`1000p` by mistake needs to see. It is ed's `?` with the word "address" added.

The `?` itself is the other question, and chapter 5 deferred it: the warning on
`q` with unsaved changes is printed as `? warning: buffer modified`, through the
error channel, although nothing failed. That was right, for a reason that has
nothing to do with the user at the terminal. GNU ed prints `?` for the same
warning, and a script that drives ed looks for the `?`. ved's warning is an
error in format because ed's is, and a program that promises compatibility
inherits its predecessor's decisions about what counts as an error, including
the ones it would not have made itself.

## 14.4 Who is listening

All of it goes to standard output. That includes the lines the user asked to
see, the confirmations, and the errors, in the order they happen. At a terminal
that is correct: the user sees one stream, in sequence, and every line is about
the command just above it.

A script is a different reader. ed scripts are usually run like this:

```text
$ ed -s notes.txt < edits.ed
```

`-s` is ed's flag for silence. It suppresses the byte counts and the
diagnostics, so the only output is whatever the script asked to print. It is
the most common way ed is invoked from a shell, and ved does not have it:

```text
$ ved -s notes.txt < edits.ed
ved: unknown option: -s
try 'ved --help' for usage information
$ echo $?
2
```

Without `-s`, a script that runs ved and captures its output gets the
confirmations mixed into the text:

```text
$ printf ',p\nd\nw\nq\n' | ved greeting.txt > captured.txt
$ cat captured.txt
6
hello
deleted line 1
wrote 0 bytes to greeting.txt
```

The script asked for one line, `hello`, and got four.

The third difference is the exit status. GNU ed, reading commands from
something that is not a terminal, exits 1 if any command failed, so a shell
script can write `ed -s file < edits.ed || exit 1` and stop when an edit did
not apply. ved exits 0 after any number of errors:

```text
$ printf 'zz\nq\n' | ved notes.txt; echo $?
22
? unknown command: zz
0
```

These are three halves of one problem. ved's voice was designed for the person
at the keyboard, and the design table says so. It made no provision for the
other reader, and ed's compatibility with scripts lives almost entirely in how
it treats that reader: silence on request, and a status that reports failure.
The ved README describes its compatibility as close enough that *most* scripts
written for ed run unchanged, which is accurate, and the scripts that do not
are mostly the ones that use `-s`. The flag and the exit status are filed as
[ved #27](https://github.com/excelano/ved/issues/27) and [ved
#26](https://github.com/excelano/ved/issues/26).

Chapter 12 left a question here, about whether `g/TODO/d` over a file with no
matches should print `? no match`, as ved does, or nothing, as ed does. The
answer is that it depends on who is listening, and that is the design ved
needs: one flag that changes the reader.

```rust,ignore
enum Action {
    Quit,
    ForceQuit,
    EnterInputMode,
    Done,
    Print(String),
    Confirm(String),
    Error(String),
}
```

The flag cannot be implemented with the `Action` the program has. `Print` is
used for both the output the user asked for, the lines `p` prints, and the
confirmations the program volunteers, `deleted line 1`, and the loop cannot
tell them apart, so it cannot suppress one without the other. A `Confirm`
variant, used by every command that reports what it did, gives the loop the
distinction. Then `-s` sets a flag in `run_repl`, the `Confirm` arm prints only
when the flag is clear, and the `Error` arm records that an error happened so
that the loop can return `ExitCode::FAILURE` at the end. No command changes
except to return a different variant.

This is chapter 2's exercise 2-1 arriving for real. It asked where a quiet
flag should live, in `dispatch` or in the loop, and the answer is the loop, for
the reason chapter 2 gave: output is the loop's job. What chapter 2 could not
show is that the answer depends on the types. A flag in the loop is only as
precise as the distinctions `Action` makes, and an enum that merges two kinds
of output into one variant has decided, in advance, that no reader will ever
want one without the other.

## 14.5 Long names

```rust,ignore
        "q" | "quit" => return Action::Quit,
        "a" | "append" => {
        // ...
        "d" | "delete" => return run_delete(&spec, buf),
        "p" | "print" => return run_print(&spec, buf, PrintMode::Plain),
        "n" | "number" => return run_print(&spec, buf, PrintMode::Numbered),
        "l" | "list" => return run_print(&spec, buf, PrintMode::List),
        "j" | "join" => return run_join(&spec, buf),
        "H" | "help" => return run_help(),
```

Slice eight is a second pattern in eight match arms, and it is safe for a
reason worth checking rather than assuming. A long name is only harmless if it
is not already a valid ed command, since `delete` might be `d` followed by
something ed would accept. Each of ved's long names typed into GNU ed is an
error, `? Invalid command suffix` or similar, so no script written for ed can
contain one meaning something else. The aliases add vocabulary without changing
the meaning of anything that was already legal.

Chapter 10 said `s` could have no long name, and the plan's design table says
why: *substitute, global, and inverse global stay short-only because their
`/foo/bar/` argument syntax makes long names ambiguous*. `subst/a/b/` cannot be
distinguished from `s` with `u` as its delimiter, and `global/x/p` from `g`
with `l` as its delimiter. The rule is that a command whose first argument
character is arbitrary cannot have a name longer than one letter.

The same table promised three more: `write`, `read`, and `edit`. They were
never implemented. The plan marks slice eight as done, and its example, in the
slice list, is *long-form command aliases (`append`, `write`, `quit`, etc.)*.
What `write` does instead:

```text
$ ved notes.txt
22
write
wrote 22 bytes to rite
edit
dit: new file
```

`write` saved the buffer to a file named `rite`, and `edit` opened a file named
`dit` and, because of chapter 13's `e`, discarded the buffer to do it. Neither
said anything that looked wrong. A user who read the plan, or guessed the name
from `append` and `delete`, gets a file with a strange name, or loses their
work. Filed as [ved #15](https://github.com/excelano/ved/issues/15).

The same mechanism reaches the most common command in ed scripts. `wq`, write
and quit, is how a great many of them end:

```text
$ printf 's/hello/bye/\nwq\n' | ved f.txt
6
bye
wrote 4 bytes to q
```

ved read `wq` as `w` with the filename `q`. It wrote the edited buffer to a new
file named `q`, left `f.txt` unchanged, did not quit, and exited 0 when its
input ran out. A script that ends in `wq` runs to completion under ved, reports
success, and does not change the file it was written to change. GNU ed writes
`f.txt` and quits. Filed as [ved
#14](https://github.com/excelano/ved/issues/14).

The cause is in the second stage of the dispatcher, which takes the first
character of the command as the letter and everything after it as the argument,
with `trim_start()` removing any space. `w file` and `wfile` are the same
command to ved. GNU ed requires the space, or the end of the line, after `w`,
`e`, and `r`, and rejects `write` with `? Unexpected command suffix`, which is
why the name was safe to give a meaning in the first place. ved accepted the
spaceless form, so the name had a meaning already, and it was not the one the
plan intended.

The fix has two parts and the order matters. First, require whitespace or the
end of the line after `e`, `r`, and `w`, so that `write` becomes `? unknown
command: write`, which is at least true. Then add the aliases. Doing only the
second, by putting `"write"` in a match arm, would fix the word and leave
`wnotes.txt` as a way to write to `notes.txt` that ed scripts do not expect.
This is in 0.3.0 unchanged.

## 14.6 Help, by hand

```rust,ignore
fn run_help() -> Action {
    Action::Print(
        "\
ved commands (addresses shown in brackets are optional):

  [.]a, append       Append text after the addressed line. End with '.'
  [.]i, insert       Insert text before the addressed line. End with '.'
  // ... fifteen more lines ...
"
        .to_string(),
    )
}
```

`H` prints a reference, and it is a string literal. Chapter 6 said the help was
a separate hand-written list of the commands, and that this chapter would admit
what that costs. Here is the cost, found by comparing the string with the
dispatcher:

`e`, `r`, and `Q` are implemented and missing from the help. The README's
command table lists all three, so the two documents that describe the commands
disagree with each other as well as with the code. `g` and `v` are shown as
`[.,.]g/re/cmd`, which says the default address is the current line; it is the
whole buffer, `1,$`, which is the most important fact about `g`. And `w` is
shown as `[.,.]w`, with the text *default: all*, which contradicts its own
brackets. Filed as [ved #29](https://github.com/excelano/ved/issues/29).

None of this can be caught by a test of the kind the program has, since the
help is a string and the tests check behavior. It can be caught by a test of a
different kind, cheaply:

```rust,ignore
    #[test]
    fn help_mentions_every_command() {
        let Action::Print(help) = run_help() else { panic!() };
        for cmd in ["a", "i", "c", "d", "p", "n", "l", "j", "m", "t",
                    "s", "g", "v", "w", "e", "r", "q", "Q", "H"] {
            assert!(help.contains(&format!("{cmd} ")) || help.contains(&format!("{cmd},")),
                    "help does not mention {cmd}");
        }
    }
```

The list in the test is a third copy of the command set, which is the obvious
objection to it, and it is still better than two copies that drift silently,
because when it drifts it fails. The real fix is the one chapter 6 decided
against: a table of commands that both the dispatcher and the help are
generated from. Chapter 6 named the condition under which a table wins,
something other than the dispatcher needing to enumerate the commands, and
noted that the help is such a thing. Three copies of a list, and four mistakes
in the one nobody tests, are what leaving the table unbuilt cost.

`H` itself is a compatibility decision. In ed, `H` toggles help mode, in which
every `?` is followed by an explanation, which is the closest ed comes to ved's
purpose. ved prints English errors all the time, so the toggle would do
nothing, and the letter was reused for the reference. A script that sends `H`
to get verbose errors gets the help text in its output instead, which is
harmless and is one more line on the list of things a script does not expect.

## 14.7 Showing what is invisible

`l` arrived in 0.1.4, as slice twelve. It prints lines with the characters a
terminal would hide made visible, and it ends each line with `$` so that
trailing spaces can be seen:

```text
a
café	x\y
.
l
caf\303\251\011x\\y$
```

Non-ASCII bytes as three-digit octal, a backslash doubled so that it cannot be
mistaken for an escape, and `$` at the end. That is POSIX's format, and GNU ed
agrees on the `é`, which is two bytes in UTF-8 and is shown as both of them.

It disagrees on two things. POSIX says that `\t`, `\a`, `\b`, `\f`, `\r`, and
`\v` are written as those escapes rather than in octal, so GNU ed prints the tab
as `\t` and ved prints `\011`. And POSIX says that long lines are folded, with
a backslash at the point of folding, so that a line of a hundred characters is
shown as two lines of output on a narrow terminal. ved prints it as one.

The tab is held in place by a test named `list_tab_becomes_octal`, which
asserts ved's behavior rather than the standard's. That is the shape chapter 11
found with `join_single_line_is_silent_noop`: a test that describes what the
code does, written at the same time as the code, so that when the code was
wrong the test agreed with it. Chapter 15 is about how to write tests that can
disagree.

## 14.8 What the voice bought

Errors that say what went wrong, a help command, long names for the commands
that can have them, `l`, and a confirmation for most commands that change the
buffer. For the person at the keyboard, ved does what chapter 1 said it would.

For a script, it is louder than ed and reports success after a failure, and the
gap between the two readers is one flag and one `Action` variant wide. The
inventory in section 14.2 found three styles of confirmation, the help was
missing three commands, and the plan's three remaining aliases did something
other than what their names said. None of that was decided. It accumulated in
the places where each command spoke for itself, and the parts of the voice that
were consistent were the parts that went through one function.

---

**Exercise 14-1.** Add `describe_lines` from section 14.2 and use it in every
confirmation. Then decide whether `e` and `r` should say `read 22 bytes from
notes.txt`, and whether doing so breaks anything a script depends on.

**Exercise 14-2.** Make `invalid address` say which of the four cases in
section 14.3 occurred. Keep the phrase `invalid address` at the start of each
message, and decide why that might matter.

**Exercise 14-3.** Implement `-s`: add `Action::Confirm`, move every
confirmation to it, suppress confirmations and byte counts when the flag is
set, and exit 1 if any command failed. Then run an ed script that uses `-s`
against both programs and diff the output.

**Exercise 14-4.** Require whitespace after `e`, `r`, and `w`, then add
`write`, `read`, and `edit`. Write the test that `write` does not create a file
named `rite`.

**Exercise 14-5.** Replace the help string and the dispatcher's match with one
table of commands, and generate the help from it. Keep the match for
dispatching if you can. Count the lines, compare with chapter 6's exercise 6-2,
and decide whether the answer flipped.

**Exercise 14-6.** Make `l` print `\t` for a tab and fold lines longer than 72
columns, as POSIX specifies. Fix the test that asserts the old behavior, and
decide what the folding width should be on a terminal that is wider.
