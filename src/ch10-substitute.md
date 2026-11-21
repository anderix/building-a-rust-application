# 10. Substitute

Slice 5b connects the regex engine to a command. It is the slice the previous
three chapters were for, and it is where ed stops being a way to type text and
becomes a way to change it.

```text
$ cargo run -q -p ch10-substitute
a
the cat sat on the mat
.
s/at/og/
the cog sat on the mat
s/at/og/g
the cog sog on the mog
s|o|/|g
the c/g s/g /n the m/g
s/the/[&]/g
[the] c/g s/g /n [the] m/g
```

The code is about a hundred and sixty lines in `main.rs` and fifty in `bre.rs`,
and it has four parts: a scanner that splits `/old/new/g` into its fields, an
expander that turns `&` and `\1` into text, a loop that applies the pattern to a
line once or repeatedly, and the command itself, which applies that to a range.
The engine does not change at all. That was the point of building it in
isolation.

This chapter also found more bugs than any other in the book. Most of them are
in the scanner and in the loop behind the `g` flag, sections 10.2 and 10.4.

## 10.1 The command, as a grammar

```text
[range]s/pattern/replacement/[flags]
```

The first character after `s` is the delimiter, and it can be anything. `/` is
the convention, `|` is what people use when the pattern contains slashes, and
ed accepts `s.a.b.` as readily as either. The delimiter is the reason `s`
cannot have a long-form alias: `subst/a/b/` would make `u` the delimiter of a
command named `s`, and chapter 14 comes back to that.

```rust,ignore
fn run_substitute(spec: &Spec, buf: &mut Buffer, args: &str) -> Action {
    let args = args.as_bytes();

    if args.is_empty() {
        return Action::Error("no delimiter".to_string());
    }

    let delim = args[0];
    let args = &args[1..]; // skip delimiter

    let (pattern, rest) = match scan_delimited(args, delim) {
        Some(r) => r,
        None => return Action::Error("unterminated pattern".to_string()),
    };

    let (replacement, rest) = match scan_delimited(rest, delim) {
        Some(r) => r,
        None => return Action::Error("unterminated replacement".to_string()),
    };

    let global = rest.contains(&b'g');
```

The function converts to bytes on its first line and never goes back. That is
the decision chapter 5's panic made safe, since `args[1..]` on a `&[u8]` cannot
land inside a character the way the same slice on a `&str` can. A delimiter
that is itself a multi-byte character fails differently: only its first byte is
taken as the delimiter, and each field starts with the stray byte after it.
That is a wrong answer rather than a crash, and nobody has ever typed it.

Two things in that block are less careful than they look.

**The closing delimiter is required.** `s/a/b` with nothing after the `b` is
`? unterminated replacement` in ved and a working substitution in ed, where the
final delimiter is optional and the line's end closes the replacement. It is
the most common way to type `s` in practice, and it is a script that does not
run unchanged.

**The flags are a search, not a parse.** `rest.contains(&b'g')` asks whether a
`g` appears anywhere in what is left. It does not ask what else is there:

```text
s/a/b/gq
bbb
s/a/b/2
baa
```

ed answers the first with `? Invalid command suffix` and the second by
replacing the *second* match, `aba`, because a number is a flag that selects
the *n*th occurrence. ved accepts the garbage, ignores the count, and does
something plausible with both. Chapter 8 called this a non-failing compiler
when it happened to bracket expressions, and the cost is the same here: a typo
becomes a different command rather than an error, and the user who typed `2`
meant something by it.

The general rule is the one from chapter 2's argument parser, applied one level
down. Start strict. A suffix parser that rejects everything it does not know
can learn `p` and `2` later without breaking anybody. One that accepts
everything has already promised that `gq` works.

## 10.2 What a field is

```rust,ignore
/// Scan a delimiter-terminated field, handling backslash escapes.
/// Returns the field content (with escapes preserved for the BRE
/// engine) and the remainder after the closing delimiter.
fn scan_delimited(input: &[u8], delim: u8) -> Option<(Vec<u8>, &[u8])> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if input[i] == delim {
            return Some((result, &input[i + 1..]));
        }
        if input[i] == b'\\' && i + 1 < input.len() {
            if input[i + 1] == delim {
                // Escaped delimiter: include just the delimiter
                result.push(delim);
                i += 2;
            } else {
                // Other escape: pass through as-is for the BRE
                // engine or replacement expander to interpret.
                result.push(input[i]);
                result.push(input[i + 1]);
                i += 2;
            }
        } else {
            result.push(input[i]);
            i += 1;
        }
    }
    None // no closing delimiter found
}
```

