# 12. The Global Command

`g/re/p` is three of ed's ideas in five characters: select every line that
matches a regular expression, and print it. It was useful enough that in 1973
it became a program of its own, named after the command, and most people who
type `grep` today have never typed `g`.

Slice seven adds `g` and its inverse, `v`, which selects the lines that do
*not* match. The checkpoint is `examples/ch12-global`, and its `run_global` is
the same function, line for line, as the one in the shipped program.

```text
$ cargo run -q -p ved
a
fn main() {
    // TODO: parse args
    let x = 1;
    // TODO: handle errors
}
.
g/TODO/n
2	    // TODO: parse args
4	    // TODO: handle errors
v/TODO/p
fn main() {
    let x = 1;
}
g/TODO/s/TODO/DONE/
    // DONE: parse args
    // DONE: handle errors
g/^ *\/\//d
,p
fn main() {
    let x = 1;
}
```

The interesting part of `g` is not the matching, which is chapter 7 through 9's
engine called once per line. It is that the command after the pattern can
change the buffer that `g` is iterating over, and this chapter is about what
that does to a loop.

## 12.1 Two passes

```rust,ignore
/// Global command: scan the addressed range for lines matching
/// (or not matching, if `invert` is true) a pattern, then execute
/// a command on each one. Default address: whole buffer (1,$).
///
/// `g/pattern/p` is literally "global regex print" — the ancestor
/// of the grep command.
///
/// Delete is special-cased (reverse iteration to keep line numbers
/// stable). All other commands dispatch through the normal path.
fn run_global(spec: &Spec, buf: &mut Buffer, args: &str, invert: bool) -> Action {
```

The first half parses the same way `s` does: a delimiter, `scan_delimited` for
the pattern, and whatever remains is the command to run, defaulting to `p` when
it is empty. The range defaults to the whole buffer through
`resolve_or_whole`, the second of chapter 4's defaults, since `g` with no
address means every line.

The second half is ed's specification, which describes `g` as two passes. The
first pass marks every line in the range that matches. The second pass visits
each marked line, makes it current, and runs the command. Doing it in two
passes is not an optimization. It is what keeps the command's effects from
feeding back into the selection: `g/a/s/a/b/` must not, after changing a line,
reconsider whether that line still matches, and `g/x/t.` must not run again on
the copies it is making.

```rust,ignore
    let mut marked = Vec::new();
    for n in range.start..=range.end {
        if let Some(line) = buf.line(n) {
            let matched = re.find(line.as_bytes()).is_some();
            if matched != invert {
                marked.push(n);
            }
        }
    }
```

`matched != invert` is all of `v`. When `invert` is false the line is marked if
it matched; when it is true, if it did not. `!=` on two booleans is exclusive
or, and it is the shortest correct way to write "flip this condition when that
flag is set". It is why the dispatcher has `g` and `v` calling one function
with a flag, which chapter 6 pointed to as a signature that says what the
difference between two commands is.

## 12.2 A mark is a number

`marked` is a `Vec<usize>`, a list of line numbers. That is the decision this
chapter is about, and the second pass shows what it assumes:

```rust,ignore
    // Delete is special: remove marked lines bottom-to-top so
    // earlier line numbers stay valid.
    if cmd == "d" {
        for &n in marked.iter().rev() {
            buf.delete_range(n, n);
        }
        return Action::Done;
    }

    // For all other commands: set current to the marked line and
    // dispatch normally. Works for p, n, s (which don't change
    // line count) and anything else that operates on current line.
    let mut output = String::new();
    for &n in &marked {
        buf.set_current(n);
        match dispatch(cmd, buf) {
            // ... collect output ...
        }
    }
```

A line number is a position, and a position is only meaningful until something
is inserted or removed in front of it. The comment states the condition under
which the loop is correct: commands that do not change the line count. `p`,
`n`, `l`, and `s` qualify, and those are most of what anyone uses `g` for.

`d` does not qualify, and the special case handles it with the standard trick:
delete from the bottom up, so that every deletion happens after all the
positions still waiting to be visited, and none of them shift. It is correct,
and it is also an admission. The loop's representation is wrong for one of the
four commands the slice was written to support, and the fix was to route that
command around the loop.

The special case matches the string `"d"`. It does not match `.,+1d`, or `-1d`,
or `d` with anything after it, and those go through the general loop with
stale numbers. Neither does it cover `m`, `t`, `j`, `c`, or `r`, which also
change the line count and arrived after `g` did.

## 12.3 What stale numbers do

Take a five-line buffer where the odd lines start with `a`:

```text
a
a1
b2
a3
b4
a5
.
```

`g/a/j` should join each `a` line with the line after it. The marks are 1, 3,
and 5. The first join turns lines 1 and 2 into one line, and now `a3` is line 2
and `b4` is line 3. The loop visits its next mark, 3, and joins `b4` with `a5`:

```text
g/a/j
joined 2 lines (1-2) into line 1
joined 2 lines (3-4) into line 3
? invalid address
,n
1	a1b2
2	a3
3	b4a5
```

