# 11. Commands That Move Lines

Slice six was two commands: `d` deletes lines and `i` inserts before one. It is
the checkpoint in `examples/ch11-editing`, and it is small. Four more commands
of the same kind arrived after the first release, as users asked for them: `c`
in 0.1.1, then `j`, `m`, and `t` together in 0.1.5. They live in
`examples/ved`, and this chapter takes all six together, because they share a
problem the substitute command did not have. They change which lines exist, not
what is in them, so every one of them has to decide where the current line goes
and what happens to the addresses of everything after it.

```text
$ cargo run -q -p ved
a
one
two
three
four
five
.
2,3d
deleted 2 lines (2-3)
p
four
$m0
moved 1 lines (3-3) to after 0
1t$
copied 1 lines (1-1) to after 3
2,3j
joined 2 lines (2-3) into line 2
,n
1	five
2	onefour
3	five
```

One thing in that session is wrong, twice, and it is not in the output of
`,n`. Section 11.6 is about it, and it is a plural.

## 11.1 Deleting

```rust,ignore
    /// Delete lines from `start` to `end` (1-indexed, inclusive).
    /// Updates current to the line after the deleted range, or the
    /// new last line, or 0 if the buffer is now empty.
    pub fn delete_range(&mut self, start: usize, end: usize) {
        self.lines.drain((start - 1)..end);
        self.modified = true;
        if self.lines.is_empty() {
            self.current = 0;
        } else if start <= self.lines.len() {
            self.current = start;
        } else {
            self.current = self.lines.len();
        }
    }
```

`drain` removes a range from a vector in one move, shifting what follows once
rather than once per line. The range is the conversion chapter 3 promised would
happen in one place: `start - 1` to get a 0-indexed start, and `end` unchanged,
because an inclusive 1-indexed end and an exclusive 0-indexed end are the same
number. That is chapter 3's cancelling off-by-one again, in a different
function, and the doc comment does not mention it. A reader who "fixes" it to
`(start - 1)..(end - 1)` gets a delete that leaves the last line of every range
behind.

The three branches that follow are ed's rule for where you are after a delete.
If anything survives after the deleted range, the current line is the first of
those, which now has the number `start`. If the range ran to the end of the
buffer, the current line is the new last line. If nothing survives, it is the
sentinel 0. The checkpoint crate shows the first two on a four-line buffer:

```text
2d
p
three
$d
p
three
```

The rule exists so that repeated `d` works the way someone deleting a paragraph
expects. Each `d` removes the line you are on and leaves you on the next one,
and at the end of the buffer it leaves you on the last line rather than on
nothing. It is the kind of rule nobody would design from first principles and
everybody relies on once they have used it.

At slice six this command printed nothing, which was ed's behavior, and
returned a new `Action` variant:

```rust,ignore
enum Action {
    Quit,
    EnterInputMode,
    Done,
    Print(String),
    Error(String),
}
```

`Done` is "the command succeeded and has nothing to say". The loop handles it
by resetting `quit_warned` and printing nothing. It arrived here because `d` was
the first command that changed the buffer without any text to show for it, and
`Print(String::new())` would have printed an empty line. Slice nine gave `d` a
confirmation, `deleted 2 lines (2-3)`, so `d` no longer uses it. Chapter 12's
`g` does.

## 11.2 Insert is append with a different number

```rust,ignore
        "i" | "insert" => {
            // Insert before the addressed line (default: current).
            // Implemented as "append after address - 1", so the
            // input-mode loop (which always does append_after) works
            // unchanged. On an empty buffer or address 1, current
            // becomes 0 (the before-first-line sentinel).
            if !buf.is_empty() {
                let addr = match spec.resolve(buf) {
                    Ok(r) => r.end,
                    Err(e) => return Action::Error(e),
                };
                buf.set_current(addr.saturating_sub(1));
            }
            return Action::EnterInputMode;
        }
```

Input mode has one operation, `buffer.append_after(buffer.current(), line)`,
and it was written in chapter 3 for `a`. Inserting before line *n* is appending
after line *n* − 1, so `i` sets the current line and enters the same mode. The
loop never learns that there are two commands, and when `c` arrives it will not
learn about a third.

This is the cheapest kind of reuse there is: noticing that a new operation is
an old one with a different argument. It works here because the sentinel was
designed in chapter 3 to mean "before the first line", so `1i` becomes
`append_after(0, ...)`, which was already the right call. The doc comment
mentions the sentinel by name, which is what keeps the trick from looking like
an accident.

Slice six also changed `a`. Until then `a` ignored its address entirely and
always appended after the current line, so `1a` appended wherever the current
line happened to be. The
commit message lists it as a fix made in passing, *also fixed a to respect
address prefixes*, and the fix is the subject of the next section.

