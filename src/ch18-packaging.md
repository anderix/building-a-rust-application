# 18. Packaging

A release is a set of files on GitHub. Almost nobody installs software by
downloading a file from GitHub. They type a command into a package manager they
already have, and they expect the same package manager to tell them when there
is a new version. Packaging is the work of putting a release into the shapes
those programs expect, and then putting it there again for every release after.

At 0.2.1, ved's README lists seven ways to install it:

| Channel | The user types | Built by | First version |
|---|---|---|---|
| Debian and Ubuntu | `sudo apt install ved` | `cargo-deb`, in CI | 0.1.2 |
| Homebrew | `brew install ved` | `cargo-dist` | 0.2.0 |
| curl installer | `curl ... install.sh \| sh` | `cargo-dist` | 0.1.0 |
| WinGet | `winget install Excelano.ved` | `cargo-dist`, then a manifest | 0.1.5 |
| PowerShell installer | `irm ... \| iex` | `cargo-dist` | 0.1.0 |
| crates.io | `cargo install ved` | the user's own compiler | 0.2.0 |
| Source | `make build && sudo make install` | the user's own compiler | 0.1.0 |

Each row is a separate product with its own users, its own update mechanism,
its own failure modes, and its own copy of what ved is called and what it does.
This chapter goes through them in order of how much they ask of the person
installing, from most to least, and ends with what having seven of them cost.

## 18.1 From source

```makefile
PREFIX ?= /usr/local

.PHONY: build install uninstall clean test

build:
	cargo build --release

test:
	cargo test

install:
	@test -f target/release/ved || { echo "Run 'make build' first."; exit 1; }
	install -d $(PREFIX)/bin
	install -m 755 target/release/ved $(PREFIX)/bin/ved

uninstall:
	rm -f $(PREFIX)/bin/ved

clean:
	cargo clean
```

A Makefile for a Rust program looks redundant, since `cargo build` does the
building. It is there for the other half: `sudo make install` is what a Unix
user expects to type to put a program in `/usr/local/bin`, and `PREFIX=/usr`
is what a distribution packager expects to be able to pass.

The `install` target does not build. The first version did, and it failed on
the first machine it was tried on, for a reason its fix records: *sudo make
install was failing because root doesn't have cargo in its PATH*. Cargo is
installed per user, under `~/.cargo`, and `sudo` runs as root with root's
environment. Separating the two targets means building as yourself and copying
as root, which is also the safer arrangement, because a build runs code from
the crate and its build scripts and should not run with privileges it does
not need. The guard line refuses to install a binary that has not been built
rather than letting `install` fail with a less useful message.

## 18.2 crates.io

`cargo install ved` downloads the source of the latest version from crates.io,
compiles it on the user's machine, and puts the binary in `~/.cargo/bin`. It
is the easiest channel to publish to, since a release is `cargo publish`, and
the most demanding to install from, since it needs a working Rust toolchain of
the right version. Chapter 17's missing `rust-version` is a bug in this
channel and no other: every other channel ships a binary someone else
compiled.

What is uploaded is the crate's source, as `cargo package` assembles it, and
the version on crates.io is permanent. ved's first crates.io version is 0.2.0;
the six releases before it never went there. That is visible on the crate's
page and is a fair summary of how channels arrive in a project: one at a time,
each when somebody asked for it.

## 18.3 Piping a script into a shell

```sh
curl -fsSL https://raw.githubusercontent.com/excelano/ved/main/install.sh | sh
```

The curl installer is the channel with the least friction for a user who has
no package manager configured, and the most trust. The script runs with the
user's permissions, it is fetched at the moment it is run, and the user has not
read it. Everything a package manager does to establish that a package is what
it claims to be, the signed index, the pinned key, the recorded checksum, is
replaced by trusting the server and the TLS connection to it.

ved's `install.sh` is one command that fetches and runs a second script, the
installer `cargo-dist` generates for each release, which detects the platform,
downloads the right archive, checks its checksum against the one published
beside it, and installs to `~/.cargo/bin`. The shim exists so that installing
and uninstalling have the same shape of URL, and `uninstall.sh` removes the
binary from where the installer put it.

The history of that shim is short and has four entries. It was added on the day
of the first release. It was removed the same day, by a commit whose message
says *per-language idiom wins over cross-portfolio URL symmetry*. It was
restored six weeks later to match the author's other projects. And it was
retired just after 0.2.1, with the message *one less thing to maintain*. Four
commits, two positions, each defensible, and none of them informed by how many
people used it, because nothing measured that. A channel is cheap to add and
its cost is ongoing, and without a way to tell whether anyone uses it, the
decision to keep it is a matter of taste that can be re-argued indefinitely.

## 18.4 Homebrew

```ruby
class Ved < Formula
  desc "The verbose ed — an ed clone with friendly errors and a built-in help system"
  homepage "https://excelano.com/ved/"
  version "0.2.1"
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/excelano/ved/releases/download/v0.2.1/ved-aarch64-apple-darwin.tar.xz"
      sha256 "4297e83fd811e228e64ab85e2f0c564d369408dd63fc56c30ba8cee5c9277a64"
    end
    # ... Intel macOS, then ARM and Intel Linux ...
  end
  license "MIT"
  # ...
  def install
    # ...
    bin.install "ved"
  end
end
```

