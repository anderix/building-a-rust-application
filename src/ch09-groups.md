# 9. A Regex Engine: Groups and Backreferences

Two more `Element` variants, one small struct, and a parameter added to three
function signatures. That is all of groups and backreferences, and it is also
the point where the thing being implemented stops being a regular
expression in the formal sense.

`\(ab\)*\1` cannot be recognized by a finite automaton. A backreference asks
the matcher to remember arbitrary text and compare it later, which is outside
what regular languages can do, and it is the reason engines that promise linear
time — RE2, Go's `regexp`, Rust's `regex` crate — do not offer backreferences
at all. ed has had them since 1969, so this engine has them, and section 9.7 is
about what they cost.

## 9.1 What a group has to remember

```rust,ignore
/// Capture state for up to 9 groups, threaded through matching
/// by value. Small enough to copy on every recursive call (9
/// usizes + 9 usizes + 9 bools = ~153 bytes).
#[derive(Debug, Clone, Copy)]
struct Caps {
    start: [usize; NGROUPS],
    end: [usize; NGROUPS],
    valid: [bool; NGROUPS],
}
```

Positions, not text. A group records where it began and where it ended, and
anyone who wants the characters goes back to the original input and slices.
Storing the text instead would mean an allocation per group per match attempt,
and there are a great many failed match attempts.

The three parallel arrays are the unusual part, and `valid` is why. A group
that matched the empty string has `start == end`, and a group that never took
part in the match at all also has nothing between its offsets. Those are
different facts. In `\(a\)*b` matched against `b`, group 1 did not participate,
and `\1` afterwards must fail rather than match nothing. The boolean is what
carries that distinction, because two equal `usize`s cannot.

Fixed-size arrays rather than `Vec` because the specification supplies the
bound: BRE addresses groups as `\1` through `\9`, so nine is not a guess. No
allocation happens anywhere in the matcher, which for a structure copied on
every recursive call is the difference between a fast engine and an unusable
one.

## 9.2 Backtracking, for free

```rust,ignore
            Element::GroupStart(n) => {
                let mut caps = caps;
                if *n <= NGROUPS {
                    caps.start[n - 1] = pos;
                }
                self.match_here(&elements[1..], text, pos, caps, full_text)
            }
```

`caps` arrives by value, `let mut caps = caps` makes it mutable, the copy is
modified, and the copy is what recurses. The caller's `Caps` is untouched.

That one line of shadowing is how backtracking is handled here, and it is worth
sitting with, because getting this wrong is the classic source of bugs in
hand-written matchers. A regex engine explores: it tries a branch, records
partial results, fails, and has to undo them before trying the next branch. An
implementation holding capture state behind `&mut` needs an explicit undo at
every failure point, and the failure points are numerous — every `None` return
in `match_here`, every extra repetition in `match_star`.

With `Caps: Copy` threaded by value, there is nothing to undo. A branch that
fails returns `None` and its modified copy goes out of scope. A branch that
succeeds returns `Some((len, caps))` and hands its copy up. The state that
survives is the state on the path that worked, and the language enforces it
rather than the programmer remembering to.

The cost is 153 bytes copied per recursive call, which the comment states
because somebody will ask. It is the right trade at this size and it would not
be at ninety-nine groups; the version that scales differently keeps a
`Vec<(usize, usize)>` and an explicit undo log, and is the reason production
engines are harder to read than this one.

## 9.3 Markers, not structure

```rust,ignore
enum Element {
    // ...
    GroupStart(usize),
    GroupEnd(usize),
    BackRef(usize),
}
```

A group is not a sub-tree. It is two zero-width markers in the same flat
`Vec<Element>` as everything else, and matching one consumes no input and
advances no position — it writes a number and recurses onward.

Nesting is handled entirely in the compiler, with a stack:

```rust,ignore
                    b'(' => {
                        group_num += 1;
                        group_stack.push(group_num);
                        elements.push(Element::GroupStart(group_num));
                        i += 2;
                    }
                    b')' => {
                        if let Some(n) = group_stack.pop() {
                            elements.push(Element::GroupEnd(n));
                        }
                        i += 2;
                    }
```

Group numbers are assigned in order of opening parenthesis, which is the
universal convention, and the stack is what pairs `\)` with the right `\(` when
they nest. By the time matching starts the nesting has been flattened away
entirely.

This is the design that chapter 8's `Star(Atom)` limitation comes from. Because
a group leaves no structure in the compiled form, there is nothing for `*` to
repeat — the elements between `GroupStart` and `GroupEnd` are not grouped in
any way the matcher can see. Supporting `\(...\)*` means giving the compiled
form the structure this representation deliberately discards.