## 11.3 One error kept, one thrown away

```rust,ignore
        "a" | "append" => {
            // Append after the addressed line (default: current).
            // On an empty buffer, current is 0 and append_after(0)
            // inserts at the start — correct.
            if !buf.is_empty()
                && let Ok(r) = spec.resolve(buf)
            {
                buf.set_current(r.end);
            }
            return Action::EnterInputMode;
        }
```

Put that beside the `i` arm and the difference is one pattern. `i` matches on
the result of `resolve` and returns the error. `a` uses `if let Ok` and ignores
it. When the address is bad, `i` reports it and `a` appends after the current
line as though no address had been typed:

```text
1p
one
5a
carried on
.
,n
1	one
2	carried on
3	two
4	three
```

GNU ed answers `5a` on a three-line buffer with `? Invalid address` and stays in
command mode. ved appended after line 1, which is where the current line
happened to be, and said nothing.

The worse case is an address that is valid in ed and rejected by `resolve`:

```text
1p
one
0a
heading
.
,n
1	one
2	heading
3	two
4	three
```

`0a` is how you put text at the top of a file in ed, and it is one of the
first idioms anyone learns, because `1i` does the same thing and `0a` is one
keystroke shorter to think about. In ved the text goes after the current line.
It is in the right file, it is not lost, and it is in the wrong place, with no
message saying so. This is still the behavior in 0.3.0.

Two decisions combined to produce it, and each was reasonable alone.
`resolve_with` rejects 0, because for `p`, `d`, `s`, and nearly every other
command there is no line 0 to act on. And `if let Ok` in a condition reads like
a guard rather than like discarding an error, especially after it was
reformatted into the `if ... && let` chain that the 2024 edition allows. The
code went through that refactor and nobody looked at what the `else` did,
because there was no `else` to look at.

The 0 address is the real subject. `p` and `d` act on lines, and a line number
must name a line. `a` and `i` put text *between* lines, and an address for them
names a gap, of which there is one more than there are lines. The program
already knew this. Chapter 4's `parse_dest`, written for `m` and `t`, says *the
destination may be 0 ("before line 1")*, because those commands write after
their destination too. The concept existed in one function and was missing from
the two commands that needed it most.

The fix gives `Spec` a method for addressing a gap:

```rust,ignore
    /// Resolve to the single line that input goes after, for `a`
    /// and `i`. A range uses its second address, as ed does, and 0
    /// is allowed: `0a` puts text before the first line.
    pub fn resolve_target(&self, buf: &Buffer) -> Result<usize, String> {
        let (first, last) = match (&self.start, &self.end) {
            (None, None) => return Ok(buf.current()),
            (Some(s), None) => (s, s),
            (Some(s), Some(e)) => (s, e),
            (None, Some(_)) => unreachable!("parser never produces (None, Some)"),
        };
        let (first, last) = (resolve_one(first, buf)?, resolve_one(last, buf)?);
        if first > buf.len() || last > buf.len() {
            return Err("invalid address".to_string());
        }
        Ok(last)
    }
```

Both addresses of a range are checked, though only the second is used, and
neither has to come before the other. That is GNU ed's rule, and it is narrower
than it looks: `9,3a` on a six-line buffer is an error, and `3,1a` appends
after line 1.

and both arms become the same shape:

```rust,ignore
        "a" | "append" => {
            match spec.resolve_target(buf) {
                Ok(n) => buf.set_current(n),
                Err(e) => return Action::Error(e),
            }
            return Action::EnterInputMode;
        }
        "i" | "insert" => {
            match spec.resolve_target(buf) {
                Ok(n) => buf.set_current(n.saturating_sub(1)),
                Err(e) => return Action::Error(e),
            }
            return Action::EnterInputMode;
        }
```

The `if !buf.is_empty()` guards are gone from both. They existed because
`resolve` fails on an empty buffer, and `resolve_target` does not need to: on an
empty buffer the current line is 0, `0a` resolves to 0, and `1a` is larger than
the buffer and is an error, which is what ed says too. This is chapter 5's
pattern of the empty case that has to be spelled out, and here it stops needing
to be spelled out, because the function now describes gaps and an empty buffer
has one.

Run against GNU ed on `0a`, `5a`, `0i`, `2i`, `$a`, `1,2a`, bare `a`, and `0a`,
`1a`, `a`, and `i` on an empty buffer, this version produces the same buffer
in every case and fails in the same cases, and the 125 existing tests pass.
None of them called `a` with an address.

## 11.4 Change

```rust,ignore
        "c" | "change" => {
            let range = match spec.resolve(buf) {
                Ok(r) => r,
                Err(e) => return Action::Error(e),
            };
            let start = range.start;
            buf.delete_range(range.start, range.end);
            buf.set_current(start.saturating_sub(1));
            return Action::EnterInputMode;
        }
```

