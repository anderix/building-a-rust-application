# 7. A Regex Engine: The Recursive Matcher

ed's `s` command takes a regular expression, and so do `g` and `v`. The
standard ed dialect is POSIX Basic Regular Expressions, which the `regex` crate
does not implement, and this program has no dependencies anyway. So the engine
gets written.

It takes three chapters and ends at a thousand lines, which is a third of the
finished program. This one covers the first forty-three.

## 7.1 A slice that adds no capability

Chapter 1 stated the rule that makes slicing work: *each slice is a complete
working program with one more capability than the previous*. Slice 5 breaks it,
and the plan says so by splitting it in two. 5a is the engine. 5b is the `s`
command that uses it.

At the end of 5a the program does what it did at the end of slice 4.
Running the two crates side by side produces identical output, because the new
module is not connected to anything:

```rust,ignore
mod address;
#[allow(dead_code)] // unused until slice 5b wires it to the substitute command
mod bre;
mod buffer;
```

That attribute is the honest version of a compromise. Without it the build is
noisy with warnings about unused functions, and a warning that is always
present is a warning nobody reads. With it, the lint is suppressed for one
module, and the comment says what will make the suppression unnecessary.

The rule to take from it is about lint suppressions generally: one that names
its expiry is a note, and one that does not is a decision nobody will revisit.
`#[allow(dead_code)]` with no comment on a module is indistinguishable from
`#[allow(dead_code)]` on code that is genuinely dead.

The deeper question is whether splitting the slice was right, given that the
point of a slice is to leave something that runs. The alternative was one slice
containing both engine and command, finished in one stretch, with nothing
working in the middle. What the split buys is that the engine gets built
against its own tests rather than against `s`, which matters more here than
usual: a regex bug reached through a substitute command presents as "the wrong
text changed", and the same bug reached through `match_any(b"a*b", b"xb")`
presents as itself.

The rule survives, with a footnote. A slice that produces no capability is
acceptable when it produces a *component with its own tests*, and it is the
only case where it is acceptable.

## 7.2 The matcher being translated

The starting point is the recursive matcher from Kernighan and Pike's *The
Practice of Programming*: roughly thirty lines of C that handle literal
characters, `.`, `*`, `^`, and `$`. It is famous for the ratio — a usable
subset of regular expressions in a page of code that can be read in one sitting
— and it works by treating the pattern as a program and recursing down it.

Three functions, mutually recursive:

```rust,ignore
pub fn match_any(pattern: &[u8], mut text: &[u8]) -> bool
fn match_here(pattern: &[u8], text: &[u8]) -> bool
fn match_star(c: u8, pattern: &[u8], mut text: &[u8]) -> bool
```

`match_any` finds a match anywhere. `match_here` requires one at the start.
`match_star` handles a repeated element. That is the design, and the next
three sections are the three functions.

## 7.3 Three decisions in the translation

The module comment records them, which is the right place for them: they are
facts about this code that a reader will otherwise rediscover.

```rust,ignore
//   * Pike's C uses `char *` and advances pointers with `regexp+1`.
//     We use `&[u8]` slices and `&pattern[1..]` — mechanically the
//     same, safer at the language level.
//   * Pike reads past the end of the string freely because C strings
//     are null-terminated: `regexp[1]` on a one-char pattern reads
//     the '\0'. Rust slices panic on out-of-bounds access, so we
//     guard each lookahead with a length check.
//   * Pike's function is named `match`, which is a Rust keyword.
//     Ours is `match_any`.
```

The middle one is the interesting one, and it is the general shape of porting C
to Rust. The C original is not sloppy; it is correct *because* of an invariant
the language provides, namely that there is always a readable byte at the end
of a string. Rust provides a different invariant, that a slice knows its own
length and indexing past it is a panic rather than a read. The translation has
to add explicitly what the source language supplied implicitly, which is why
`match_here` has a `pattern.len() >= 2` that the C has no equivalent of.

Ports go wrong when this is treated as noise. Every one of those length checks
corresponds to a place the original depended on the terminator, and dropping
one produces a panic on a pattern like `a*` where the lookahead runs off the
end.

