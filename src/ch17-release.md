# 17. Release

A release is a version number attached to a commit, and a promise that the
commit can be obtained by someone who is not you. Everything before this
chapter was about the program. This one is about the promise, and about the
machinery that turns `git tag v0.2.1` into five binaries, two installers, a
Homebrew formula, two Debian packages, and a crate.

ved has had nine releases:

| Version | Released | What changed, for someone using it |
|---|---|---|
| 0.1.0 | 2026-05-14 | The first release: slices one through eleven |
| 0.1.1 | 2026-05-14 | `c`, change |
| 0.1.2 | 2026-05-28 | A missing file is a new file; Debian packages |
| 0.1.3 | 2026-06-12 | `*` is greedy, as POSIX requires |
| 0.1.4 | 2026-06-12 | `l`, list |
| 0.1.5 | 2026-06-12 | `j`, `m`, `t`, octal escapes, bare `s` |
| 0.2.0 | 2026-06-25 | Warnings for UTF-16 and UTF-7 files, and the overwrite refusal |
| 0.2.1 | 2026-09-15 | Two panics fixed; continuous integration |
| 0.3.0 | 2026-11-11 | Search addresses; a character-aware engine (chapter 19) |

The dates come from the GitHub releases. The code was finished on April 12,
when slices eight through eleven landed in one commit. The first release came a
month later, on the day the release machinery was added, and the Debian
packages, the first channel beyond GitHub, two weeks after that.

## 17.1 What the number promises

Cargo versions are semantic versions: `MAJOR.MINOR.PATCH`, where a change to
the major number means existing users may break, a minor number means new
features, and a patch number means fixes. Before 1.0 the rule shifts one place
to the right. Cargo treats `0.2` and `0.3` as incompatible, the way it treats
`1` and `2`, and a patch release within `0.2` as compatible.

For a library the promise is about the API, and the compiler can check most of
it. For a program, the API is whatever other programs and people depend on,
and for ved that is a long list: the command language, the flags, the exit
status, what is printed and where, and what is written to disk. A script that
greps ved's output for `deleted` depends on the confirmation format. Chapter
14's proposal to say `read 22 bytes from notes.txt` instead of `22` is, by
that definition, a breaking change.

ved's history uses the numbers sensibly. 0.2.0 is the release where the program
first refused to do something it used to do, overwriting a file it had flagged
as non-UTF-8, and it took the minor number. 0.2.1 fixed two crashes and changed
nothing else a user could see, and it took a patch number. 0.3.0 changed what
`.` matches in a regex, from a byte to a character, which changes the result of
existing commands on non-ASCII text, and it took the minor number.

Three releases in one evening, 0.1.3 to 0.1.5, is the other end of the scale.
Each one shipped a feature as soon as it was done. That is cheap when release
is automated and expensive when it is not, and the cost is not in the release
itself. It is in the channels downstream, each of which has its own update
cadence and review, which is chapter 18.

## 17.2 The manifest, read by strangers

Chapter 1 filled in `Cargo.toml`'s metadata on the first day and said this
chapter would come back to it:

```toml
[package]
name = "ved"
version = "0.2.1"
edition = "2024"
description = "The verbose ed — an ed clone with friendly errors and a built-in help system"
homepage = "https://excelano.com/ved/"
repository = "https://github.com/excelano/ved"
license = "MIT"
keywords = ["ed", "editor", "line-editor", "regex", "bre"]
categories = ["command-line-utilities", "text-editors"]
```

None of it is needed to build. All of it is read at release. crates.io refuses
a crate without a `license` or `license-file`, and without a `description`.
`keywords` is limited to five, all of which ved uses, and `categories` must be
slugs from crates.io's fixed list. `cargo publish --dry-run` checks all of that
without uploading, and `cargo package --list` shows the files that would be
uploaded, which is how to find out that a stray test fixture or a local note is
about to be published.

The `description` is the sentence most people will ever read about the
program. It appears in `cargo search`, on the crates.io page, and, through
`cargo-deb`, in `apt show`. It is worth writing as carefully as the README's
first paragraph, and it has the advantage of being shorter.

`version` is the only field that changes at every release, and the program
reads it at compile time with `env!("CARGO_PKG_VERSION")`, chapter 2's
`--version`. One place holds the number, and every artifact gets it from
there.

## 17.3 The field that is missing

