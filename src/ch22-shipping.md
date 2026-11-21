# 22. Shipping the Fixes

A fix on a branch helps nobody. This chapter takes the twenty-three commits
from chapter 21 to the people who have ved installed: the version number, the
merge, the release, the four channels, what the release notes said, and what
the release could not reach. It happened on November 21, the same day as the
fixes, and every output quoted here is from that run.

## 22.1 Which number

Chapter 17's rule is that the version says what changed for the person using
the program. Three of 0.4.0's changes are new things a user can do: the long
forms, `%`, and `-s`. Those make it at least a minor release. Several more
change what an existing input does: a script that ignored errors now exits 1,
a Windows file keeps its line endings, `0a` puts text somewhere else, a
one-address `j` no longer moves the cursor. Each of those is a bug fix, and
each could break a script that relied on the bug.

Before 1.0, Cargo's rules treat a minor release as the breaking one, so
`0.3.0` to `0.4.0` is the number that says "read the notes before you
upgrade," and it is what `build-release minor` produced. After 1.0, the same
set of changes would raise a harder question. The usual reading of semantic
versioning is that a change to documented behavior is breaking, and a change
to undocumented, wrong behavior is a fix, and ved's exit status was
documented by nothing except its own code. A project that reaches 1.0 needs
that line drawn in writing before the first release that tests it, because
drawing it during the release is how a project talks itself out of a fix.

## 22.2 One branch, one merge

The fixes were made on a branch and merged as one pull request, #37, rather
than committed to `main` one at a time. The branch kept `main` releasable while
twenty-three commits were in progress, and it let one commit be rewritten
safely: the delimiter fix in section 21.9 was amended, and the messages that
said "as in ed" were corrected to name which ed, both before anything was
pushed. Rewriting history that nobody else has is housekeeping. Rewriting
history that somebody has pulled is a problem for them.