Working in `&[u8]` rather than `&str` is the third decision, unstated because
it follows from the first. It also means `.` matches a byte rather than a
character, which is correct for ASCII and wrong for anything else; chapter 9
comes back to it.

## 7.4 `match_any`: slide along the text

```rust,ignore
pub fn match_any(pattern: &[u8], mut text: &[u8]) -> bool {
    if !pattern.is_empty() && pattern[0] == b'^' {
        return match_here(&pattern[1..], text);
    }
    // Try each starting position, INCLUDING the empty tail, so that
    // `$` against `""` still gets a chance to match.
    loop {
        if match_here(pattern, text) {
            return true;
        }
        if text.is_empty() {
            return false;
        }
        text = &text[1..];
    }
}
```

An unanchored pattern matches anywhere, which is implemented by trying `here`
at every position. Two details carry their weight.

**`^` is handled once, at the top, and then removed from the pattern.** It is
not a case inside `match_here`, because `^` does not mean "start of text"
recursively — it means "do not slide". Having stripped it, the rest of the
engine never sees it. A `^` in the middle of a BRE is a literal caret, and this
structure gives that for free rather than as a special case.

**The empty tail is a position.** The order of the two checks inside the loop
is the point: `match_here` runs before the `is_empty` test, so when
`text` has been consumed down to nothing there is still one attempt. Without
that, `$` against any text would fail, because `$` only matches when the
remaining text is empty and the loop would have exited first. There is a test
named `dollar_alone_finds_end` whose only job is to hold that ordering in
place.

## 7.5 `match_here`: peel one element

```rust,ignore
fn match_here(pattern: &[u8], text: &[u8]) -> bool {
    if pattern.is_empty() {
        return true;
    }
    // Lookahead: if the next element is `*`, hand off to match_star.
    if pattern.len() >= 2 && pattern[1] == b'*' {
        return match_star(pattern[0], &pattern[2..], text);
    }
    // `$` only anchors when it's the last character of the pattern.
    // Everywhere else it's a literal, which is what BRE actually
    // specifies.
    if pattern[0] == b'$' && pattern.len() == 1 {
        return text.is_empty();
    }
    // Otherwise match one character (literal or `.`) and recurse.
    if !text.is_empty() && (pattern[0] == b'.' || pattern[0] == text[0]) {
        return match_here(&pattern[1..], &text[1..]);
    }
    false
}
```

Four branches, in an order that is not arbitrary.

The empty pattern returns true, because a pattern with nothing left has matched
everything it asked for. This is the base case of the recursion and the reason
`match_any(b"", anything)` is true.

The `*` lookahead comes second, before anything else looks at `pattern[0]`,
because `a*` is one element and not two. A design that peeled `a` first and
then discovered `*` would have to un-consume it.

The `$` rule is a specification detail, and getting it right is a small
argument for cloning something with a written standard. In POSIX BRE, `$` is an
anchor only as the final character of the expression; `a$b` matches a literal
dollar sign. Most people's intuition, informed by Perl-style regex, is that `$`
is always an anchor. The comment states the rule so the next reader does not
"fix" it.

The last branch is the one people picture when they picture a regex engine, and
it is three lines: if there is text and the pattern character matches it, drop
one from each and recurse.

## 7.6 `match_star`, and the bug it shipped with

```rust,ignore
/// Search for `c*pattern` at the beginning of `text`. Tries zero
/// matches of `c` first, then one, then two, and so on, until
/// either the tail pattern matches or `c` stops consuming.
fn match_star(c: u8, pattern: &[u8], mut text: &[u8]) -> bool {
    loop {
        if match_here(pattern, text) {
            return true;
        }
        if text.is_empty() {
            return false;
        }
        if text[0] != c && c != b'.' {
            return false;
        }
        text = &text[1..];
    }
}
```

Zero repetitions first, then one, then two, stopping at the first success. The
doc comment describes the behavior accurately, the twenty-three tests pass, and
for answering "does this match?" the function is correct: a pattern that
matches with three repetitions also matches with the shortest number that
works, and `match_any` returns a `bool`.

It is the wrong answer as soon as the caller wants to know *what* matched.

