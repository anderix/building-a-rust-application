# 13. Files, and the Outside World

Until slice eleven, every line in the buffer was typed by the person at the
keyboard. From here the program reads files it did not write, made by other
programs, on other operating systems, in encodings it may not understand, and
the assumption that the program knows what a line is stops being safe.

```text
$ cargo run -q -p ved -- notes.txt
22
,n
1	shopping
2	.
3	eggs
4	bread
a
milk
.
e other.txt
22
,p
contents of other.txt
```

The `milk` is gone, and nothing said it would be. GNU ed would have refused the
`e` until it was typed twice. Section 13.5 is about that.

Slice eleven arrived in one commit with slices eight, nine, and ten, so the
checkpoint `examples/ch13-files` holds four slices: long-form aliases, the
rewritten error messages, the `H` help command, and file loading. Chapter 14
takes the first three. This chapter takes the last, plus what was added to it
after release: treating a missing file as a new one in 0.1.2, and the encoding
warnings in 0.2.0.

## 13.1 A filename on the command line

Chapter 2's argument parser rejected anything it did not recognize, and said
that this would be the first thing to loosen. Slice eleven loosens it:

```rust,ignore
            s if s.starts_with('-') => {
                return Err(format!("unknown option: {s}"));
            }
            _ => {
                filename = Some(arg.clone());
            }
```

Unknown flags still fail, and a bare word is a filename. Two bare words are two
filenames, and the second silently replaces the first, so `ved notes.txt
small.txt` opens `small.txt`. GNU ed opens the first and silently ignores the
rest. Neither says anything, and they disagree about which file you get.
`parse_args` now returns a pair, `(prompt, filename)`, and `run_repl`
takes the filename and loads it before the loop starts, which is the twenty
lines chapter 6 counted as the loop's growth.

The guard has one gap that chapter 2's exercises pointed at:

```text
$ ved -- -p
ved: unknown option: --
```

`--` is the universal way to say "everything after this is an operand, even if
it starts with a dash", and it is how a user edits a file named `-p`. ved
treats `--` as an unknown flag. The fix is one arm that sets a flag to stop
looking for options, and it is the sort of thing an argument-parsing crate gets
right without being asked, which is part of the price of chapter 1's
constraint.

## 13.2 What a buffer can hold

```rust,ignore
fn load_file(filename: &str, buf: &mut Buffer) -> Result<LoadOutcome, String> {
    let detected = enc::warn_if_non_utf8(filename);
    let content = match std::fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            buf.set_filename(filename.to_string());
            buf.mark_saved();
            return Ok(LoadOutcome::NewFile);
        }
        Err(e) => return Err(format!("cannot open {filename}: {e}")),
    };
    let bytes = content.len();

    for line in content.lines() {
        buf.append_after(buf.len(), line.to_string());
    }
    buf.set_filename(filename.to_string());
    buf.mark_saved();
    if let Some(enc) = detected {
        buf.mark_non_utf8_source(enc, filename.to_string());
    }

    Ok(LoadOutcome::Loaded(bytes))
}
```

Chapter 4 said this chapter would examine the difference between the command
language and the buffer, and `read_to_string` is where the program answers it.
The command language is ASCII, so the address parser and the dispatcher can
work in bytes. The buffer is `Vec<String>`, and a `String` is valid UTF-8 by
construction, so a file can only enter the buffer if it is valid UTF-8.
`read_to_string` checks, and refuses anything else:

```text
$ ved latin1.txt
? cannot open latin1.txt: stream did not contain valid UTF-8
```

That file is `caf` followed by the byte `E9`, which is `é` in Latin-1 and is
not valid UTF-8 on its own. GNU ed opens it, shows it, and writes it back
unchanged, because ed's buffer is bytes and it has no opinion about what they
encode. ved cannot open it at all.

This is a real limitation and it is a defensible one. The alternative types
for the buffer are `Vec<Vec<u8>>`, which makes every command that prints or
matches a line responsible for deciding what to do with bytes that are not
text, or `Vec<OsString>`, which has the same problem with less convenient
methods. Either is more correct, and either would have spread a question that
the program currently answers once, at the door. And refusing is the right
failure. Chapter 9 showed what happens when bytes that are not a whole
character get into a `String` through `from_utf8_lossy`: the text is changed
and nothing says so. A program that refuses to open a file has lost nothing.

