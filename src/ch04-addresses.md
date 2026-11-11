# 4. Addresses: A Little Language

Slice three is where the command line gets a grammar. Until now `p` printed the
current line and `,p` printed everything, and both were string literals in a
`match`. From here on, almost every command in ed accepts an address or a range
in front of it, the forms compose, and the program needs a parser.

```text
$ cargo run -q -p ch04-addresses -- -p '* '
* a
alpha
beta
gamma
delta
.
* 1p
alpha
* 
beta
* 2,4p
beta
gamma
delta
* $p
delta
* ,n
1	alpha
2	beta
3	gamma
4	delta
* q
$
```

The new file is `address.rs`, a hundred and ninety-four lines, and it is the
first piece of the program that is a language implementation rather than a
feature.

## 4.1 The grammar

ed's address syntax is small enough to state completely:

| Form | Means |
|---|---|
| `5` | line 5 |
| `.` | the current line |
| `$` | the last line |
| `+3`, `-2` | three lines after, two lines before, the current line |
| `+`, `-` | the same with an implied 1 |
| `2,4` | the range from line 2 through line 4 |
| `,` | shorthand for `1,$`, the whole buffer |
| `;` | shorthand for `.,$`, here to the end |
| `5,` | shorthand for `5,$` |

An address prefixes a command letter, so `2,4p` is a range and a print, `$d`
deletes the last line, and an address with no command letter at all means "go
there and print it", which is why `5` on its own is a usable way to move
around.

Two forms in real ed are missing from that table, and section 4.8 is about why.

## 4.2 Parsing and resolving are two jobs

The module splits the work in half, and the split is the reason the rest of it
stays small.

```rust,ignore
pub enum Address {
    Number(usize),
    Current,
    Last,
    Offset(isize),
}

pub struct Spec {
    pub start: Option<Address>,
    pub end: Option<Address>,
}

pub struct Range {
    pub start: usize, // 1-indexed, inclusive
    pub end: usize,   // 1-indexed, inclusive
}
```

`Spec` is what the user typed. `Range` is what it turns out to mean. `$` is an
`Address::Last` in a `Spec` and it is the number 4 in a `Range`, and which one
of those two it is depends on the buffer at the moment the command runs.

Keeping them apart matters because the same `Spec` resolves differently at
different times, and because parse errors and address errors are different
kinds of wrong. `2,4p` typed against a two-line buffer parses cleanly and
resolves to an error; `zp` fails to parse as an address and is a command
problem, not an address problem. A single function doing both would have to
report both through one channel and the caller would have to sort them out.

`Spec::parse` says so in its own doc comment: *this does NOT validate that the
addresses are in range — that's `resolve`'s job. It just tokenizes.*

One naming note, since the type will be read next to standard library code all
day. This `Range` is inclusive at both ends and 1-indexed, and `std::ops::Range`
is half-open and 0-indexed. They are opposite in both respects. The field
comments carry the convention, and the type is always written `address::Range`
at its use sites.

## 4.3 The dispatcher becomes two-stage

```rust,ignore
    let (spec, rest) = match Spec::parse(cmd) {
        Ok(parsed) => parsed,
        Err(e) => return Action::Error(e),
    };

    // No command letter, just an address. ed treats this as
    // "go to that line and print it" — so `5<Enter>` jumps to
    // line 5 and prints it.
    if rest.is_empty() {
        return run_print(&spec, buf, false);
    }

    // Stage one: commands that take no arguments after the letter.
    // Exact-match these so we don't have to deal with `q` matching
    // the front of `quit`.
    match rest {
        "q" | "quit" => return Action::Quit,
        "a" => return Action::EnterInputMode,
        "p" => return run_print(&spec, buf, false),
        "n" => return run_print(&spec, buf, true),
        _ => {}
    }

    // Stage two, when there is one: commands that take arguments
    // after the letter. Slice 4 adds the first of them.
    Action::Error(format!("unknown command: {rest}"))
```

Peel the address, then look at what is left. Every command from here to slice
12 is added to one of those two matches, and none of them parse addresses
themselves.

The comment on the first match is doing real work. The arms match the remainder
exactly, rather than testing its first character, because `q` and `quit` are
both commands and a first-character test makes `quit` a `q` with garbage after
it. Slice 8 adds long-form aliases for most of the command set, and exact
matching is what lets them be added as extra arms instead of as a prefix
problem.

The `rest.is_empty()` branch above it is ed's bare-address rule, and it is what
finally makes chapter 3's bare-Enter branch reachable. `1p` moves the current
line to the top; Enter walks down from there.

## 4.4 Absence is not an error

