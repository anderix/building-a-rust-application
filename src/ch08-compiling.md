# 8. A Regex Engine: Compiling the Pattern

The matcher in chapter 7 treats the pattern as a program and walks it with
recursion. That works for literals, `.`, `*`, `^`, and `$`, all of which are
one byte. It stops working at the next feature in BRE.

`[a-z]` is one matchable thing spelled with five bytes. So is `[^abc]`, and so
is `[]]`. The recursive matcher looks at `pattern[0]` and peeks at
`pattern[1]`, and neither tells it anything useful about a bracket expression.
Handling them in that shape means scanning forward for the closing `]` on every
match attempt, at every position in the text, and then doing it again to find
out whether a `*` follows.

The fix is the one Ken Thompson used in the original ed: turn the pattern into
a data structure once, and match against that.

This chapter is the compile step and bracket expressions. The crate is
`examples/ch08-bre`, which contains the end state of slice 5a; the parts of it
dealing with groups and backreferences are chapter 9.

## 8.1 Two types, and the difference between them

```rust,ignore
/// One matchable thing: a literal byte, any-byte wildcard, or
/// a bracket character class.
enum Atom {
    Literal(u8),
    Dot,
    Class { negated: bool, chars: Vec<u8> },
}

/// One element of a compiled pattern.
enum Element {
    One(Atom),
    Star(Atom),
    Caret,
    Dollar,
    GroupStart(usize),
    GroupEnd(usize),
    BackRef(usize),
}
```

An `Atom` answers one question: does this byte match? An `Element` is an
instruction in the compiled pattern, and most instructions are about an atom
rather than being one.

The split is what the design turns on, so it is worth stating plainly.
`Dot` is an `Atom` because `.` matches a byte. `Dollar` is an `Element`
because `$` matches a *position* and consumes nothing. `Star` is an `Element`
because repetition is a thing you do to an atom, not a thing that matches.
Putting them in one enum would work and would mean every function that handles
atoms has arms for cases that cannot occur there.

`Class` carries its characters already expanded: `[a-z]` compiles to a `Vec<u8>`
of twenty-six bytes. Storing the ranges and testing membership arithmetically
would use less memory and more code. For patterns typed by a person at an
editor the difference is invisible, and the expanded form makes
`atom_matches` a three-line function:

```rust,ignore
fn atom_matches(atom: &Atom, byte: u8) -> bool {
    match atom {
        Atom::Literal(c) => byte == *c,
        Atom::Dot => true,
        Atom::Class { negated, chars } => {
            let found = chars.contains(&byte);
            if *negated { !found } else { found }
        }
    }
}
```

`contains` on a `Vec<u8>` is a linear scan, which for a class of twenty-six is
faster than anything cleverer. There are only 256 distinct bytes to name, so a
class written by a person stays well inside the range where the scan wins.

## 8.2 The compiler is a loop

```rust,ignore
    pub fn compile(pattern: &[u8]) -> Regex {
        let mut elements = Vec::new();
        let mut i = 0;

        // ^ only anchors at the very start of the pattern.
        if i < pattern.len() && pattern[i] == b'^' {
            elements.push(Element::Caret);
            i += 1;
        }

        while i < pattern.len() {
            // $ only anchors at the very end of the pattern.
            if pattern[i] == b'$' && i + 1 == pattern.len() {
                elements.push(Element::Dollar);
                i += 1;
                continue;
            }
            // ... backslash sequences ...

            // Parse one atom.
            let (atom, next) = parse_atom(pattern, i);
            i = next;

            // If followed by *, wrap in Star; otherwise One.
            if i < pattern.len() && pattern[i] == b'*' {
                elements.push(Element::Star(atom));
                i += 1;
            } else {
                elements.push(Element::One(atom));
            }
        }

        Regex { elements }
    }
```

A cursor, a vector, and a loop. No recursion, no grammar, no parser generator,
because BRE without alternation is a flat sequence — there is nothing to nest
except groups, which are markers rather than structure.

The anchors moved out of the matcher and into the compiler, and this is the
improvement that is easy to miss. Chapter 7's `match_here` had to ask, on every
call, whether `pattern[0] == b'$' && pattern.len() == 1`. Here the question is
asked once per pattern, at compile time, and the answer is recorded as an
`Element::Dollar` that means "anchor" and nothing else. A `$` in the middle of
the pattern never becomes a `Dollar` at all; it falls through to `parse_atom`
and becomes `Literal(b'$')`. The specification rule from chapter 7 is now
enforced by the shape of the compiled output rather than re-derived at every
step.