The same function scans the pattern and the replacement, which is right,
because the rule for where a field ends is the same in both: at the first
delimiter not preceded by a backslash. The return shape is chapter 4's again,
"what I found and where I stopped", and the `None` is how `unterminated
pattern` gets its own message rather than being reported as something stranger
further along.

The comment says escapes are preserved for whoever reads the field next, and
that is true of every escape except one. An escaped delimiter is *unescaped*:
`\/` becomes `/`. That is the reason the function returns a `Vec<u8>` rather
than a slice of its input. It has to build a new string, because the field it
returns is not a substring of the field it was given.

It is also a bug, and the comment explains why without noticing. If every other
escape is kept for the next reader to interpret, then the next reader is the
one who knows what an escape means, and the scanner has just decided one on its
behalf. When the delimiter is `/` the decision is harmless, since `/` means
nothing to a BRE. When the delimiter is a character the engine treats as
special, the backslash that made it literal is gone before the engine sees it:

```text
$ ved
a
axb
.
s.a\.b.X.
X
```

`\.` was the user's way of saying "a literal dot that is not the end of the
field". The scanner turned it into `.`, which the engine reads as any
character, and `axb` matched. GNU ed says `? No match`. The same thing happens
to `s*x\*y*Z*`, which ved compiles as `x*y`, and to `\&` in a replacement
delimited by `&`, which ved expands into the matched text instead of a literal
ampersand.

The fix is to stop interpreting. The scanner's job is to find the end of a
field, and it can do that without changing a byte:

```rust,ignore
fn scan_delimited(input: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    let mut i = 0;
    while i < input.len() {
        if input[i] == delim {
            return Some((&input[..i], &input[i + 1..]));
        }
        if input[i] == b'\\' && i + 1 < input.len() {
            i += 2;
        } else {
            i += 1;
        }
    }
    None
}
```

Every escape now reaches the next stage intact. The pattern compiler already
handles `\X` for an arbitrary `X` as a literal `X`, which is what POSIX
requires, so `\.` is a dot and `\/` is a slash without either one being a
special case. The replacement expander needs one arm changed, from "push the
backslash" to "push the character after it", which also corrects a second
difference from ed: `s/b/\n/` on `abc` produced `a\nc` in ved and `anc` in ed.
With both changes, the scanner returns a borrowed slice and allocates nothing,
and the 125 tests in the crate still pass.

That is the shape of this bug in general and it is common in hand-written
parsers: a lexer that does a little interpretation "while it is there", for the
case its author was thinking about, and destroys information a later stage
needed for a case the author was not. The tell is in the comment. A function
that preserves escapes for someone else should preserve all of them.

## 10.3 `&` and `\1`

```rust,ignore
/// Expand a replacement string using match results.
///
/// Handles ed's replacement conventions:
///   - `&` is replaced with the entire matched text
///   - `\1`-`\9` are replaced with captured group contents
///   - `\&` is a literal ampersand
///   - `\\` is a literal backslash
///   - Everything else is literal
pub fn expand_replacement(replacement: &[u8], m: &Match, text: &[u8]) -> Vec<u8> {
```

The replacement is not a string. It is a template, and expanding it needs the
match: its span for `&`, its captures for `\1`. That is why this function lives
in `bre.rs` rather than next to `run_substitute`. `Match` keeps its captures
private and exposes them through `m.group(n)`, and the expander is the one
consumer outside the engine that needs them, so putting it inside the module
keeps `Caps` from becoming public for one caller.

It takes `text` as well as the match, for the reason chapter 9 gave for
`full_text`: a match is offsets, not characters, and offsets need the original
to be read against.

```text
a
2026-11-20
.
s/\([0-9]*\)-\([0-9]*\)-\([0-9]*\)/\3.\2.\1/
20.11.2026
```

A group that did not participate expands to nothing, which is what ed does and
what one of the seven tests that arrived with this function holds in place:

```rust,ignore
    #[test]
    fn replace_backref_uncaptured_vanishes() {
        let text = b"abc";
        let m = Regex::compile(b"^\\(abc\\)$").find(text).unwrap();
        // \2 was never captured, so it expands to nothing
        assert_eq!(expand_replacement(b"\\1-\\2", &m, text), b"abc-");
    }
```

A reference to a group that does not exist at all is a different case, and
POSIX leaves it undefined. `s/\(b\)/\2/` on `abc` is `a2c` in GNU ed, which
treats the `\2` as a literal digit, and `ac` in ved, which treats it as an
empty group. Neither reports the mistake, and a user who typed `\2` meaning
the first group gets no hint in either program.