```rust,ignore
/// Parse a single address atom. Returns `(maybe_address, rest)`.
/// `Ok((None, rest))` means "no address here, the rest is the
/// command letter" — used by callers that need to distinguish an
/// empty address from a parse error.
fn parse_one(input: &str) -> Result<(Option<Address>, &str), String> {
    let bytes = input.as_bytes();
    let Some(&first) = bytes.first() else {
        return Ok((None, input));
    };
    match first {
        b'.' => Ok((Some(Address::Current), &input[1..])),
        b'$' => Ok((Some(Address::Last), &input[1..])),
        // ... offsets and digits ...
        _ => Ok((None, input)),
    }
}
```

The return type is `Result<Option<Address>, _>` flattened into a tuple, and the
two layers mean different things. `Err` is "that was meant to be an address and
it is malformed". `Ok(None)` is "there was no address here, and that is fine".

The final arm is the one that makes the dispatcher work. When the input
is `pq` or `zp` or anything else that does not start with an address character,
`parse_one` consumes nothing, returns `None`, and hands the string back
untouched. The command matcher below then gets the full string and reports
`unknown command: zp`. Had that arm been an error instead, every unrecognized
command in the program would have been reported as a bad address, which is both
wrong and the kind of wrong that sends a user looking in the wrong place.

The rule generalizes past this parser: a component that can be handed input
meant for someone else needs a way to decline that is not failure.

## 4.5 The parser hands back the rest of the string

```rust,ignore
pub fn parse(input: &str) -> Result<(Spec, &str), String>
```

The returned `&str` borrows from `input`, which the elided lifetime ties
together, and nothing is allocated to represent "the remainder". The tokenizer
walks forward by slicing, `take_digits` finds the end of a digit run with
`position` and splits there, and the whole module allocates only when it builds
an error message.

```rust,ignore
/// Pull a run of ASCII digits off the front of `s`. Returns the
/// digit slice and the remainder.
fn take_digits(s: &str) -> (&str, &str) {
    let end = s
        .as_bytes()
        .iter()
        .position(|b| !b.is_ascii_digit())
        .unwrap_or(s.len());
    s.split_at(end)
}
```

This is the shape most hand-written parsers in Rust end up with, and it is
pleasant for a reason that has nothing to do with performance: a function
returning "what I understood, and where I stopped" composes with the next one
without anybody owning a cursor. The alternative, a struct holding the input
and a position, is what you reach for when the grammar gets big enough that
threading the remainder through return types becomes noise. ed's grammar never
gets there, and chapter 8 is where the regex compiler does and takes the other
option.

Working in bytes rather than characters is safe here because every character
the grammar recognizes is ASCII, and the byte after a multi-byte character is
never mistaken for one of them. Chapter 13 is where that assumption gets
examined properly, because the *buffer* is a different question from the
*command language*.

## 4.6 Every command brings its own default

An address can be omitted, and what it defaults to is the command's business,
not the parser's.

```rust,ignore
    /// Resolve with current-line as the default for an omitted
    /// spec. Used by `p`, `n`, and most other commands.
    pub fn resolve(&self, buf: &Buffer) -> Result<Range, String> {
        self.resolve_with(buf, (Address::Current, Address::Current))
    }
```

Bare `p` prints the current line, so `p`'s default is `.,.`. By the time the
program ships there are two more of these, and they are the reason the default
does not live inside `resolve`:

```rust,ignore
    /// Resolve with the whole buffer (`1,$`) as the default for an
    /// omitted spec. Used by `w` — typing `w foo.txt` with no
    /// address writes the entire buffer, not just the current line.
    pub fn resolve_or_whole(&self, buf: &Buffer) -> Result<Range, String> {
        self.resolve_with(buf, (Address::Number(1), Address::Last))
    }

    /// Resolve with `.,.+1` as the default for an omitted spec.
    /// Used by `j` (join) — bare `j` joins the current line with
    /// the next one, matching ed.
    pub fn resolve_or_pair_with_next(&self, buf: &Buffer) -> Result<Range, String> {
        self.resolve_with(buf, (Address::Current, Address::Offset(1)))
    }
```

A bare `w` that wrote only the current line would be a data-loss bug wearing
the costume of a sensible default. A bare `j` that joined the current line with
nothing would do nothing. Neither default is derivable from the address syntax,
because neither is a fact about addresses. `resolve_with` takes the pair as an
argument and each command picks.

This is worth recognizing as a shape, because the wrong version of it is
common: a parser that helpfully fills in a default, and then one caller that
needs a different one, and then a flag parameter, and then two. The default
belongs to whoever knows the semantics.

## 4.7 A state the type allows and the parser never builds

```rust,ignore
        let (start_addr, end_addr) = match (&self.start, &self.end) {
            (None, None) => default,
            (Some(s), None) => (clone_addr(s), clone_addr(s)),
            (Some(s), Some(e)) => (clone_addr(s), clone_addr(e)),
            (None, Some(_)) => unreachable!("parser never produces (None, Some)"),
        };
```