The error message is less good than the decision. `stream did not contain valid
UTF-8` is the standard library's wording, and it does not say what the user can
do. Section 13.7 is about the module that was written to improve it.

## 13.3 What a line is

`content.lines()` splits the text into lines, and every line goes into the
buffer through `append_after`, the primitive from chapter 3. Loading a file is
typing it very fast.

Chapter 3 noted that a line consisting of one `.` cannot be typed into the
buffer, because it ends input mode, and promised that files containing one
would load without trouble. They do, because loading never enters input mode:

```text
$ printf 'a\n.\nb\n' > dot.txt
$ ved dot.txt
6
,n
1	a
2	.
3	b
```

`lines()` does two things beyond splitting on `\n`, and both are
conveniences. It does not produce an empty last line when the text ends in a
newline, which is right, since `a\nb\n` is two lines and not three. And it
strips a `\r` before each `\n`, so that a file with Windows line endings comes
out with no carriage returns at the ends of its lines.

The second one is a silent conversion:

```text
$ printf 'one\r\ntwo\r\n' > crlf.txt
$ ved crlf.txt
10
w
wrote 8 bytes to crlf.txt
q
```

Ten bytes in, eight bytes out. Every line ending in the file was changed from
CRLF to LF, and the user's only edit was to save. `run_write` joins lines with
`\n`, as it should, and the information that they were ever joined with
anything else was discarded by `lines()` before the first command ran. GNU ed
writes back ten bytes, because to ed the `\r` is the last character of the
line. Filed as [ved #18](https://github.com/excelano/ved/issues/18).

ved ships through WinGet, so this is not a hypothetical. A user on Windows
opening a file their editor saved with CRLF endings, fixing one typo with ved,
and committing it, produces a diff that changes every line.

There are two fixes, and they differ in what the user sees. The ed fix is to
split on `\n` alone and keep the `\r`:

```rust,ignore
    for line in content.split_terminator('\n') {
        buf.append_after(buf.len(), line.to_string());
    }
```

`split_terminator` behaves like `lines()` about the final newline and leaves
everything else alone, so the file round-trips byte for byte. In a scratch copy
of `examples/ved` with this change in both `load_file` and `run_read`, the CRLF
file writes back as ten bytes and the 125 tests pass. The cost is the one ed
users already pay: the `\r` is part of the line, so `s/e$/E/` on `one\r` is
`? no match`, because the last character is not `e`, and `l` shows the `\r`
as `\r`.

The other fix is to detect the line-ending style at load, strip the `\r` as
`lines()` does now, remember the style on `Buffer` beside the filename, and use
it in `run_write`. That keeps `$` working and the file intact, and it needs a
decision about a file whose endings are mixed. ed's version is two words; the
other is a feature. Either is better than converting without saying so.

A file that does not end in a newline gets one, in both programs:

```text
$ printf 'abc' > nonl.txt
$ ved nonl.txt
3
w
wrote 4 bytes to nonl.txt
```

GNU ed does the same and prints `Newline appended` when it loads the file.
ved changes the file without the warning, which, for a program whose purpose is
to say what it did, is the wrong one to be quieter about.

## 13.4 A missing file is a new file

```rust,ignore
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            buf.set_filename(filename.to_string());
            buf.mark_saved();
            return Ok(LoadOutcome::NewFile);
        }
```

`ved notes.txt` on a file that does not exist yet starts an empty buffer that
remembers the name, so that `w` will create it:

```text
$ ved brandnew.txt
brandnew.txt: new file
```

That arrived in 0.1.2. Before it, a missing file was `? cannot open`, which is
correct and useless, since creating a file is the most common reason to name
one that is not there. ed behaves the same way, and the guard is narrow: only
`NotFound` is treated as new. A directory, a permission failure, or anything
else is still an error with the operating system's own words in it.

`LoadOutcome` is the enum that carries the distinction back to the caller. It
is two variants, and it replaces the alternative of returning a byte count of
zero and hoping the caller can tell an empty file from a missing one.

## 13.5 Replacing the buffer

```rust,ignore
/// Edit: replace the buffer with the contents of a file.
/// Warns if the buffer has unsaved changes (like q does).
fn run_edit(buf: &mut Buffer, args: &str) -> Action {
    let filename_arg = args.trim_start();

    let filename = if filename_arg.is_empty() {
        // ... the remembered filename, or an error ...
    };

    // Replace the buffer entirely.
    *buf = Buffer::new();
    match load_file(&filename, buf) {
        Ok(LoadOutcome::Loaded(bytes)) => Action::Print(format!("{bytes}")),
        Ok(LoadOutcome::NewFile) => Action::Print(format!("{filename}: new file")),
        Err(e) => Action::Error(e),
    }
}
```

The doc comment says `e` warns about unsaved changes, as `q` does. It does not.
There is no check of `is_modified` anywhere in the function, and the first
thing it does is throw the buffer away:

```text
a
an afternoon of work
.
e other.txt
22
,p
contents of other.txt
```

GNU ed answers the `e` with `? Warning: buffer modified` and does nothing until
it is typed a second time, which is the same double-command dance as `q`. ved
discards the work without a word. Filed, with the failed load below, as
[ved #17](https://github.com/excelano/ved/issues/17).

The ordering makes the second case worse. `*buf = Buffer::new()` happens before
`load_file`, so if the load fails, the buffer is already gone and there is
nothing to replace it with:

```text
a
an afternoon of work
.
e latin1.txt
? cannot open latin1.txt: stream did not contain valid UTF-8
,p
? invalid address
```

The command failed, said so, and lost the buffer anyway. Of everything this
book has found in ved, this is the case most likely to cost somebody real work,
because it needs nothing unusual: a typo in a filename that happens to name a
binary file, or a file in the wrong encoding, after an hour of unsaved editing.
It is in 0.3.0.

Both problems have short fixes. Load into a new buffer and replace the old one
only if the load succeeded:

```rust,ignore
    if buf.is_modified() && !force {
        return Action::Error("buffer modified; use E to discard changes".to_string());
    }

    let mut fresh = Buffer::new();
    let outcome = match load_file(&filename, &mut fresh) {
        Ok(o) => o,
        Err(e) => return Action::Error(e),
    };
    *buf = fresh;
```

The `force` parameter is ed's `E`, which is `e` without the check, dispatched
as `'E' => run_edit(buf, args, true)`. ed itself accepts either a second `e` or
an `E`. Implementing the second `e` would need the loop's `quit_warned` state,
which `dispatch` cannot see, so this version takes only the `E` route and keeps
`dispatch` stateless; the message says which command to use. Built into a
scratch copy, `e` on a modified buffer refuses, `E` on a file that fails to load
leaves the buffer as it was, `E` on a good file loads it, and the 125 tests
pass.

The general lesson is about the order of destructive operations. Construct the
replacement first, then swap it in. A function that clears state and then does
something fallible has made its failure path destroy the thing it was asked to
replace, and the `match` that reports the error afterwards gives the impression
of handling it.

## 13.6 Reading into the buffer

`r` inserts a file's contents after an addressed line, defaulting to the end of
the buffer. Its address handling has a shape that chapter 11's `resolve_target`
would simplify:

```rust,ignore
    let after = if buf.is_empty() {
        0
    } else if spec.is_empty() {
        buf.len()
    } else {
        match spec.resolve(buf) {
            Ok(r) => r.end,
            Err(e) => return Action::Error(e),
        }
    };
```

The empty case, the default, and the general case, written out separately,
because `resolve` fails on an empty buffer and rejects 0. `0r header.txt` is an
ed idiom for putting a file at the top, and in ved it is `? invalid address`.
That is at least an error, which is better than what `0a` did.

`r` is the one command that loads a file without being able to lose anything,
since it only adds lines, and its failure is clean:

```text
r nope.txt
? cannot open nope.txt: No such file or directory (os error 2)
```

It prints the byte count it read, as ed does, and if the buffer had no filename
it takes the name of the file it read. Both are ed's behavior, and the second
one is the kind of rule nobody would guess, which is a reason to copy it from a
specification rather than decide it.

## 13.7 Encodings, sniffed

```rust,ignore
/// Sniff the first 4 KiB of `path` for a non-UTF-8 signature. On detection,
/// emit a two-line warning to stderr with a copy-pasteable iconv command and
/// return the encoding display name (e.g. "UTF-7"). Sniff failures (missing
/// file, IO error) and clean UTF-8 files return None silently.
pub fn warn_if_non_utf8(path: &str) -> Option<String> {
```

`enc.rs` arrived in 0.2.0 to improve the error from section 13.2 for the two
non-UTF-8 encodings its author kept meeting. A file that starts with the byte
order mark of UTF-16 gets a warning that names the encoding and a command that
converts it:

```text
$ ved u16.txt
warning: u16.txt appears to be UTF-16 little-endian encoded.
hint: iconv -f UTF-16LE -t UTF-8 u16.txt > u16.utf8.txt
? cannot open u16.txt: stream did not contain valid UTF-8
```

The cryptic error still follows, but the user has been told what to do. That is
chapter 14's subject arriving early, and it is a good message: it names the
problem in the user's terms and hands them the fix.

The module's header says something about the program as a whole: *inlined
(rather than depending on the encsniff crate) to keep ved pure-stdlib*. The
detection logic already existed as a crate, written by the same author for
sibling programs, and ved could not use it. Chapter 1 said the no-dependencies
constraint answered questions in a minute instead of a week, and this is one of
them. The answer cost a 131-line module that duplicates code living elsewhere,
which has to be kept in step with it by hand, as the header says it is.

The second encoding is the problem. UTF-7 has no byte order mark, so the module
looks for a marker instead:

```rust,ignore
const UTF7_MARKER: &[u8] = b"+ACI-";
```

`+ACI-` is how UTF-7 writes a double quote, and it appears in UTF-7 exports
whenever the text contains one. The module comment says *detection rules are
deliberately narrow — byte-perfect signatures only, no heuristics*, and this
rule is a heuristic. UTF-7 is a 7-bit encoding, so every UTF-7 file is also a
valid ASCII file and a valid UTF-8 file, and there is no signature that can
tell them apart. The marker is a guess about content.

The guess has a consequence, because 0.2.0 also added a refusal to overwrite a
file that was detected as non-UTF-8:

```text
$ cat note.txt
He said +ACI-hi+ACI- in UTF-7.
$ ved note.txt
warning: note.txt appears to be UTF-7 encoded.
hint: iconv -f UTF-7 -t UTF-8 note.txt > note.utf8.txt
31
s/said/says/
He says +ACI-hi+ACI- in UTF-7.
w
? refusing to overwrite note.txt (loaded as UTF-7); convert with iconv first, or use `w <newname>` to write elsewhere
```

That file is plain UTF-8 that happens to contain the marker, as a sentence
about UTF-7 would. It cannot be saved in place. Filed as [ved
#25](https://github.com/excelano/ved/issues/25).

Follow the two detections through `load_file` and the refusal turns out to
protect only this case. A UTF-16 file fails `read_to_string` and never reaches
the buffer, so `mark_non_utf8_source` is never called for it. The only files
that are ever marked, and so the only files the refusal ever blocks, are the
ones that passed the UTF-7 heuristic, all of which are valid UTF-8 and would
have been written back byte for byte. The guard was written for a danger, saving
a file's content in the wrong encoding, that cannot occur on the path that
reaches it.

What the guard was protecting against is real in a narrower form. A user who
types a non-ASCII character into a real UTF-7 file and saves it produces a file
that is neither encoding. The warning on load covers that, and the refusal adds
nothing but the false positive. The fix is to keep the warning and drop the
refusal for UTF-7, which deletes code. The lesson is about guards in general:
a check that blocks the user needs the same scrutiny as the operation it guards,
because a false positive in a refusal is a bug the user cannot work around
without leaving the program.

## 13.8 What `w` does not do

`run_write` is chapter 5's function with the refusal added. The file it writes
is the one named, and that includes names ed would not treat as names:

```text
w !cat
wrote 2 bytes to !cat
```

In ed, `w !command` pipes the buffer to a shell command and `r !command` reads
a command's output into the buffer. They are how ed users sort a range, format
a paragraph, or insert a date. ved has no shell escapes, so `w !cat` creates a
file named `!cat` in the current directory, which is the worst of the available
behaviors for a script written for ed: it does not fail, and it leaves a file
behind.

The standard library can run a command, with `std::process::Command`, so this
is not a cost of the constraint. It is unimplemented. A one-line check that
refuses filenames beginning with `!`, with a message saying shell commands are
not supported, would turn a silent wrong answer into a refusal, and it
is the right first step before the feature.

## 13.9 How large a file

Chapter 3 noted that `,p` builds its whole output as one `String` and said this
chapter would come back to it when files got large. They have. A file of one
million lines, 56.9 MB, measured on the machine this was written on, with peak
memory from `getrusage`:

| Command | Time | Peak memory |
|---|---|---|
| open and `q` | 0.08 s | 140.8 MB |
| `,p` to `/dev/null` | 0.12 s | 140.9 MB |
| `w out.txt` | 0.12 s | 141.6 MB |
| `g/7/p` | 0.27 s | 140.0 MB |
| `,s/line/LINE/` | 0.28 s | 140.2 MB |
| GNU ed, open and `q` | 0.28 s | 47.7 MB |
| GNU ed, `,s/line/LINE/` | 2.33 s | 141.1 MB |

The `,p` copy does not show up at all. Its peak is the same as opening the file
and quitting, because loading already holds two copies of the text for a
moment: the `String` from `read_to_string`, and the million `String`s that
`lines()` and `append_after` build from it, before the first is dropped. `,p`
builds one buffer-sized string while the lines exist, which is the same shape.
The cost chapter 3 worried about is real and is dominated by one it did not
mention.

GNU ed opens the same file in a third of the memory because it keeps the text
in a scratch file on disk and holds only an index in memory, which is a design
from a machine with 64 KB of address space. ved is three times faster at
opening and eight times faster at a substitution across the file, and uses
three times the memory to be so. For the files people edit with ed, that is
the right side of the trade. Chapter 3 bet that `Vec<String>` would be fast
enough, and at a million lines it still is.

## 13.10 What slice eleven bought

A program that edits files: `ved file` on the command line, `e` to switch,
`r` to read one in, `w` to save, and a missing file that becomes a new one.
At the end of slice eleven the program is 2,006 lines, and it is the first
checkpoint that could replace ed for someone's daily work.

It is also where most of the data loss in this book lives. CRLF endings are
rewritten without a word, a failed `e` destroys the buffer, and a guard meant
to prevent corruption blocks saving an ordinary file. None of them is visible
in a session that types its text by hand, which is how nearly every test in the
program builds its buffer. The two tests that load a file check a missing file
and a directory. All three problems are visible the first time the program
meets a file from somewhere else.

---

**Exercise 13-1.** Make `ved -- -p` open a file named `-p`. Then decide what
`ved -p` (with no value) should do now that a bare word can be a filename, and
check that your parser and chapter 2's agree.

**Exercise 13-2.** Apply the `split_terminator` fix from section 13.3. Then
implement the other one: detect CRLF at load, remember it on `Buffer`, and use
it in `run_write`. Decide what a file with mixed endings should get, and write
the test for each.

**Exercise 13-3.** Apply the `e` fix from section 13.5, add `E`, and write a
test for each of the three cases: refused, forced and failed, forced and
loaded. Then make a second `e` act as a confirmation, the way a second `q`
does, without giving `dispatch` any state.

**Exercise 13-4.** Remove the overwrite refusal for UTF-7 and keep the warning.
Then decide whether a UTF-8 file containing `+ACI-` should be warned about at
all, and if not, what evidence would be enough to say a file is UTF-7.

**Exercise 13-5.** Implement `r !command` with `std::process::Command`, reading
the command's standard output into the buffer. Decide what happens to its
standard error and its exit status, and what the confirmation should say.

**Exercise 13-6.** Reduce the peak memory of opening a file. Read it with
`BufRead::lines` instead of `read_to_string`, measure again, and explain why
the number did or did not halve.