## 10.4 Once, or everywhere

```rust,ignore
/// Replace the first match in `text`. Returns None if no match.
fn substitute_first(re: &bre::Regex, text: &[u8], replacement: &[u8]) -> Option<Vec<u8>> {
    let m = re.find(text)?;
    let mut result = Vec::new();
    result.extend_from_slice(&text[..m.start]);
    result.extend_from_slice(&bre::expand_replacement(replacement, &m, text));
    result.extend_from_slice(&text[m.end..]);
    Some(result)
}
```

Everything before the match, the expansion, everything after. The `?` on the
first line returns `None` from the function when there is no match, and that
`None` is how `run_substitute` later tells a line that changed from a line that
did not. The function is seven lines and has no bugs.

The `g` flag is the other function, and it is the harder problem by some
distance, because "replace every match" is not a well-defined instruction until
three questions are answered. Where does the next search start? What does `^`
mean when it is not at the start of the line? And what happens when the
pattern matches the empty string?

```rust,ignore
fn substitute_all(re: &bre::Regex, text: &[u8], replacement: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut pos = 0;
    let mut matched = false;

    while pos <= text.len() {
        match re.find(&text[pos..]) {
            Some(m) => {
                matched = true;
                let abs_start = pos + m.start;
                let abs_end = pos + m.end;
                result.extend_from_slice(&text[pos..abs_start]);
                result.extend_from_slice(&bre::expand_replacement(replacement, &m, &text[pos..]));
                // Advance past the match. If the match was empty,
                // advance by one byte to avoid an infinite loop.
                if abs_end == abs_start {
                    if pos < text.len() {
                        result.push(text[pos]);
                    }
                    pos = abs_start + 1;
                } else {
                    pos = abs_end;
                }
            }
            None => {
                result.extend_from_slice(&text[pos..]);
                break;
            }
        }
    }

    if matched { Some(result) } else { None }
}
```

The design is to search, replace, and search again in the slice that is left.
It is the obvious design and it answers all three questions wrongly, each in a
different way.

**`^` matches again.** `re.find(&text[pos..])` hands the engine a fresh slice,
and the engine has no way to know the slice is not a whole line. An anchored
pattern matches at the start of every slice it is given:

```text
a
aaa
.
s/^a/X/g
XXX
```

ed prints `Xaa`, because there is one start of line and the anchor can match it
once.

**An empty match away from `pos` copies the wrong byte.** The empty-match
branch pushes `text[pos]` to step over one character. That is correct when the
empty match is *at* `pos`, and every test of this code had one there. When
the empty match is further along, which is what `$` produces, `pos` is still
pointing at the start of the slice:

```text
a
abc
.
s/$/!/g
abc!a
```

`text[pos..abs_start]` copied `abc`, the expansion added `!`, and the step-over
pushed `text[0]`, which is `a`, to the end of the line. That is a substitution
that adds a character the user did not type and did not ask for, and it lasted
from this slice through the release of 0.2.1.

**An empty match next to a real one is counted.** `a*` matches the empty
string anywhere, including immediately after it has matched `aaa`:

```text
a
baaac
.
s/a*/X/g
XbXXcX
```

`sed` prints `XbXcX`. The POSIX rule, which `sed` follows, is that an empty
match adjacent to the previous match is not a new match. Without it, the loop
replaces `aaa` and then replaces the nothing that follows it. GNU ed sidesteps
the question and refuses this substitution outright, with `? Infinite
substitution loop`.

All three have one cause. The loop throws away context every time it slices:
the engine no longer knows where the line began, and the loop no longer knows
where the last match ended. The fix gives both back. The engine learns to
search from a position in the whole text rather than from the start of a piece
of it:

```rust,ignore
    pub fn find_at(&self, text: &[u8], start: usize) -> Option<Match> {
        let elements = &self.elements;
        let anchored = !elements.is_empty() && matches!(elements[0], Element::Caret);
        let elems = if anchored { &elements[1..] } else { elements };
        if anchored && start > 0 {
            return None;
        }

        let mut offset = start;
        // ... the rest of `find`, unchanged
    }
```

and `find` becomes `self.find_at(text, 0)`. The loop keeps two positions, where
it has copied up to and where it searches next, plus the end of the previous
match:

```rust,ignore
fn substitute_all(re: &bre::Regex, text: &[u8], replacement: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut copied = 0;
    let mut search = 0;
    let mut last_end = None;

    while search <= text.len() {
        let Some(m) = re.find_at(text, search) else {
            break;
        };
        if m.start == m.end && last_end == Some(m.start) {
            search = m.start + 1;
            continue;
        }
        result.extend_from_slice(&text[copied..m.start]);
        result.extend_from_slice(&bre::expand_replacement(replacement, &m, text));
        copied = m.end;
        last_end = Some(m.end);
        search = if m.start == m.end { m.end + 1 } else { m.end };
    }

    last_end?;
    result.extend_from_slice(&text[copied..]);
    Some(result)
}
```

Matches are now in the coordinates of the whole line, so the expander gets the
whole line and the relative-offset comment that the original needed is gone.
The step-over no longer pushes a byte at all: skipping a position changes
`search` and leaves `copied` alone, so the skipped character is copied with the
next stretch of unmatched text. And `matched`, a flag kept in step with the
loop, has become `last_end`, which the loop needed anyway. Built and run against
the cases above, this version prints `Xaa`, `abc!`, and `XbXcX`, agrees with
`sed` on `s/x*/-/g` (`-a-b-c-`) and `s/a*$/E/g` (`bE`), and passes the 125
existing tests, none of which had exercised any of the three.

Version 0.3.0 rewrote the engine to work in characters, for chapter 9's reasons,
and the `$` corruption went away with that rewrite, because the new loop pushes
the character at the match rather than the one at `pos`. The anchor and the
adjacent empty match are still there. Chapter 19 picks up the list.

## 10.5 Applying it to a range

```rust,ignore
    let range = match spec.resolve(buf) {
        Ok(r) => r,
        Err(e) => return Action::Error(e),
    };

    let mut last_modified_line = None;

    for n in range.start..=range.end {
        let line = match buf.line(n) {
            Some(l) => l.to_string(),
            None => continue,
        };
        let line_bytes = line.as_bytes();

        let new_line = if global {
            substitute_all(&re, line_bytes, &replacement)
        } else {
            substitute_first(&re, line_bytes, &replacement)
        };

        if let Some(new_bytes) = new_line {
            let new_str = String::from_utf8_lossy(&new_bytes).into_owned();
            buf.replace_line(n, new_str);
            last_modified_line = Some(n);
        }
    }
```

`spec.resolve` with its default of the current line, which is ed's: a bare `s`
changes one line, and `,s` or `1,$s` changes all of them. The command did not
have to know any of that. It is the return on chapter 4.

`buf.line(n).to_string()` copies the line before working on it, and the copy
is not waste. `line` borrows from `buf`, `replace_line` needs `buf` mutably,
and the two cannot overlap. Copying first ends the borrow. The alternative,
taking the line out of the buffer with something like `std::mem::take`, avoids
the copy and leaves the buffer holding an empty line for as long as the
substitution takes, which is fine until the code between the take and the put
learns to return early.

`String::from_utf8_lossy` is chapter 9's data loss. The engine produced bytes,
the buffer holds strings, and this is the one place they meet. Most of the
time the bytes are valid UTF-8 and the conversion copies them. When a match
split a character, the conversion is where the damage becomes permanent,
because it replaces what it cannot decode rather than refusing. In 0.3.0 the
line is `String::from_utf8(new_bytes).expect(...)`, with a comment explaining
why the result is valid by construction: the engine consumes whole characters,
so every boundary it reports is a character boundary. The conversion went from
repairing a broken invariant to asserting a kept one.

`replace_line` is the one addition to `buffer.rs` this slice needed:

```rust,ignore
    /// Replace the content of 1-indexed line `n`. Marks the buffer
    /// as modified. Panics if `n` is out of range.
    pub fn replace_line(&mut self, n: usize, content: String) {
        self.lines[n - 1] = content;
        self.modified = true;
    }
```

It can panic on a bad address, and its doc comment says so. That is a weaker contract than chapter 3's `line`, which
returns `None`, and the reason it is acceptable is that the only caller passes
an `n` it has just read with `buf.line(n)`. The invariant is real and it is
held by the call site, which is the arrangement chapter 4's `unreachable!` was
criticized for. The difference is one of distance: here the check and the
reliance on it are four lines apart.

## 10.6 What it says afterwards

```rust,ignore
    match last_modified_line {
        Some(n) => {
            buf.set_current(n);
            match buf.line(n) {
                Some(s) => Action::Print(s.to_string()),
                None => Action::Error("invalid address".to_string()),
            }
        }
        None => Action::Error("no match".to_string()),
    }
```