The `*` handling has the same character. Chapter 7 looked ahead for `*` during
matching. Here the lookahead happens once, and `Star(atom)` versus `One(atom)`
is decided before any text is seen.

## 8.3 Bracket expressions, and POSIX's awkward corners

```rust,ignore
/// Follows POSIX BRE bracket rules:
///   - `]` as the first character (after optional `^`) is literal
///   - `-` at the start or end is literal, not a range
///   - `a-z` in the middle is a range, expanded inline
///   - `[^...]` negates the class
fn parse_bracket(pattern: &[u8], start: usize) -> (Atom, usize) {
```

Those four rules exist because bracket expressions have no escape character.
Inside `[...]`, a backslash is a literal backslash, so `]` and `-` cannot be
escaped and the specification instead gives them positions where they lose
their meaning. `[]]` is a class containing `]`. `[a-]` is a class containing
`a` and `-`. `[]-]` is both.

The implementation follows the rules positionally, which is the only way:

```rust,ignore
    // ] as the very first character (after optional ^) is literal.
    if i < pattern.len() && pattern[i] == b']' {
        chars.push(b']');
        i += 1;
    }

    while i < pattern.len() && pattern[i] != b']' {
        // Range: lo-hi, but only when - is followed by a char
        // that isn't the closing ].
        if i + 2 < pattern.len()
            && pattern[i + 1] == b'-'
            && pattern[i + 2] != b']'
        {
            let lo = pattern[i];
            let hi = pattern[i + 2];
            for c in lo..=hi {
                chars.push(c);
            }
            i += 3;
        } else {
            chars.push(pattern[i]);
            i += 1;
        }
    }
```

This is the part of the engine where having a written specification pays for
itself twice over. Nobody's intuition produces "a `]` immediately after the
opening bracket is literal", and nobody would defend it as a design. It is
there because ed did it in 1969 and every tool since has copied it, and a
clone that guessed differently would break scripts in a way that looks like a
bug in the script.

`for c in lo..=hi` expands the range with no check that `lo <= hi`. A reversed
range like `[z-a]` produces an empty iterator and therefore an empty class,
which matches nothing. POSIX says the behavior is undefined. Producing an empty
class is a defensible reading of undefined, and it is not what GNU ed does: ed
rejects the command and, with `H` set, says `Invalid range end`.

## 8.4 A pattern that cannot fail to compile

```rust,ignore
pub fn compile(pattern: &[u8]) -> Regex
```

There is no `Result`. Every byte string is a valid pattern, and this is a
decision rather than an oversight — it is the same signature in the released
program.

What it means in practice:

```text
$ ved
a
xaybzc
.
s/[abc/X/g
,p
xXyXzX
```

`[abc` has no closing bracket. The loop in `parse_bracket` runs to the end of
the pattern, the "skip the closing `]`" step finds nothing to skip, and the
result is a class containing `a`, `b`, and `c`. The substitution then does
something entirely reasonable and not what POSIX says, which is that an
unmatched `[` is an error.

The appeal of a non-failing compiler is real. There is no error type, no
`Result` threaded from `compile` through `run_substitute` to `Action::Error`,
and no question about what to do with a half-compiled pattern. The cost is that
a typo becomes a silent behavior change. A user who types `s/[abc/X/` meant
something, and what they get is neither an error nor what they meant.

The version with errors is not hard — `compile` returns `Result<Regex, String>`,
`parse_bracket` reports an unterminated class, and the two call sites already
have an `Action::Error` to return it through. It was not done, and the reason
is the ordinary one: the first version did not need it, and nothing since has
forced the question. Chapter 19 is where the backlog records that kind of debt
honestly, and this is one of the entries.

## 8.5 From a bool to a position

```rust,ignore
pub struct Regex {
    elements: Vec<Element>,
}

/// The result of a successful match: byte offsets into the text,
/// plus any captured groups.
pub struct Match {
    pub start: usize,
    pub end: usize,
    caps: Caps,
}

pub fn find(&self, text: &[u8]) -> Option<Match>
```

Chapter 7's engine answered whether a pattern matched. That is enough for `g`
and `v`, which select lines, and useless for `s`, which has to replace the
matched span and therefore has to know where it is.

`Option<Match>` rather than `Match` with a sentinel, and `start`/`end` as byte
offsets into the text rather than a borrowed slice. The offsets are the choice
worth noticing: returning `&[u8]` would tie the `Match` to the lifetime of the
text, and `run_substitute` wants to build a new line out of the pieces before
and after the match, which means it wants indices.

The slide moved into `find`, where it can be written once:

```rust,ignore
        let anchored = !elements.is_empty()
            && matches!(elements[0], Element::Caret);
        let elems = if anchored { &elements[1..] } else { elements };

        let mut offset = 0;
        loop {
            let caps = Caps::new();
            if let Some((len, caps)) =
                self.match_here(elems, &text[offset..], offset, caps, text)
            {
                return Some(Match { start: offset, end: offset + len, caps });
            }
            if anchored || offset >= text.len() {
                return None;
            }
            offset += 1;
        }
```

The anchor check is the same trick as chapter 7 — strip the `Caret` and refuse
to slide — expressed against the compiled form. The `offset >= text.len()`
test after the match attempt preserves the empty-tail behavior that `$` depends
on, and it is the same ordering, now with a comment's worth of distance between
the two versions.

`match_here` returns `Option<(usize, Caps)>`: how many bytes were consumed, and
the capture state. The length has to come back because the caller adds it to
`offset` to get `end`, and because `match_star` needs it to account for what a
repetition consumed. Chapter 7 could return `bool` because nobody downstream
cared how much was eaten.

## 8.6 A defensive arm, and a comparison

```rust,ignore
            Element::Caret => None,
```

`compile` emits `Element::Caret` only at position 0, and `find` strips it
before calling `match_here`. A `Caret` can therefore never reach this arm.

Chapter 4 had the same situation — a state the type permits and the code never
produces — and answered it with `unreachable!("parser never produces (None,
Some)")`. Here the answer is `None`, which is to say: if this impossible thing
happens, the match fails.

The second answer is better, and the difference is what happens when the
assumption stops holding. A future change that emits a `Caret` in the middle of
a pattern turns the `unreachable!` into a crash for the user and the `None`
into a pattern that does not match. Both are wrong. One of them loses the
buffer.

The rule that falls out is narrow enough to apply: when an unreachable case has
a sensible failure value, return it. Keep `unreachable!` for the cases where
there is no such value and continuing would corrupt something.

## 8.7 The limitation visible in the types

```rust,ignore
enum Element {
    One(Atom),
    Star(Atom),
```

`Star` holds an `Atom`, so `*` can repeat one matchable thing. It cannot repeat
a group:

```text
$ ved
a
ababX
.
s/\(ab\)*/Y/
? no match
```

The module comment says so:

> Known limitation: `\(...\)*` (star applied to a whole group) is not yet
> supported. Star can only apply to a single atom. This is uncommon in ed usage
> and can be added later if needed.

What makes this a good limitation to have is that it is not a bug hiding in a
branch. It is the shape of the type. `Star(Atom)` says what it can repeat, and
anyone reading the enum knows the answer before reading a line of matching
code. The fix is equally visible: `Star` would have to hold something that can
represent a sequence, which means either a boxed sub-pattern or a pair of
indices into `elements`, and the matcher would have to try repetitions of a
compiled span rather than of a single byte test.

Compare that to the star greediness from chapter 7, which was also a
limitation, also documented in a doc comment, and completely invisible in the
types. One of the two was found by reading the code. The other was found by a
user.

## 8.8 What the compile step bought

Bracket expressions, which is what it was for. Escaped specials, because the
compiler has a place to turn `\.` into `Literal(b'.')` before any matching
starts. Positions instead of a boolean, which is what makes `s` possible in the
next chapter but one. And a representation that groups and backreferences can
be added to as two more `Element` variants rather than as another lookahead in
the matcher — which is chapter 9.

It cost the property that made chapter 7's version worth showing: you can no
longer read the engine in one sitting. The file went from 221 lines to
823, and the part a reader has to hold in their head went from three functions
to three functions plus two enums, a compiler, and the invariant that connects
them.

---

**Exercise 8-1.** Make `compile` return `Result<Regex, String>`, reporting an
unterminated bracket expression. Follow the error to `run_substitute` and
decide what the message should say. Count the lines it cost.

**Exercise 8-2.** `[z-a]` compiles to a class that matches nothing. Decide what
it should do, then find out what GNU ed does, then decide again.

**Exercise 8-3.** Write the three-line test that proves `[]]` is a class
containing `]`, and the one that proves `[a-]` contains a hyphen. Then break
`parse_bracket` in the smallest way that leaves both passing.

**Exercise 8-4.** Implement `\(...\)*`. Start by writing the `Element` variant,
and stop when you can state what `match_star` would have to become — the rest
is a weekend, and the design decision is the part worth doing.

**Exercise 8-5.** `Class` stores expanded bytes. Replace it with a list of
ranges plus a list of singletons, and measure the compile time and match time
for `[a-z]` and for `[0-9a-fA-F]`. Report both numbers, then decide whether the
original was wrong.
