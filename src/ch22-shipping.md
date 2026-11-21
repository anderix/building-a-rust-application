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
description (#30) has no commit at all, because it is not in the repository,
which section 22.5 explains.

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

## 22.5 What the release could not reach

**The WinGet description** (#30). The manifest still calls ved "a drop-in ed
clone." `ship-winget` builds each new manifest with `komac update`, which
copies the previous version's description forward, so the text from the
first submission has been carried into every version since, and 0.4.0 is no
exception. Fixing it needs either a change to the release tool, so that the
description comes from `Cargo.toml` at every release, or a manifest edited by
hand. The fleet's rule is that releases go through the tools and not around
them, so the issue stays open until the tool can do it. That is the right
call for a sentence of marketing copy, and it would be the wrong one for a
security fix; a rule that forbids hand-editing needs to know which it is
facing.

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

**What the tracker still holds.** #36 lists the differences the harness still
finds, and #38 the `l` escapes. Both were filed after the branch was planned
and are for the next release. Shipping with known open issues is the normal
case, and the useful property is that they are known, written down, and
linked from the chapter that found them.

## 22.6 The loop

The loop chapter 19 described, someone reads the program closely, writes down
what is wrong, and the list goes into the place where lists get done, has now
gone around once, and it can be measured. Eighteen findings from chapters 10 to
19 went into the tracker on one day, and reading the specification added two
more. Twenty issues were closed the same day, one commit each, each with a test
whose expected value came from a reference, and a twenty-first, the tutorial,
closed when the websites were deployed. Two more bugs were found by the tools
that checked the fixes, and one was introduced by a fix and caught before it
shipped. The release went out through four channels and verified each, and the
notes that went with it had two mistakes that the next release can avoid.

None of that needed anything this book has not shown. The tracker is
chapter 19's. The order is chapter 20's rule. The procedure is chapter 15's
tests and harness and fuzzer, aimed at a list instead of at a program. The
release is chapter 17's, run by the tools chapter 19 described. What the loop
adds is the habit of going around it: of treating the close reading as the
start of the work rather than the end of it.

ved 0.4.0 is a better program than 0.3.0 in the ways chapter 20 listed, and it
still has bugs. Some are in #36 and #38. The rest are in the commands nobody
has read closely yet, which is where chapter 20 said they would be, waiting
for the next person who explains the code to somebody in writing.

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

**Exercise 22-4.** Design the change to the release tool that #30 needs: where
the description comes from, and what the tool should do when the published
manifest disagrees with it.