The current line moves to the last line that changed, which is ed's rule, and
that line is printed. ed prints nothing unless the command ends in `p`; ved
always shows the result, which is the program's purpose stated in chapter 1
showing up as an `Action::Print`. Across a range, only the last changed line is
shown:

```text
a
alpha
beta
gamma
.
1,3s/a/A/
gAmma
```

That is a compromise between printing every changed line, which on `,s` over a
long file floods the terminal, and printing a count, which would be more
consistent with the `deleted 3 lines` confirmations chapter 14 introduces and
less useful for the common case of fixing one line.

`no match` is reported only when nothing in the range matched. A range in which
some lines matched is a success, and the lines that did not match are
unchanged without comment. That is ed's behavior, and it is also what makes
`,s/foo/bar/g` usable at all: a substitution across a file is expected to miss
most lines.

## 10.7 Repeating the last substitution

Bare `s` arrived later, in version 0.1.5, and it is short:

```rust,ignore
    let (pattern, replacement, global) = if args.is_empty() {
        match buf.last_subst() {
            Some((p, r, g)) => (p.clone(), r.clone(), *g),
            None => return Action::Error("no previous substitute".to_string()),
        }
    } else {
        // ... scan the fields as before ...
        buf.set_last_subst(pattern.clone(), replacement.clone(), global);
        (pattern, replacement, global)
    };
```

The `no delimiter` error disappears, because an empty argument now means
"again". The interesting decision is where the remembered substitution lives,
and the commit that added it states it: *the state lives on Buffer alongside
filename and current-line — all single-session REPL state.*

Chapter 5 argued for putting the filename on `Buffer` because "which file is
this" is a property of the text, and that argument does not extend to the last
substitution. The last thing the user replaced is a fact about the session. The
two places are equivalent until something replaces the buffer while the session
continues, and `e` does that, with `*buf = Buffer::new()`:

```text
e notes.txt
8
s/teh/the/
the one
w
wrote 8 bytes to notes.txt
e draft.txt
8
s
? no previous substitute
```

In ed, the remembered substitution survives `e`, and the second `s` fixes the
same typo in the second file, which is the use the feature exists for. ved
forgets, because the memory was stored in the object that `e` throws away.

It is a small bug with a general lesson about convenience. `Buffer` was the
place that was already threaded through `dispatch`, so adding a field to it
meant no new parameter anywhere, and the commit message counts that as an
advantage. The cost of storing state in the nearest available struct is that
it inherits that struct's lifetime, whether or not the state should.

ed has a related feature ved does not: `s//new/` with an empty pattern reuses
the last *pattern*, whether it came from `s`, `g`, or a search. In ved the
empty pattern is compiled like any other, matches the empty string at the start
of the line, and inserts the replacement there. It is in the backlog, and it
would need the remembered pattern to live somewhere `g` can reach as well.

## 10.8 What slice 5b bought

The program can change text, with a regex engine that is its own, through a
command that ed users already know. At 1,711 lines, this checkpoint is the
first one where ved does the job people use ed for.

It also has the program's densest cluster of disagreements with ed, and they
share a cause. The scanner interpreted an escape it should have passed on. The
`g` loop discarded the positions it needed. The flag parser accepted characters
it did not understand. In each case a component answered a question that belonged to
somebody else, or declined to ask a question that was its own, and the result
was a plausible line of output. None of it was visible to tests that checked
the cases their author had in mind, which is chapter 15's subject.

---

**Exercise 10-1.** Make the flag suffix strict: accept `g`, `p`, and a decimal
count, reject anything else with `? invalid command suffix`, and implement the
count by replacing only the *n*th match. Then decide what `s/a/b/2g` should do,
and check GNU ed's answer.

**Exercise 10-2.** Make the closing delimiter optional, as it is in ed. Find the
one case where this is ambiguous, which involves the pattern field rather than
the replacement, and decide what ved should do with it.

**Exercise 10-3.** Apply the `scan_delimited` fix from section 10.2 to
`examples/ved`, including the one-arm change to `expand_replacement`. Then
write the four tests that would have caught the original: one for each of `.`,
`*`, and `&` as the delimiter, and one for `\n` in a replacement.

**Exercise 10-4.** Apply the `find_at` fix from section 10.4. Write a test for
each of the three failures before you apply it and watch them fail. Then find
out whether `g/^a/s/a/X/g` was ever affected, and explain why or why not.

**Exercise 10-5.** Move the last substitution off `Buffer` and into `run_repl`,
so that `e` preserves it. Work out how `dispatch` gets access to it, and decide
whether that changes chapter 6's claim that `dispatch` holds no state.