A Homebrew formula is a Ruby class that says where to get a program and how to
install it. Homebrew's own formulae usually build from source, or install a
prebuilt "bottle" that Homebrew's servers built. This one does neither: it
points straight at the release archives on GitHub and checks each against a
SHA-256 written into the formula. `cargo-dist` generates it at release and
pushes it to a tap, `excelano/homebrew-tap`, which is a Git repository of
formulae that users add with `brew tap`.

The checksum in the formula is the first point in this chapter where a
checksum means something. Chapter 17 said a checksum published beside its file
proves only that the download was not corrupted. This one is in a different
repository, committed when the release was made, so replacing the archive on
GitHub without also changing the tap makes `brew install` fail. That is not a
signature, since whoever controls both repositories can change both, but it
moves the trust from "whoever can upload a release asset" to "whoever can push
to the tap".

## 18.5 Debian packages and an apt repository

A `.deb` is an archive of files with their destinations, plus a control file
that tells `dpkg` what the package is called, what version it is, and what it
needs. `cargo-deb` builds one from `Cargo.toml`, using the package metadata and
a section of its own:

```toml
[package.metadata.deb]
maintainer = "..."
extended-description = """
ved is a verbose clone of ed, the original Unix line editor, written in
pure-stdlib Rust. ..."""
section = "editors"
priority = "optional"
depends = "libc6, libgcc-s1"
assets = [
    ["target/release/ved", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/ved/README.md", "644"],
    ["LICENSE", "usr/share/doc/ved/copyright", "644"],
]
```

and the result is three files:

```text
$ dpkg-deb -c ved_0.2.1-1_amd64.deb
./usr/share/doc/ved/copyright
./usr/share/doc/ved/README.md
./usr/bin/ved
```

`depends` is chapter 17's `ldd` output written down, and it is correct. The
version has a `-1` after it, the Debian revision, which counts packagings of
the same upstream version and is what lets a packaging fix ship without
pretending the program changed.

Two things a Debian user expects are missing. There is no manual page. Debian's
policy is that every program in `/usr/bin` has one, and a user who types `man
ved` after installing is told there is no entry, while `man ed` works. The
README is in `/usr/share/doc/ved`, which is where Debian puts documentation and
where almost nobody looks. And `copyright` is the MIT license text, which is
what the file must contain at minimum, rather than the machine-readable format
Debian's own packages use. Neither is an error for a third-party repository.
The first is a gap in the program's documentation, of the kind chapter 16
counted: the README, the tutorial, the help, and `--help` are four places that
describe ved, and the one place Unix users look is the one that is empty.

The `.deb` files are attached to the GitHub release, and a user could download
and `dpkg -i` one. That is not what makes them useful. What makes them useful is
an apt repository: a directory served over HTTPS containing the packages and an
index of them, with the index signed. A user adds it once:

```sh
curl -fsSL https://excelano.com/apt/setup.sh | sudo sh
```

and the script does three things: fetches the repository's public key into
`/usr/share/keyrings/excelano-archive-keyring.gpg`, writes a source line that
names that key,

```text
deb [signed-by=/usr/share/keyrings/excelano-archive-keyring.gpg] https://excelano.com/apt stable main
```

and runs `apt-get update`. From then on `sudo apt install ved` works, and `apt
upgrade` brings each new version without the user doing anything.

This is the point in the chain where authenticity arrives. apt refuses an index
whose signature does not verify against the key, and every package's checksum
is in the signed index, so a substituted `ved` fails to install unless the
attacker also has the private key. The `signed-by` option limits the key to
this one source, so the Excelano key cannot vouch for packages that claim to
come from Debian, which is the reason that form replaced the older global
`apt-key` mechanism. The weak point is the first step: the key itself arrives
over HTTPS by `curl | sudo sh`, so the user is trusting the server once, at
setup, and the signature from then on.

The repository keeps the current version of each package and the one before
it, so a bad release can be rolled back with `apt install ved=<version>`.
That is the apt answer to chapter 17's spent tags: the version number cannot be
reused, but the previous one is still there to go back to.

## 18.6 WinGet

WinGet is Windows' package manager, and its packages are manifests in a single
public Git repository, `microsoft/winget-pkgs`. Publishing a version means
opening a pull request that adds three YAML files under
`manifests/e/Excelano/ved/0.2.1/`. The installer manifest says where the
program is and how to install it:

```yaml
PackageIdentifier: Excelano.ved
PackageVersion: 0.2.1
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
- RelativeFilePath: ved.exe
  PortableCommandAlias: ved
ReleaseDate: 2026-09-15
Installers:
- Architecture: x64
  InstallerUrl: https://github.com/excelano/ved/releases/download/v0.2.1/ved-x86_64-pc-windows-msvc.zip
  InstallerSha256: 77B0DB94E95349E05097A7853B56947E20E2C0E67B96F9AD0EE124DCCD71253F
```