The `if let Some(n) = group_stack.pop()` is quietly permissive: a `\)` with no
matching `\(` is dropped. Combined with the non-failing compiler from chapter
8, `s/\(ab/X/` compiles to a pattern with an unmatched `GroupStart`, matches
`ab`, and substitutes — no error, and group 1 never becomes valid.

## 9.4 A backreference needs the whole text

```rust,ignore
            Element::BackRef(n) => {
                if !caps.valid[n - 1] {
                    return None;
                }
                let captured = &full_text[caps.start[n - 1]..caps.end[n - 1]];
                if text.starts_with(captured) {
                    let clen = captured.len();
                    self.match_here(&elements[1..], &text[clen..], pos + clen, caps, full_text)
                        .map(|(len, caps)| (len + clen, caps))
                } else {
                    None
                }
            }
```

This is why `match_here` has both `text` and `full_text`. `text` is the
remaining suffix being matched; `caps` holds absolute offsets into the original
input. Slicing `text` with them would be wrong by however much has been
consumed, so the function needs the original to look captures up in.

`pos` is the third of the trio, and it is the absolute position corresponding
to the start of `text` — carried so that `GroupStart` can record where it is in
the coordinate system the captures use. Three parameters to express one idea:
where we are, in two coordinate systems, plus the tape.

The `valid` check earns itself here. A backreference to a group that did not
participate returns `None` rather than matching the empty string, which is what
POSIX requires and what the test named `backref_to_uncaptured_fails` holds in
place.

## 9.5 Nine slots, and the day there were ten

