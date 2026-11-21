# Tools

The programs the book used to check ved, so you can run them yourself.
Chapters 15, 20 and 21 describe what each one found.

**`difftest.py`** is the differential harness. It generates random ed
scripts from a small grammar, runs each through ved and through a reference
ed on the same six-line file, and counts the scripts that leave a different
file, grouped by the first command:

```text
python3 tools/difftest.py path/to/ved 3000 1        # 3,000 scripts, seed 1
MAXC=1 python3 tools/difftest.py path/to/ved 3000 1 # one command per script
python3 tools/difftest.py path/to/ved 3000 1 12     # and show 12 examples
```

The reference is `/usr/bin/ed` unless `ED` names another. It works in `v/`
and `e/` under the current directory. A difference is something to read,
not a failure: the reference can be the one that is wrong.

**`fuzz_test.rs`** is a test that throws random command lines at
`dispatch` and reports every distinct panic. Append it to a copy of ved's
`src/main.rs` and run it with
`RUNS=1000000 cargo test --release fuzz_dispatch_never_panics -- --nocapture`.

**`bugs-by-release.sh`** runs the reproduction for each issue the book
filed against one ved binary and prints `BUG` or `ok` for each, which is
how chapter 20 measured how long each bug shipped. Build every release
from its tag and run it on each.

**`v7/`** runs ed scripts on Seventh Edition Unix, the 1979 ed, under the
SIMH PDP-11 simulator. `tools/v7/setup.sh` builds SIMH and fetches the
Unix V7 disk kit from the SIMH archive, once, into `tools/v7/`; the kit is
distributed under its own license, which it includes. Then
`python3 tools/v7/v7ed.py tools/v7/cases.json` boots the disk and runs each
case: a file's content and an ed script, either typed at the console or
fed from a file. `cases.json` holds the cases chapter 21 cites.

**`vbench/`** times a thousand insertions at the top and at the end, and a
thousand deletions at the top, of a `Vec<String>` buffer like ved's with a
hundred thousand and a million lines: `cargo run --release` in that
directory.