One field that `Cargo.toml` does not have is `rust-version`, the minimum
supported Rust version. Without it, Cargo has no way to tell a user that their
compiler is too old except by failing to compile.

Chapter 1 said `edition = "2024"` sets the minimum compiler version to 1.85,
and the 0.2.1 README's build instructions say the same, *1.85+ for edition
2024*. Both statements are true of the edition. They are not true
of ved 0.2.1:

```text
$ cargo +1.87.0 build --release
error[E0658]: `let` expressions in this position are unstable
error[E0658]: `let` expressions in this position are unstable
$ cargo +1.88.0 build --release
$ echo $?
0
```

The two errors are `if ... && let` chains, which the 2024 edition allows from
Rust 1.88 onward. 0.2.0 builds on 1.87. So does the commit immediately before
the one that added continuous integration. The commit that added it does not,
and the next section explains why. Someone with Rust 1.85, 1.86, or 1.87 who
ran `cargo install ved` after September got a compile error from code they had
not written, with no indication that the fix was to upgrade. Filed as [ved
#28](https://github.com/excelano/ved/issues/28).

Declaring it is one line:

```toml
rust-version = "1.88"
```

With it, Cargo on an older toolchain says that the crate needs 1.88 instead of
showing an error from the middle of `main.rs`, and newer versions of Cargo use
it to pick a version of a dependency that the toolchain can build. The line is
only useful if it is true, which means checking it: the fleet's shared CI
workflow takes a `toolchain` input, so a second job that builds on the declared
version keeps the declaration true. Until it exists, the minimum version is
whatever the last refactor happened to need.

## 17.4 The gate

Releases 0.1.0 through 0.2.0 were built by a workflow that compiled and
packaged the program and ran nothing. The commit that changed that, in August,
says so:

```text
Nothing compiled a test between a laptop and a release: cargo-dist builds the
artifacts but runs nothing, so a broken commit could be tagged, published to
crates.io, and promoted to apt behind a green checkmark.
```

The check that arrived is three commands, in a workflow shared with the
author's other Rust repositories: `cargo fmt --check`, `cargo clippy` with
warnings denied, and `cargo test`. Sharing it is the decision that keeps it
maintained, since a fix to the workflow reaches every repository at once.
Denying warnings is the decision that keeps it meaningful, and the shared
workflow's header explains it: *a warning nobody has to fix is a warning nobody
fixes*.

A check that would have failed on arrival is not a check anyone keeps, so the
commit that added it also made the code pass. Three files were not formatted.
Clippy reported seven warnings, among them `collapsible_if`, the lint that
chapter 3 predicted would find its three-deep prompt:

```rust,ignore
        if !input_mode {
            if let Some(p) = prompt {
                if write!(out, "{p}").is_err() || out.flush().is_err() {
                    return ExitCode::FAILURE;
                }
            }
        }
```

`cargo clippy --fix` rewrote it as one condition:

```rust,ignore
        if !input_mode
            && let Some(p) = prompt
            && (write!(out, "{p}").is_err() || out.flush().is_err())
        {
            return ExitCode::FAILURE;
        }
```

That is clearer, and it is the version in `examples/ved`. It is also a let
chain, and it is why 0.2.1 needs Rust 1.88. The lint fixed the indentation and
raised the minimum compiler version in the same edit, and nothing reported the
second effect, because CI builds with the current stable compiler, which has
let chains. The cleanup was correct and the release that shipped it broke
`cargo install` for anyone three versions behind.

The lesson is not to avoid clippy. It is that a gate checks what it runs, and
this one runs the newest compiler. An automated fix is a code change like any
other, and its effects include the ones the tool does not mention.

## 17.5 Five targets

ved is built for five platforms. Doing that by hand means five machines, or
cross-compilers and linkers for each, and it is the step most likely to be done
once and never again. ved used `cargo-dist` for it, configured by one file:

```toml
[dist]
cargo-dist-version = "0.31.0"
ci = "github"
installers = ["shell", "powershell", "homebrew"]
tap = "excelano/homebrew-tap"
publish-jobs = ["homebrew"]
targets = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu",
           "x86_64-apple-darwin", "x86_64-unknown-linux-gnu",
           "x86_64-pc-windows-msvc"]
hosting = "github"
```

From that, `cargo-dist` generated a 343-line GitHub Actions workflow that runs
when a version tag is pushed. It builds each target on a runner of the right
operating system, archives each binary with the README and license, computes
checksums, generates a shell installer and a PowerShell installer that
download the right archive for the machine they run on, writes a Homebrew
formula and pushes it to a tap, and creates the GitHub release with all of it
attached. It also added a profile to `Cargo.toml`:

```toml
[profile.dist]
inherits = "release"
lto = "thin"
```

Link-time optimization shrinks the Linux binary from 526,744 bytes to 492,032.
The release build itself, measured on the machine this was written on, takes
0.54 seconds from clean, which is the preface's "half a second".

Chapter 1's design table says *single static binary, no runtime, no shared
libraries*. On Linux that is not quite what the build produces:

```text
$ ldd target/release/ved
	linux-vdso.so.1
	libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1
	libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
	/lib64/ld-linux-x86-64.so.2
```

Rust's standard library links the platform's C library dynamically on the
`-gnu` targets, so ved needs glibc and libgcc, which every Linux system with a
package manager has. The Debian package says so, with `depends = "libc6,
libgcc-s1"`. A binary with no shared libraries at all comes from the
`x86_64-unknown-linux-musl` target, which links a C library statically, and
which ved does not build. The design table states what the author wanted. The
`.deb` metadata states what is true, and it is the one a package manager reads.

## 17.6 What a release contains

The v0.2.1 release on GitHub has nineteen files:

| File | Bytes |
|---|---|
| `ved-x86_64-unknown-linux-gnu.tar.xz` | 186,796 |
| `ved-aarch64-unknown-linux-gnu.tar.xz` | 171,904 |
| `ved-x86_64-apple-darwin.tar.xz` | 178,832 |
| `ved-aarch64-apple-darwin.tar.xz` | 160,424 |
| `ved-x86_64-pc-windows-msvc.zip` | 164,455 |
| a `.sha256` for each of those five | |
| `sha256.sum` | 578 |
| `ved-installer.sh` | 53,752 |
| `ved-installer.ps1` | 21,482 |
| `ved.rb`, the Homebrew formula | 2,354 |
| `ved_0.2.1-1_amd64.deb` | 174,640 |
| `ved_0.2.1-1_arm64.deb` | 154,504 |
| `source.tar.gz` and its checksum | 45,639 |
| `dist-manifest.json` | 18,180 |

The shell installer is more than a quarter the size of the compressed program
it downloads. That is
not a criticism of it; it detects the platform, downloads, verifies, and puts
the binary on the `PATH`, which is a lot of shell. It is a reminder that the
code around a release is code, with its own bugs, and that it runs on machines
the author has never seen.

The checksums are for integrity, not authenticity. A checksum published on the
same page as the file it describes proves the download was not corrupted. It
does not prove that the page was not changed, since anyone who could replace
the archive could replace the checksum. Authenticity needs a signature made
with a key the attacker does not have, and ved's releases are not signed. The
Debian packages acquire a signature in chapter 18, from the apt repository's
key, which is the first point in the chain where someone could tell a real
`ved` from a substitute.

## 17.7 The release notes

The body of the v0.2.1 release on GitHub, generated by `cargo-dist`, is install
instructions and a table of downloads. It does not say what changed.
`cargo-dist` fills that section from a changelog file, and ved has none. A user
looking at the release page to decide whether to upgrade learns how to install
it and nothing else.

What changed is in the history:

```text
$ git log --reverse v0.2.0..v0.2.1 --format=%s -- src Cargo.toml
docs: state ed compatibility accurately, disclose AI assistance
Run the fleet's Rust CI on every push and PR
Take the command letter as a character, not a byte
Record only the nine groups a BRE can address
Release 0.2.1
```

The release notes for 0.2.1 are in there, and they are the second through
fourth lines, rewritten for someone who does not know what a command letter is:
*a non-ASCII character typed as a command no longer crashes ved*, and *a
pattern with more than nine groups no longer crashes ved*. The first line is
about the README, the second is about CI, and the last is the release itself.
Twenty-three other commits between the two tags touched only workflows and
documentation.

This is why chapter 16 said subject lines should be written for the person
installing the release. The history is the only complete record of what
changed, and the notes have to be made from it, by hand or by a tool. A tool
can only do it if the subjects say which kind of change each one is. The
convention ved's later history adopted is Conventional Commits, a type before
the subject, `fix:` and `feat:` for changes a user would notice and `docs:`,
`ci:`, `build:`, and `chore:` for everything else. With it, the release notes
are the `fix:` and `feat:` subjects since the last tag, and the rest stay out.
The tool that produced 0.3.0's release notes worked that way, and most of the
subjects since 0.2.1 had no type. The few that did, `docs:`, `build:`, and
`chore:`, were left out. The rest could not be sorted, so all of them went in,
and *Support /pattern/ and ?pattern? search addresses* sits in the same list as
*Dependabot merges move to the organisation's sweep*, in no order a user would
choose.

## 17.8 A tag is spent

A pushed tag can be deleted and pushed again, and Git will allow it. Nothing
downstream will. crates.io never accepts the same version twice, and yanking a
version only stops new projects from selecting it; the number stays used. A
`.deb` with a version that is already in an apt repository is refused or, worse,
replaces the old one under the same name for some users and not others. A
WinGet manifest for a version that has been submitted is a duplicate.

So a release is irreversible, in practice, once any channel has it, and the
response to a bad release is the next patch number. That changes how the steps
before the tag should be treated. Everything that can be checked should be
checked before the tag is pushed, because after it the only fix is another
release: the tests, `cargo publish --dry-run`, a build of each package, and,
for ved, the check that `RELEASING.md` named as the real risk:

```text
**ved is a clone of `ed`, so compatibility is the release risk.** The tests
cover the command grammar, but the thing that breaks quietly is a real `ed`
script behaving differently under ved.
```

That is chapter 15's differential harness, stated as a release step before
the harness existed.

## 17.9 The order of operations

At 0.2.1 a release went like this. Bump `version` in `Cargo.toml` and commit.
Tag the commit and push the tag. The tag starts two workflows at once: the
`cargo-dist` release, which builds and creates the GitHub release, and
`publish-crate.yml`, which runs `cargo publish` if a crates.io token is
available and skips cleanly if not. Then dispatch the Debian package workflow by
hand, for a reason its comment records:

```text
# cargo-dist creates the release with the default GITHUB_TOKEN and GitHub fires
# no `release: published` for a token-created release, so this is dispatched by
# hand after one:
#
#     gh workflow run deb.yml -f tag=v0.2.1
```

GitHub does not let events caused by a workflow's own token start other
workflows, to prevent loops, so a workflow triggered by "a release was
published" never fires for a release another workflow published. The
`workflow_dispatch` trigger is the escape hatch, and the manual step is the
cost. Then promote the `.deb` files into the apt repository, and submit the
WinGet manifest. Those two are chapter 18.

Every step after the tag has a different owner: GitHub's runners, crates.io,
the apt repository, Microsoft's package repository. A release is a sequence of
handoffs, and the procedure is the list of them in order with what to check at
each one. ved's `RELEASING.md` held the parts specific to ved and pointed to a
shared document for the rest, and chapter 19 is about what happened when that
procedure moved out of the repository entirely.

## 17.10 What release bought

A program that anyone can install on five platforms without a Rust toolchain,
in a form that identifies its version, from a page that lists every file and
its checksum. The mechanism is a tag, a manifest that was correct from the
first day, and a generated workflow.

It also put the program in front of people whose machines are not the author's,
and that is when the gaps between what the project says and what it does start
to cost something. The minimum Rust version was whatever the last cleanup
needed. The binary was not as static as the design said. The release page did
not say what had changed. Each of those is a fact a user needed and could not
get, and each is one line in a file, written down once.

---

**Exercise 17-1.** Add `rust-version` to `examples/ved/Cargo.toml`. Install the
toolchain it names with `rustup toolchain install`, build with it, and then
find the oldest toolchain that builds `examples/ch05-write`. Explain the
difference.

**Exercise 17-2.** Build ved for `x86_64-unknown-linux-musl`, check it with
`ldd`, and compare its size with the `-gnu` build. Decide which one the
Debian package should contain, and why it is not obvious.

**Exercise 17-3.** Write the release notes for every release from 0.1.0 to
0.2.1 from the history alone, one sentence per user-visible change, written
for someone who has never seen the code. Then write the rule that would let a
tool produce your list.

**Exercise 17-4.** Run `cargo package --list` in `examples/ved` and in the
workspace root. Decide what should not be in each, and add the `include` or
`exclude` field that keeps it out.

**Exercise 17-5.** Write the pre-tag checklist for a program you maintain: the
checks that can still be fixed before the version number is spent. For each
one, decide whether a machine can run it, and if so, why it is on a list
instead of in CI.