The merge closed every issue whose fix said `Fixes #N`. The tutorial's fix
said `Refs #16` instead, and the issue stayed open on purpose. The tutorial
was fixed in the repository, but the copies on anderix.com and excelano.com
are separate files that change only when those sites are deployed, and
closing the issue before then would have claimed that a reader of either site
saw the fix. It closed when they were deployed, the same evening. The WinGet
description (#30) has no commit in ved at all, because it is not in ved's
repository, which section 22.5 explains.

## 22.3 The release

`build-release minor -y`, run from ved's root, did the release. Its output,
trimmed:

```text
==> pushing main
==> building debs
    /home/anderix/repos/tools/ved/target/distrib/ved_0.4.0-1_amd64.deb
    /home/anderix/repos/tools/ved/target/distrib/ved_0.4.0-1_arm64.deb
==> building ved-x86_64-pc-windows-msvc.zip on winwork
    ved-x86_64-pc-windows-msvc.zip
==> tagging v0.4.0
==> creating the release
    https://github.com/excelano/ved/releases/tag/v0.4.0
==> uploading 3 assets
==> verifying the release
    ved-x86_64-pc-windows-msvc.zip
    ved_0.4.0-1_amd64.deb
    ved_0.4.0-1_arm64.deb
0.4.0
```

Before any of that, it bumped `Cargo.toml` and ran the tests, all 225 of
them, so the commit that the tag points at is one that passed. Then three
more commands, one per channel, each of which ends by checking the live
channel rather than trusting its own success:

```text
$ ship-apt -y
...
==> verifying live
    amd64  ved 0.4.0-1
    arm64  ved 0.4.0-1
==> excelano/ved 0.4.0 is live on apt

$ ship-crates -y
...
==> verifying crates.io
    ved 0.4.0

$ ship-winget -y
...
==> submitting Excelano.ved 0.4.0
    https://github.com/microsoft/winget-pkgs/pull/486054
```

Chapter 17 described a release procedure that lived in a document; chapter
19 described it moving into programs. This is what that bought on a release
day: four commands, run by the person who wrote the fixes, with nothing
remembered and nothing skipped. The WinGet line ends at a pull request
because WinGet's catalogue is a repository someone else reviews, and the
release reaches Windows users when a moderator merges it.

## 22.4 What the notes said

`build-release` writes the release notes from the commit subjects since the
last tag, which is why chapter 16 asked for subjects written for the person
installing the release. The GitHub release for 0.4.0 begins:

```text
- J with one address leaves the current line where it was
- G/re/d leaves the line after the last deletion current
- A delimiter that is not one ASCII byte is an error, not a panic
- A UTF-8 file containing +ACI- can be saved in place
- -s prints only what the commands ask for
- End of input on a modified buffer warns, as q does
- Exit nonzero when any command failed
...
- E warns before discarding a modified buffer, and a failed e keeps it
- Write, edit and read long forms, and no filename glued to w, e or r
- Wq writes the file and quits
```

Two things in it are wrong, and neither was visible until the notes existed.

The tool removes the `fix:` prefix and capitalizes what is left, which is
right for an English sentence and wrong for a subject that begins with an ed
command. `j`, `g`, and `wq` became `J`, `G`, and `Wq`, which a reader can
probably decode. `e` became `E`, which is a different command: the note says
that `E` warns before discarding a buffer, and `E` is the one command that
does not. The same subjects went into the Debian package's changelog. The
fix for the next release is in the subjects, not the tool: a subject for a
program whose commands are single letters starts with a word, such as "the
`e` command warns before discarding a modified buffer", so that the first
letter is safe to capitalize.

The second is what is missing. The notes list `feat` and `fix` commits and
leave out `docs` and `build`, by design, because those are changes to the
repository rather than to the program. That left out the README's new
instructions for macOS users, which exist only to reach people who installed
with Homebrew, and the declared minimum Rust version, which is the first
thing a user with an old toolchain will hit. A change a user needs to hear
about is a user-facing change whatever directory it is in, and its type
should say so. The two commits should have been `fix:`, and the rule they
broke is the one chapter 17 quoted: `fix:` and `feat:` are for changes a user
would notice, which is a question about the user, not about which files the
change touches.

Both were corrected the same evening. The GitHub release's notes were edited,
with each command letter kept lowercase in code style and the two missing
lines added. The Debian changelog was not, and cannot be: it is inside the
packages, and a package changes only with a new version. A generated document
that is wrong in one place is wrong in every place it was copied to before
anyone read it, and the copies that cannot be edited are the ones to think
about before the generator runs.

## 22.5 What the release could not reach

**The WinGet description** (#30). The first 0.4.0 manifest still called ved
"a drop-in ed clone." `ship-winget` builds each manifest with `komac update`,
which copies the previous version's description forward, so the text from the
first submission had been carried into every version since, and 0.4.0 was no
exception. The fix went into the release tool rather than the manifest: for a
repository with one WinGet package, `ship-winget` now sets the description
from `Cargo.toml`'s `description` on every submission, so the description has
one source and a correction reaches WinGet at the next release without anyone
remembering to make it.

The 0.4.0 pull request was then closed and submitted again with the new tool,
the same evening, with the corrected description and the corrected notes.
That broke one of the tool's own rules, one pull request per package per day.
The rule was the fleet's, not WinGet's, whose guide asks only that a version
never have two pull requests open at once, and the first one had been closed.
A rule a project made for itself can be set aside when its reason does not
apply. Telling the rules a project made from the rules it was given is what
makes that safe.

**The websites** (#16). The two published copies of the tutorial are not the
repository's file, so the release did not change them. Each got the same
corrections by hand and was deployed that evening, and then the issue closed.
Doing it turned up two more pages: ved's page on excelano.com and its card on
the home page still called it a drop-in clone and said that any script
written for ed runs unchanged, the claim the README had dropped in July.
They were corrected in the same deploy. Chapter 18 counted six copies of
ved's one-sentence description; these were two more that nobody had listed,
and a correction reaches only the copies on the list.

**Homebrew users.** Chapter 19 found that people who installed 0.2.1 with
`brew` were left on it, with no way to learn that anything had changed. 0.4.0
does not reach them either, because no channel they use carries it. The
README now tells them what to do, and the README is the one place they are
unlikely to look. The only message that reaches a user of a dead channel is
one sent before it dies.

**What the tracker still held.** #36 listed the differences the harness still
found, and #38 the `l` escapes. Both were filed after the branch was planned,
so 0.4.0 shipped with them open, which is the normal case. The useful
property is that they were known, written down, and linked from the chapter
that found them, which is what made the next section short.

## 22.6 0.4.1

The two open issues were fixed the same night, on a second branch merged as
pull request #39, and released as 0.4.1. A patch release, because nothing in
it adds a command; it makes five existing behaviors match the reference.

#36 turned out to have three causes, none of them the one its description guessed.
The `g` and `j` case was not about `g`: `+1m-1` from line 1 failed because an
offset that landed on 0 was rejected even where 0 is a valid address. An `s`
with an invalid address was recorded for a bare `s` to repeat, because ved
parsed the command before it resolved the address, where ed does the
reverse. And `,` on an empty buffer was rejected for `a` and `i`, where POSIX
bounds only the final address, which there is 0. Each was a commit with a
test whose expected values came from GNU ed's output, and after them the
harness, run without its empty-match command, finds 3 differing scripts in
3,000.

All three are one case: `m` to the last line of its own range, such as
`1m1`. GNU ed accepts that as a move that changes nothing. POSIX says it is
an error "if address falls within the range of moved lines," and Seventh
Edition ed rejects it, so ved does too, and #36 closed with that decision
written on it. A harness that ends at a short list of differences, each with
a reason, is a harness that has done its job; it was never going to end at
zero, because the reference is not the specification.

#38 made `l` write `\t`, `\r`, and the other escapes POSIX names, and fold
long lines. POSIX leaves the folding width open, so ved folds where GNU ed
does, measured by running it on lines of 75, 76, and 77 characters, and the
output now matches GNU ed's byte for byte, folds included. The test that had
held the octal tab in place was replaced by one whose expected value is GNU
ed's output.

Two of the commit subjects began with a command, `l` and `,`, and would have
come out of the release-notes generator as "L writes" and ", on an empty
buffer." They were reworded to begin with a word before the branch was
pushed, which is section 22.4's lesson applied once, and 0.4.1's notes read
correctly as generated.

The WinGet pull request for 0.4.0 was still waiting for a moderator. Only the
current version is owed a manifest, so it was closed and 0.4.1 submitted in
its place, with the description from `Cargo.toml`. A Windows user goes from
0.3.0 to 0.4.1 directly, and loses nothing by skipping a version they never
had.

## 22.7 The loop

The loop chapter 19 described, someone reads the program closely, writes down
what is wrong, and the list goes into the place where lists get done, has now
gone around once, and it can be measured. Eighteen findings from chapters 10 to
19 went into the tracker on one day, and reading the specification added two
more. Twenty issues were closed the same day, one commit each, each with a test
whose expected value came from a reference, and a twenty-first, the tutorial,
closed when the websites were deployed. Two more bugs were found by the tools
that checked the fixes, and one was introduced by a fix and caught before it
shipped. The release went out through four channels and verified each, and the
notes that went with it had two mistakes, corrected that evening, that the next
release can avoid.

None of that needed anything this book has not shown. The tracker is
chapter 19's. The order is chapter 20's rule. The procedure is chapter 15's
tests and harness and fuzzer, aimed at a list instead of at a program. The
release is chapter 17's, run by the tools chapter 19 described. What the loop
adds is the habit of going around it: of treating the close reading as the
start of the work rather than the end of it.

ved 0.4.1 is a better program than 0.3.0 in the ways chapter 20 listed, and its
tracker has no open issues. It still has bugs. They are in the commands nobody
has read closely yet, which is where chapter 20 said they would be, waiting for
the next person who explains the code to somebody in writing.

---

**Exercise 22-1.** Rewrite the subjects of the twenty-three commits so that
the release notes read correctly after the type is removed and the first
letter capitalized. Then decide whether the tool or the subjects should
change, and write down why.

**Exercise 22-2.** Write the 0.4.0 entry of a `CHANGELOG.md` by hand, for a
user deciding whether to upgrade. Include what can break a script that
relied on 0.3.0's behavior, and compare it with the generated notes.

**Exercise 22-3.** Draft the policy section 22.1 asks for: what counts as a
breaking change for ved after 1.0. Apply it to every commit in 0.4.0 and
count how many would have required a major version.

**Exercise 22-4.** The fix for #30 takes the description from `Cargo.toml`
only when a repository ships one WinGet package. Design what a release tool
should do for a repository that ships several, each with its own
description, without keeping a second copy of any of them.