`c` is `d` followed by `i` at the same address, and the code is that sentence.
The commit that added it in 0.1.1 says *equivalent to d followed by i in one
command*, and it was cheap enough to add in an afternoon because both halves
already existed.

The ordering has a consequence that ed shares. The lines are deleted before the
replacement is typed, so a `c` followed immediately by `.` is a delete:

```text
2c
.
,n
1	one
2	three
```

The alternative, holding the deleted lines until input mode ends and putting
them back if nothing was typed, would need input mode to know which command
started it, and the design in chapter 3 deliberately does not. ed made the same
choice, and the user who wanted the lines back has `u` in ed and nothing in
ved. Section 11.7 is about that.

## 11.5 Join, and a silent no-op that prints

```rust,ignore
fn run_join(spec: &Spec, buf: &mut Buffer) -> Action {
    let range = match spec.resolve_or_pair_with_next(buf) {
        Ok(r) => r,
        Err(e) => return Action::Error(e),
    };
    if range.start == range.end {
        buf.set_current(range.start);
        return Action::Print(String::new());
    }
    let mut joined = String::new();
    for n in range.start..=range.end {
        if let Some(line) = buf.line(n) {
            joined.push_str(line);
        }
    }
    buf.replace_line(range.start, joined);
    buf.delete_range(range.start + 1, range.end);
    buf.set_current(range.start);
    // ... the confirmation
}
```

`resolve_or_pair_with_next` is the third default from chapter 4, and `j` is the
command it was written for: bare `j` joins the current line with the next. The
join is a plain concatenation with no separator, which surprises people who
expect a space and is what ed does.

The single-line case is where the code and its documentation part company. The
doc comment says *a single-line range is a silent no-op*, and a test holds it in
place:

```rust,ignore
    #[test]
    fn join_single_line_is_silent_noop() {
        let mut buf = buf_with(&["one", "two", "three"]);
        let act = dispatch("2j", &mut buf);
        if let Action::Print(s) = act {
            assert!(s.is_empty());
        } else {
            panic!("expected silent Print on single-line join");
        }
        // ...
    }
```

The test passes, and the command is not silent. `Action::Print` goes to a
`writeln!`, and `writeln!` of an empty string writes a newline:

```text
2,2j

p
two
```

ed prints nothing. The fix is `return Action::Done`, which is the variant section
11.1 introduced for this purpose, and which was already in the enum when `j`
was written.

The bug is small and the way it survived is not. The test checks the value
`dispatch` returns, which is the decision from chapter 2 that makes the whole
command language testable without a terminal. It cannot check what the loop
does with that value, and here the meaning lives in the loop: "silent" is a
property of the terminal, and the test is asserting on something one step
before the terminal. The test's name describes the requirement and its body
describes the implementation, and they agree with each other and not with the
user. Chapter 15 comes back to the gap between the two.

## 11.6 Move and transfer

```rust,ignore
fn run_move(spec: &Spec, buf: &mut Buffer, dest_arg: &str) -> Action {
    let range = match spec.resolve(buf) {
        Ok(r) => r,
        Err(e) => return Action::Error(e),
    };
    let dest = match address::parse_dest(dest_arg, buf) {
        Ok((d, _)) => d,
        Err(e) => return Action::Error(e),
    };
    if dest >= range.start && dest <= range.end {
        return Action::Error("invalid destination".to_string());
    }
    let lines: Vec<String> = (range.start..=range.end)
        .filter_map(|n| buf.line(n).map(str::to_string))
        .collect();
    let count = lines.len();
    buf.delete_range(range.start, range.end);
    // Deletion shifts everything past `range.end` down by `count`.
    let adjusted = if dest > range.end { dest - count } else { dest };
    for (at, line) in (adjusted..).zip(lines) {
        buf.append_after(at, line);
    }
    buf.set_current(adjusted + count);
    Action::Print(format!(
        "moved {count} lines ({}-{}) to after {dest}",
        range.start, range.end
    ))
}
```

`2,4m6` moves lines 2 through 4 to after line 6. The destination is parsed by
`parse_dest`, which allows 0, so `m0` moves to the top.

The comment in the middle is the most useful line in the function. Once the
range is deleted, every address after it has moved down by `count`, and the
destination the user typed refers to the buffer as it was. A destination before
the range is unaffected and one after it needs adjusting. That is the classic
bug in any operation that removes and reinserts within one sequence, and the
comment states the reason for the subtraction rather than restating the
subtraction, which is the kind of comment that stops someone from removing it.

A destination inside the range is an error, matching ed, since moving lines 2
through 4 to after line 3 has no meaning. `t`, the copy, allows it: `1,3t2` puts
a copy of all three lines after line 2, and the original lines stay where they
were. `t` is the same function without the delete and without the adjustment,
because nothing moved.

