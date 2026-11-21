"""Differential test: random ed scripts through ved and a reference ed.

    python3 tools/difftest.py path/to/ved RUNS SEED [EXAMPLES]

Each script runs on the same six-line file in both editors, ends with
`w out.txt` and `Q`, and the two written files are compared. MAXC sets
the most commands per script (default 3) and ED the reference editor
(default /usr/bin/ed).
"""
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)
import random, subprocess, sys, os, collections
ved = sys.argv[1]; runs = int(sys.argv[2]); seed = int(sys.argv[3])
rng = random.Random(seed)
MAXC = int(os.environ.get("MAXC", "3"))
ED = os.environ.get("ED", "/usr/bin/ed")
START = "a1\nb2\na3\nb4\na5\nab6\n"
addrs = ["", "1", "2", "3", "$", ".", "0", "+1", "-1", "9"]
def rng_range():
    k = rng.random()
    if k < 0.3: return ""
    if k < 0.6: return rng.choice(addrs[1:])
    if k < 0.9: return rng.choice(addrs[1:]) + "," + rng.choice(addrs[1:])
    return rng.choice([",", ";"])
CMDS = ["p", "n", "d", "j", "m{a}", "t{a}", "s/a/X/", "s/a/X/g", "s/^a/X/g",
        "s/$/!/g", "s/[ab]*/-/g", "s/\\(a\\)\\(.\\)/\\2\\1/", "s.a\\..X.", "s/a/X",
        "g/a/d", "g/a/j", "g/a/m0", "g/b/-1d", "v/a/d", "g/a/s/a/A/", "g/a/.,+1d",
        "a\nNEW\n.", "i\nNEW\n.", "c\nNEW\n.", "s"]
def command():
    c = rng.choice(CMDS).replace("{a}", rng.choice(addrs[1:]))
    return rng_range() + c
def run(exe, script, d):
    with open(os.path.join(d, "in.txt"), "w") as f: f.write(START)
    out = os.path.join(d, "out.txt")
    if os.path.exists(out): os.remove(out)
    subprocess.run([exe, "in.txt"], input=(script + "\nw out.txt\nQ\n").encode(), cwd=d,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5)
    return open(out, "rb").read() if os.path.exists(out) else None
os.makedirs("v", exist_ok=True); os.makedirs("e", exist_ok=True)
bad = collections.Counter(); total = collections.Counter(); examples = {}
for _ in range(runs):
    cmds = [command() for _ in range(rng.randint(1, MAXC))]
    script = "\n".join(cmds)
    a, b = run(ved, script, "v"), run(ED, script, "e")
    key = cmds[0].split("\n")[0]
    key = key.lstrip("0123456789$.,;+-") or "(address only)"
    total[key] += 1
    if a != b:
        bad[key] += 1
        examples.setdefault(key, (script, a, b))
print(f"{sum(bad.values())} of {runs} scripts left a different file")
for k in sorted(bad, key=lambda k: -bad[k]):
    print(f"  {bad[k]:4} / {total[k]:4}  first command {k!r}")
if len(sys.argv) > 4:
    for k, (s, a, b) in list(examples.items())[:int(sys.argv[4])]:
        print("---", repr(s)); print("ved:", a); print("ed: ", b)