`Spec` holds two `Option<Address>`, which is four combinations, and the parser
produces three of them. A spec with an end and no start is meaningless, cannot
be typed, and is representable.

The `unreachable!` is an honest way to write this and not the best one. It
states the invariant, it names the component responsible for it, and it turns a
violation into a panic with a message rather than a silently wrong answer. What
it does not do is prevent the state, and the compiler is not checking the claim
— it is taking the author's word and generating a panic for the case.

The alternative is to make the state unrepresentable:

```rust,ignore
enum Spec {
    Empty,
    Single(Address),
    Range(Address, Address),
}
```

Three variants, three cases, no fourth arm and no panic. *The Rust Thought
Process* spends a chapter on this move, and it is the right one often enough
that reaching for it should be automatic.

ved did not make it, and the reason is worth being straight about rather than
dressing up. `Spec`'s fields are read directly by `parse_dest` in the shipped
code, which checks `spec.end.is_some()` to reject a range where a single
address is required, and by `is_empty`, which checks that both are `None`. Both
would work against the enum. The structure stayed because it was written in
slice 3 and never caused a bug, which is the same reason most such structures
stay. The panic has never fired.

That is a real answer and it is not a defence. An invariant maintained by one
function's control flow is one refactor away from being maintained by nobody,
and the compiler will not mention it.

## 4.8 Two things this code gets wrong

The first is small:

```rust,ignore
fn clone_addr(a: &Address) -> Address {
    match a {
        Address::Number(n) => Address::Number(*n),
        Address::Current => Address::Current,
        Address::Last => Address::Last,
        Address::Offset(n) => Address::Offset(*n),
    }
}
```

Twelve lines that `#[derive(Clone, Copy)]` on `Address` deletes. Every variant
carries either nothing or a plain integer, so the derive is available, correct,
and free. This function is in the shipped program.

The second is not small, and it is a comment:

```rust,ignore
// Search addresses (/foo/, ?foo?) wait for slice 5 when the regex
// engine arrives. Compound expressions like $-5 also wait.
```

Slice 5 built the regex engine. Neither of those arrived with it, and neither
has arrived since. In the released program:

```text
$ ved
a
alpha
beta
.
/beta/p
? unknown command: /beta/p
$-1p
? unknown command: -1p
```

GNU ed accepts both. The offsets that ved does have (`+3`, `-2`) always resolve
against the current line, because `Address::Offset` holds a number and nothing
to attach it to, so an offset cannot compose onto a preceding address.

Chapter 1 made a distinction between a repository documenting what is *true*
and a decision log recording what *changed*, and this comment is the third
thing, which is a promise. It describes a future that did not happen. It has
been wrong for longer than it was ever right, and it reads, to anyone opening
the file, as a statement about work in progress rather than a permanent
limitation of the program.

The fix is not to delete the knowledge. It is to move it to where the program's
limitations are recorded and to state it in the present tense: *addresses are
line numbers, `.`, `$`, and current-relative offsets; regex search addresses
and compound offsets are not implemented*. Then the comment is true, and stays
true, and the backlog carries the intention. Chapter 19 is about the file where
that intention lives, and search addresses sit at the top of it.

## 4.9 What slice three bought

Two hundred and thirty lines of `main.rs` and a hundred and ninety-four of
`address.rs`, and the program can now name any line or span of lines in the
buffer. More importantly, the *next* eight commands do not have to think about
it. `d`, `w`, `s`, `g`, `m`, `t`, `j` and `c` all arrive as an arm in a match
and a call to `resolve`.

That is the return on building a small language early rather than adding
addressing to each command as it needs it. The alternative pattern — the one
where `p` learns about `$` and then `d` learns about `$` separately — is how
command-line programs end up with three subtly different notions of what `-1`
means.

---

**Exercise 4-1.** Add `#[derive(Clone, Copy)]` to `Address` and delete
`clone_addr`. Then find every place the borrow checker now lets you simplify,
and check whether `resolve_with` still needs to take its default by value.

**Exercise 4-2.** Implement `$-5`. The honest version restructures `Address` so
that an offset composes onto a base address rather than always onto the current
line; sketch the type before you write the parser, and check what it does to
`resolve_one`.

**Exercise 4-3.** `5,p` means `5,$p`. Find the three lines in `Spec::parse` that
implement it, then decide whether `,5p` should mean `1,5p` and whether ed
agrees with you.

**Exercise 4-4.** Replace `Spec`'s two `Option`s with the three-variant enum
from section 4.7 and delete the `unreachable!`. Count the lines that changed.
If it was fewer than twenty, ask why it was not done.

**Exercise 4-5.** `zp` reports `unknown command: zp` rather than a bad address.
Change `parse_one`'s final arm to return an error and run the test suite for
chapter 4's crate, then explain the failure to someone in one sentence.