POSIX specifies leftmost-longest: of all matches starting at the earliest
position, a BRE matches the longest one. This function finds the shortest.
While the engine only returns `true` or `false` the difference is invisible,
which is why it survived this chapter and the next two. Once it is
wired to `s`, it is visible on the first try:

```text
$ cargo run -q -p ch10-substitute
a
aaa
.
s/a*/X/
,p
Xaaa
```

`a*` matched zero characters at position 0, so `X` was inserted in front and
the three a's remain. POSIX requires `X`. The released program produces `X`;
this was fixed in v0.1.3, well after ed scripts had been run against it.

Two things are worth taking from that, and neither is "check your star".

The first is about borrowed code. The matcher was copied from a famous, correct
program, and it was copied correctly. What did not come with it was the
question of whether the specification the *new* program is implementing has a
requirement the original never claimed to meet. The original answers "does this
match", which is a complete and honest answer to the question it was written
for.

The second is about where the bug was visible. Not in `match_star`, where the
code is four lines and matches its own comment. Not in the tests, which all
pass. It appears in the output of a command two slices away, and the path back
from `Xaaa` to a loop that tries zero first is the path this chapter's
split of slice 5 was meant to shorten. It did not shorten it, because the bug
is not in the layer the tests covered — it is in the agreement between the
layer and the standard.

Chapter 15 has a name for this and a way to test for it.

## 7.7 Twenty-three tests for forty-three lines

```rust,ignore
    #[test]
    fn star_combined_with_slide() {
        // Exercises both the outer slide and match_star's zero case:
        // at position 0, `a*b` fails because `xb` starts with x.
        // We slide to position 1 and match `b` with zero a's.
        assert!(match_any(b"a*b", b"xb"));
    }
```

That is one test for every two lines of code, which is high, and the right
number for this module and the wrong number for most others. What makes
it right is that this is a pure function over two byte strings with no I/O, no
state, and a specification someone else wrote. Every test is one line of setup
and one assertion, they run in microseconds, and the input space is small
enough to cover the interesting corners by hand.

The names are the documentation: `caret_refuses_to_slide`,
`dollar_anchors_at_end`, `empty_anchored_matches_empty`,
`anchored_star_rejects_wrong_char`. A failure prints the name, and the name is
the sentence you would have written in a bug report.

The comment in the test above is the other half. It explains which code path
the case is reaching, which is what a reader needs when the test fails two
years later and the obvious reading of the assertion does not suggest why it
ever mattered.

What the twenty-three do not do is compare against another implementation. They
encode what the author believed POSIX required, which is why they are all green
in a chapter that ends with a shipped deviation from POSIX.

## 7.8 What slice 5a bought

Forty-three lines of code, in three mutually recursive functions, answering
whether a byte string matches a pattern containing literals, `.`, `*`, `^`, and
`$`. They sit in a 221-line file, because the other hundred and sixty are tests
and the comment explaining the translation, and they are connected to nothing.

That is a fraction of BRE. Bracket expressions, escaped specials, groups, and
backreferences are all missing, and the structure that makes this version
readable — the pattern is the program, walked by recursion — is the structure
that has to be replaced to get them. Chapter 8 replaces it.

---

**Exercise 7-1.** Make `match_star` greedy: consume as many repetitions as
possible, then back off one at a time until the tail matches. Check
`^a*$` against `aaa`, `a*b` against `xb`, and `a*` against the empty string
before you believe it.

**Exercise 7-2.** Delete the `pattern.len() >= 2` guard in `match_here` and
find the pattern that panics. Then explain, in one sentence, what the C
original does instead.

**Exercise 7-3.** Add `+` (one or more) to the engine. It is not in BRE, so
decide first whether a clone should have it, then implement it in terms of the
functions that exist.

**Exercise 7-4.** `.` matches one byte. Write the test that demonstrates the
problem using a multi-byte character, then decide whether the fix belongs in
`match_here` or in whoever calls the engine.

**Exercise 7-5.** The twenty-three tests all pass against an engine that
deviates from POSIX. Write one test, in the shape they are written in, that
would have failed.