The chapter you are reading found a bug while it was being written. It is
[ved #9](https://github.com/excelano/ved/issues/9), fixed in 0.2.1.

```text
$ ved
a
abcdefghij
.
s/\(a\)\(b\)\(c\)\(d\)\(e\)\(f\)\(g\)\(h\)\(i\)\(j\)/X/
thread 'main' panicked at src/bre.rs:253:17:
index out of bounds: the len is 9 but the index is 9
```

`Caps` has nine slots because `\1` through `\9` are what a BRE can address.
`compile` numbers groups with `group_num += 1` and no bound. A tenth group
produced `GroupStart(10)`, which indexed `caps.start[9]`, which is past the
end.

The specification is the source of the confusion, and it is worth being precise
about what it says: POSIX limits how many groups can be *referenced*, not how
many can *appear*. GNU ed accepts the pattern above and substitutes normally.
So this was a crash and a compatibility gap at once, and the fix follows from
the distinction — record the first nine, let the rest delimit without being
recorded:

```rust,ignore
                if *n <= NGROUPS {
                    caps.start[n - 1] = pos;
                }
```

Since groups leave no structure for the matcher to use, a group that is not
recorded is a group that costs nothing. The nine also became a named constant
rather than a literal appearing in the struct, its constructor, and
`Match::group`, which is how the bug was able to hide: three places agreed on a
number, and a fourth place generated it.

This is the second panic in this book found the same way, after chapter 5's,
and both have the same shape: an index derived from user input, bounded by an
assumption held somewhere else in the file. The first was bounded by "the
command letter is one byte". This one by "there are at most nine groups".
Neither assumption was written down anywhere the compiler could see it.

## 9.6 Six parameters

```rust,ignore
    fn match_here(
        &self,
        elements: &[Element],
        text: &[u8],
        pos: usize,
        caps: Caps,
        full_text: &[u8],
    ) -> Option<(usize, Caps)>
```

Chapter 7's version took two arguments and returned a `bool`. This one takes
six and returns a tuple in an `Option`, and every addition paid for itself when
it arrived: `pos` for group positions, `caps` for the captures,
`full_text` for backreferences, the `usize` in the return so the caller can
compute the end of the match.

It is still too many, and the shape of the fix is standard: three of those six
never change during a match. `full_text` is fixed, `&self` is fixed, and
`elements` changes only by being advanced. A struct holding the invariant parts,
with `text`, `pos`, and `caps` as the moving state, would make the signature
two arguments and a method call.

That it was not done is the same answer as chapter 6's inline commands: nothing
forced it. The signal to watch for is the one visible above — when a parameter
list grows past what fits on a line, and rustfmt breaks it one-per-line, the
formatter is reporting something.

## 9.7 What backtracking costs, measured

This engine tries possibilities until one works. With one star in a pattern
that is linear. With several, the possibilities multiply.

The pattern `a*a*a*a*a*a*a*a*b` matched against a string of *n* a's and no `b`
has to try every way of splitting the a's among the eight stars before it can
report failure. Measured against the release binary:

| a's | `a*a*a*a*a*a*a*a*b` |
|---|---|
| 16 | 0.045s |
| 20 | 0.161s |
| 24 | 0.454s |
| 28 | 1.137s |
| 32 | 2.86s |

Every two characters roughly doubles the time. Forty characters would be about
a minute, fifty about half an hour, and the line is not long — it is shorter
than this sentence.

The same text with the pattern `a*b` takes 0.001s.

This is not a defect in the implementation; it is what backtracking is. The
engines that avoid it — Thompson's NFA simulation, which tracks the set of
states reachable at each position rather than exploring paths one at a time —
run in time proportional to pattern length times text length, always. They also
cannot do backreferences, because tracking a set of states means not knowing
which path you took, and a backreference is a question about the path.

So the choice is not between a good algorithm and a bad one. It is between two
languages: regular expressions, which are fast and cannot express `\(a\)\1`, and
the thing ed actually specifies, which can. ed chose in 1969 and a clone
inherits the choice.

Whether it matters here is a separate question, and the answer is mostly no. ed
patterns are typed by a person, applied to one line at a time, and the
pathological shapes do not occur in anything anyone types on purpose. The place
it could bite is `g/pattern/` over a large file, where a bad pattern is applied
to every line. Nothing in the program guards against it — there is no step
limit and no timeout — and adding one is in the same category as chapter 8's
compile errors: known, cheap, not done.

## 9.8 The engine matches bytes

```rust,ignore
fn atom_matches(atom: &Atom, byte: u8) -> bool
```

Everything in this engine is a `u8`. `Atom::Dot` matches any byte, `One`
advances by one byte, and bracket classes hold bytes. For ASCII that is correct
and fast. For anything else it is wrong, and the way it is wrong is the worst
of the available options:

```text
$ ved
a
café
.
s/./Y/g
,p
YYYYY
```

Five replacements for four characters, because `café` is five bytes and `é` is
two of them. GNU ed prints `YYYY`.

The damaging case is a match that covers part of a character:

```text
$ ved
a
café
.
s/f./Z/
w out.txt
wrote 7 bytes to out.txt
```

`f.` consumed the `f` and the first byte of `é`. The second byte was left
stranded, and `run_substitute` rebuilds the line with
`String::from_utf8_lossy`, which replaces the stranded byte with U+FFFD. The
file on disk holds `caZ` followed by `EF BF BD`. The character is gone, no
error was printed, and the buffer is no longer what the user had.

A text editor that silently alters text it was not asked to alter has a
problem, and this one is worse than a crash: a crash loses the buffer and the
user knows. This loses one character and looks like it worked. It became [ved
#10](https://github.com/excelano/ved/issues/10), and chapter 19 tells how it
was fixed.

The README says, under "Limitations worth knowing", that "UTF-8 text with
multi-byte characters works". That sentence is the part of this that a reader
of chapter 1 should recognize — a document stating what is true, which is no
longer true, and which nobody checked because nothing tests it.

The fix is not small. Byte orientation is correct for the `\0NN` octal escapes
and it is why the engine is simple, so making `.` character-aware means
teaching `One`, `Star`, and the bracket parser to work in characters while
keeping literal bytes addressable. There is a cheaper first step that removes
the data loss without touching the engine: have `run_substitute` refuse a match
whose boundaries fall inside a character, and report it. That converts silent
corruption into an error message, which is the property an editor most needs.
Chapter 19 is where an entry like this one gets written down rather than
discovered.

## 9.9 What slice 5a ended with

An 823-line module implementing literals, `.`, `*`, `^`, `$`, bracket
expressions with POSIX's positional rules, escaped specials, capturing groups,
and backreferences, with its own tests and no connection to the rest of the
program.

It cannot repeat a group, it cannot fail to compile, it matches bytes rather
than characters, and it can be made to run for half an hour by a fifty-byte
pattern. Three of those four are written down in the module comment or the
backlog. The one that was not is the one that corrupts data.

Slice 5b, next chapter, is the two hundred lines that connect it to the `s`
command.

---

**Exercise 9-1.** Make `Caps` hold `Vec<(usize, usize)>` behind a `&mut` and
add the undo logic that failed branches now need. Count the places that need
it, then put the original back.

**Exercise 9-2.** Write the implementation of the interim UTF-8 fix from
section 9.8: reject a match whose start or end is not on a character boundary.
`str::is_char_boundary` does the work. Then decide what the error message says.

**Exercise 9-3.** Add a step limit to the matcher: a counter threaded through
`match_here`, and a failure when it is exceeded. Pick the number by measuring,
then work out what `g/pattern/` should do when one line hits the limit.

**Exercise 9-4.** `s/\(ab/X/` substitutes rather than erroring. Decide whether
an unmatched `\(` should be an error, then find out what GNU ed does, then
write the test.

**Exercise 9-5.** Collapse `match_here`'s six parameters into a struct plus
three. Do it for real, run the tests, and then decide whether the call sites
read better. They might not.