GNU ed produces `a1b2`, `a3b4`, `a5`, and reports the error on the last line,
where `j` has nothing to join.

The case that loses data is a command with a relative address:

```text
g/a/-1d
? invalid address
deleted line 2
deleted line 4
,n
1	a1
2	a3
3	b4
```

The intent is to delete the line before each `a` line. The first mark is line
1, `-1` is line 0, and that is an error. The loop carries on. The second mark
is line 3, `-1` is line 2, and `b2` is deleted, which is right. The third mark
is line 5, in a buffer that now has four lines. `set_current(5)` does not check
its argument, `-1` resolves to 4, and line 4 is `a5`: a line that matched the
pattern, that the user was not deleting, and that is now gone, with a
confirmation reporting the deletion of a line number rather than of a line.

GNU ed, given the same command, stops at the first error and changes nothing.

Some commands happen to work. `g/^/m0`, the ed idiom for reversing a file, is
correct, because each move takes a line from below the remaining marks to the
top, and only the lines already visited shift. `g/a/t$` is correct, because the
copies land after every mark. Both are the same kind of correct as `p` and `s`:
the line count changes, but never in front of a position still to be visited.
That is a fact about the arithmetic of particular commands, not about the loop,
and nothing in the code says which commands have it.

All of this has been in the program since slice seven, and it is in 0.3.0. At
0.2.1 the test suite had no tests for `g` at all. 0.3.0 added seven, and the
only one with a command that changes the line count is `g/foo/d`, which takes
the special case.

## 12.4 Marking lines instead of numbers

ed itself does not have this problem, because ed does not mark numbers. It
marks lines. In the Seventh Edition's `ed.c`, the buffer is a table with one
entry per line, the mark is the low bit of the entry, and when a command moves
entries around, the bit moves with them. The second pass scans the table for a
set bit, clears it, makes that line current, runs the command, and starts the
scan again from the top. It does not matter what the command did to the buffer
in between, because the next marked line is found by looking rather than
remembered by position.

ved can do the same thing without a linked list, and the reason it can is
chapter 11's table. Every change to `lines` goes through three methods. If
`Buffer` keeps a second vector of flags, one per line, then two of those
methods have one line each to add and the flags follow the lines wherever they
go:

```rust,ignore
pub struct Buffer {
    lines: Vec<String>,
    marks: Vec<bool>,
    // ...
}

    pub fn append_after(&mut self, after: usize, line: String) {
        self.lines.insert(after, line);
        self.marks.insert(after, false);
        // ...
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        self.lines.drain((start - 1)..end);
        self.marks.drain((start - 1)..end);
        // ...
    }
```

`replace_line` changes no positions, so it needs nothing. The invariant, that
`marks` and `lines` have the same length and the mark at index *i* belongs to
the line at index *i*, is held in `buffer.rs` beside chapter 3's invariant about
`current`, for the same reason. Then:

```rust,ignore
    /// Unmark and return the first marked line, wherever earlier
    /// commands have moved it.
    pub fn take_mark(&mut self) -> Option<usize> {
        let i = self.marks.iter().position(|&m| m)?;
        self.marks[i] = false;
        Some(i + 1)
    }
```

and the second pass becomes one loop with no special case:

```rust,ignore
    let mut output = String::new();
    while let Some(n) = buf.take_mark() {
        buf.set_current(n);
        match dispatch(cmd, buf) {
            Action::Print(msg) => {
                // ... append to output ...
            }
            Action::Error(msg) => {
                buf.clear_marks();
                // ... append "? msg" to output ...
            }
            _ => {}
        }
    }
```

The bottom-up delete is gone, because a deleted marked line takes its mark with
it and the next `take_mark` finds whatever is marked now. `clear_marks` on the
first error is ed's rule, which stops the command rather than continuing with a
buffer the user did not expect.

Built against `examples/ved` and run beside GNU ed on sixteen global commands,
among them every failure above and `g/b/.,+1j`, `g/b/1,.d`, `v/b/m0`, `g/a/m$`,
and `g/./t.`, this version produced the same buffer as ed in all sixteen. The
125 existing tests pass unchanged.

`take_mark` scans from the start each time, so a `g` over *n* lines with *m*
marks is O(*n* × *m*) in the worst case. The Seventh Edition loop has the same
shape, and for a buffer of a hundred thousand lines it is still faster than printing the
result. A program that needed better would keep the scan position and restart
from it, which is correct only if commands never move a marked line upward,
and `m0` does.

One difference from ed remains. `m` and `t` are built from `delete_range` and
`append_after`, so a moved line is a new line and arrives unmarked. In ed a
moved line keeps its mark. It matters only for a command like `g/a/.,+2m0`,
where the range being moved contains marks not yet visited, and the fix is the
`splice` method that exercise 11-4 asked about, made to carry the marks across.

## 12.5 `dispatch` calls `dispatch`

The second pass runs its command with `dispatch(cmd, buf)`, the same function
the loop calls for every command the user types. `run_global` is called from
`dispatch`, so this is recursion, and it works without any preparation because
of a property chapter 2 established for other reasons: `dispatch` holds no
state. It is a function from a string and a buffer to an `Action`, and calling
it from inside itself is no different from calling it twice in a row.