The implementation is built from the buffer's existing operations: collect the
lines, delete the range, append them one at a time. Each `append_after` is a
`Vec::insert`, so moving *k* lines costs *k* shifts of everything after the
destination, where a single `splice` would cost one. For the files ed is used on
it does not matter, and chapter 3's measurement applies. What matters more is
that `m` and `t` did not need new methods on `Buffer`, which is the subject of
the next section.

Two smaller problems are visible in the confirmation. The first is the plural:

```text
1m2
moved 1 lines (1-1) to after 2
```

`run_delete`, written at the same time, checks `count == 1` and says `deleted
line 2`. `run_move` and `run_transfer` do not, so moving one line reads like a
program that has not been finished. It is a one-line fix in each, and it is the
sort of inconsistency that accumulates when each command formats its own
message. Chapter 14 is about the messages.

The second is the `_` in `Ok((d, _))`. `parse_dest` returns the destination and
the rest of the input after it, and `run_move` discards the rest:

```text
2m3xyz
moved 1 lines (2-2) to after 3
```

GNU ed says `? Invalid command suffix`. It is chapter 10's flag parser again: a
parser that returns "what I found and where I stopped" is only strict if the
caller checks where it stopped. Throwing the remainder away with `_` is the
quiet version of `if let Ok`, and it accepts anything a user types after the
address.

## 11.7 Three primitives, and no undo

Six commands in this chapter, and `buffer.rs` gained one method for them:
`delete_range`. Everything else is built from what was already there.

| Command | Built from |
|---|---|
| `a`, `i` | `set_current`, then input mode's `append_after` |
| `d` | `delete_range` |
| `c` | `delete_range`, `set_current`, then input mode |
| `j` | `replace_line`, `delete_range` |
| `m` | `delete_range`, `append_after` |
| `t` | `append_after` |

Every change to the buffer's lines in the finished program, including `s`,
`g`, and `r`, goes through `append_after`, `delete_range`, or `replace_line`,
and `e` starts a new buffer and fills it with `append_after`. That is a narrow interface, and it is the reason chapter 3's
invariant about `current` was still holding at 2,800 lines: three methods
maintain it, and nothing else can reach the fields.

It is also the precondition for the feature ved most conspicuously lacks:

```text
u
? unknown command: u
```

ed's `u` undoes the last command that changed the buffer, and it is the command
that makes `c` followed by `.` survivable. ved has never had it. A program with
three mutation primitives is in a good position to add it, because there are
three places to record what changed: a deleted range and its lines, an inserted
line and its position, a replaced line and its old content. A command would
open a group, its primitives would append to it, and `u` would replay the group
backwards. The cruder version, which copies the whole `Vec<String>` before
every command that changes it, is a dozen lines and costs a buffer's worth of
memory per command.

Either one is less work in this program than in one where commands edit `lines`
directly, and that is the useful general point. Undo is hard to add to a
program after the fact in proportion to the number of places that mutate its
state, and the number here is three. Chapter 19 says why it has not been done.

## 11.8 What slice six bought

At the checkpoint, `d` and `i`, a working `a` address, and an `Action` variant
for commands with nothing to say. In the finished program, six commands built
from three buffer methods, and the set of edits an ed user expects to make.

The bugs this chapter found are all in the gap between a value and what is
done with it. `if let Ok` discarded an error, `Ok((d, _))` discarded the input
after an address, and a test checked a returned value that the loop then turned
into output nobody wanted. In each case the code that produced the value was
right. The mistake was in what the next line did with it.

---

**Exercise 11-1.** Apply the `resolve_target` fix from section 11.3 to
`examples/ved`. Before you do, write a test for `0a` that fails. Then decide
whether `c` should use it too: what should `0c` do, and what does GNU ed say?

**Exercise 11-2.** Fix the single-line `j` to return `Action::Done`, and then
fix the test, which will now fail. Rename the test if the name no longer
describes what it checks.

**Exercise 11-3.** Make `m` and `t` reject trailing input after the
destination. Then grep the program for every other place a parser's remainder
is bound to `_` or never read, and decide for each one whether ed would accept
what ved accepts.

**Exercise 11-4.** Replace the insertion loop in `run_move` with one
`Vec::splice`. That needs a new method on `Buffer`, since `lines` is private.
Decide whether the method is worth adding for `m` and `t` alone, and measure
the difference on a 100,000-line buffer before you decide.

**Exercise 11-5.** Implement `u`, single level, by snapshotting the buffer's
lines and current line before any command that changes them. Then work out
which commands need the snapshot, where in `dispatch` it should be taken so
that a failed command does not overwrite it, and what `u` followed by `u`
should do.