A portable package is an executable with no installer of its own. WinGet
unpacks the zip, puts `ved.exe` where it keeps portable programs, and adds the
alias `ved` to the user's path. The checksum is in the manifest, which is in
Microsoft's repository, so it is the Homebrew arrangement again, with a third
party holding the copy.

The pull request is reviewed by automation before a person sees it: the
installer is downloaded, scanned, and installed in a sandbox, and the program
is run. That pipeline is the part of this channel the author does not control,
and ved's history has two traces of it. A workflow was added to scan the
release with Microsoft Defender before submitting, so that a rejection could be
predicted, and was deleted a month later with the message *the scan never
predicted winget's validation*. And `RELEASING.md` recorded that WinGet's
validation reports an executable error for ved by design, with a recipe for
getting past it, and an instruction not to change the program to avoid it. Both
are what a channel with a gatekeeper looks like from the outside: a check that
cannot be run locally, and a list of things learned about it the hard way.

The second manifest, the locale file, holds the descriptive text, and it has a
problem that is not WinGet's:

```yaml
PackageName: ved
ShortDescription: The verbose ed, a drop-in ed clone with friendly errors and a built-in help system
```

*A drop-in ed clone.* Chapter 16 told how the README's claim of strict drop-in
compatibility was corrected to *close compatibility so most scripts written for
real ed run against ved unchanged*, in July. The WinGet manifests for 0.1.5, 0.2.0, 0.2.1, and 0.3.0 all say *drop-in*,
because the description was written into the manifest when the package was
first submitted and has been carried forward with every version since. The correction reached the
README, the crate description, and the Debian package. It did not reach the
channel whose metadata lives in somebody else's repository.

The same locale file's `ReleaseNotes` field is the body of the GitHub release,
which chapter 17 found was install instructions, so `winget show Excelano.ved`
tells a Windows user how to install ved with `curl | sh` and Homebrew.

## 18.7 Six descriptions

Count the places ved's one-sentence description lives at 0.2.1. There are six.
`Cargo.toml`'s `description`, which crates.io shows. The `extended-description`
in the same file, which `apt show` shows. The Homebrew formula's `desc`,
generated from `Cargo.toml` at each release. The WinGet locale manifest's
`ShortDescription`, copied forward by hand. The README's first paragraph. The
tutorial's first paragraph.

Three of those are `Cargo.toml` or generated from it, and stay in step with it.
The rest are copies, and a copy is correct until the original changes. Chapter
16's rule, state each fact once, applies to packaging with more force than to
documentation, because the copies are in places the author does not see every
day, and some of them are in repositories the author does not own. The repair
is to generate every channel's metadata from one source at release time, the
way the Homebrew formula already is, so that changing the description in
`Cargo.toml` changes it everywhere at the next release.

## 18.8 What a channel costs

Each row of the table at the top of this chapter is, at every release: an
artifact to build, a step to run, a credential that can expire, a reviewer or a
validation that can reject it, and a set of users who expect the new version to
arrive. The work is not in setting a channel up, which is an afternoon. It is
in the nine releases after.

At 0.2.1, ved paid that for seven channels and five platforms, and it had no
measure of which were used. The next chapter is about what happened when the
cost was counted: by 0.3.0, the curl installer, the Homebrew formula, the macOS
builds, and `cargo-dist` itself were gone, and each removal is a commit with a
one-line reason. The general lesson is to add a channel when someone needs it,
to know how you would tell whether they still do, and to treat removing one as
a normal release decision rather than a failure.

## 18.9 What packaging bought

A program that installs with the command its users already know, on Debian,
Ubuntu, macOS, and Windows, and that updates with the command they already run.
A signed path from the build to the user's disk, for the users of the apt
repository. And on every channel, a copy of the program's name, its
description, and its promises, held by a system that does not know when the
original changes.

---

**Exercise 18-1.** Write a manual page for ved in `man(7)` format, add it to
the `.deb` assets so it installs as `/usr/share/man/man1/ved.1.gz`, and check
it with `man -l`. Then decide what should be in it that is not in `H`, and what
should be in `H` that is not in it.

**Exercise 18-2.** Build the `.deb` with `cargo deb` and run `lintian` on it.
Sort what it reports into what a third-party repository should fix and what it
can ignore, and say why for each.

**Exercise 18-3.** Generate the WinGet locale manifest from `Cargo.toml` with a
script, so that the description cannot drift. Decide which fields come from
`Cargo.toml`, which from the release, and which are written once by hand.

**Exercise 18-4.** Write the Makefile's `install` target again so that it also
installs the manual page from exercise 18-1, and an `uninstall` that removes
both. Check that `make install PREFIX=/tmp/ved-test` works without `sudo`.

**Exercise 18-5.** Pick a program you maintain and list its channels. For each,
write down how you would find out how many people installed from it last month.
If you cannot, decide whether that is a reason to remove it.