That is the cheapest feature in this chapter. Every command in the program
became something `g` can run, including commands added years later, without
`g` knowing they exist. `g/x/l` worked the day `l` was added.

It also means `g` can run itself, which ed forbids:

```text
g/a/g/1/d
? no match
? no match
```

The inner `g` runs once per outer mark, on the whole buffer, because its range
defaults to `1,$` rather than to the current line. Two of its three runs find no
line containing `1` and report it, and the first run deletes `a1`. GNU ed
refuses with `? Cannot nest global commands`, and in a recursive design the
refusal is three lines at the top of the second pass:

```rust,ignore
    if cmd.starts_with(['g', 'v']) {
        return Action::Error("cannot nest global commands".to_string());
    }
```

Recursion through a shared dispatcher gives every command to every other
command, and some of those combinations are meaningless. The property is worth
having, and it comes with the job of listing the combinations that should not
be allowed, because the type system will not.

## 12.6 What `g` cannot run

Two kinds of command are swallowed by the `_ => {}` arm, and in both cases the
user is not told.

`q` inside `g` does nothing. ed treats it as a quit, subject to the usual
warning. Nobody types `g/x/q` on purpose, so the difference is small, but the
silence is the same silence as chapter 11's `if let Ok`: an `Action` that the
loop would have acted on, discarded by a match arm that exists to make the
match exhaustive.

`a`, `i`, and `c` return `EnterInputMode`, and the arm drops that too:

```text
g/x/a
new
? unknown command: new
```

The append does nothing, and the text meant for it is read as the next command.
This one is a limit of the design rather than a dropped value. Input mode
belongs to the loop, and `g` is inside `dispatch`, which cannot read from the
terminal. ed solves it with command lists: a `g` command can span several
lines, each ending in a backslash, so `g/x/a\` followed by the text puts the
text after every matching line. ved reads one line per command and has no
continuation syntax, so `g` can only run commands that fit on one line and need
no input.

Implementing command lists would mean giving `dispatch` a way to ask for more
input, which is the first thing in the program that would break chapter 6's
rule that the loop owns the terminal. The design that keeps the rule is to
collect the continuation lines in the loop, before dispatch, and pass the
command list in as one string. It is the right design and it is not small.

## 12.7 Output, collected

```rust,ignore
            Action::Print(msg) => {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&msg);
            }
            Action::Error(msg) => {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!("? {msg}"));
            }
```

Every line of output from the inner commands is gathered into one string and
returned as a single `Print`, because `run_global` is in `dispatch` and does not
get to write. That is chapter 3's choice for `,p` again, with the same cost: a
`g/./p` over a large file holds the whole output in memory before any of it is
shown.

The `Error` arm is the interesting one. It formats `? {msg}` itself, which is
the only place outside `run_repl` that knows what an error looks like. Chapter 2
moved the `?` prefix into the loop so that one place would decide the format,
and this arm is a second place, written because `Action` has no way to carry a
mixture of output and errors. If the format of errors ever changes, this is
the line that will not change with them.

A `g` that matches nothing is `? no match`. ed says nothing, on the reasoning
that "for every matching line" over zero lines is a command that did nothing
wrong. ved's version is consistent with `s`, which does report it, and it means
a script running `g/TODO/d` over a clean file sees a `?` that ed would not have
printed. Which of those is right is the question of chapter 14.

## 12.8 What slice seven bought

The ancestor of `grep`, built on top of everything already there: the scanner
from `s`, the address default from chapter 4, the engine from chapters 7
through 9, and the dispatcher from chapter 2, called recursively. `run_global`
is eighty-four lines, and nearly all of them are about running other commands rather
than about matching.

The representation chosen for the marks was the first one that came to mind,
it was right for the commands the slice was tested with, and the special case
for `d` was the sign that it was wrong. That pattern is common enough to be
worth a rule. When a loop needs a special case for one of the operations it was
written to perform, the representation is the thing to question before the loop
is.

---

**Exercise 12-1.** Apply the marks change from section 12.4 to `examples/ved`.
Before you do, write tests for `g/a/j` and `g/a/-1d` on the buffer from section
12.3 and watch them fail.

**Exercise 12-2.** `set_current(5)` on a four-line buffer succeeded silently,
and that is half of why `g/a/-1d` deleted the wrong line. Make `set_current`
refuse an out-of-range line, decide what "refuse" means for a method that
returns nothing, and find every caller that breaks.

**Exercise 12-3.** Make `g/a/.,+2m0` keep the marks of the lines it moves, as
ed does. You will need a `Buffer` method that moves a range in one operation.

**Exercise 12-4.** Implement ed's command lists for `g`: a command ending in a
backslash continues on the next line. Keep the rule that `dispatch` never reads
from the terminal.

**Exercise 12-5.** `grep` began as `g/re/p` run non-interactively over a file.
Write it with ved's modules: a binary that takes a pattern and a filename,
loads the file into a `Buffer`, runs `g/pattern/p`, and exits 1 if nothing
matched. Count how many lines of new code it took.
