"""Run ed scripts on Seventh Edition Unix under SIMH.

    tools/v7/setup.sh                       # once: build SIMH, fetch the disk
    python3 tools/v7/v7ed.py tools/v7/cases.json

Boots a copy of the V7 RL02 disk to the single-user shell, then for each
case writes the file, runs `ed f` with the script on standard input, and
prints ed's output, its exit status, and `od -c` of the file afterwards.
A case with "tty": true types the script at ed on the console instead,
which is how the commands were meant to be used: from a file, Seventh
Edition ed drops the rest of the script after the first error.
In content, the character `~` stands for a carriage return. The console
is typed a character at a time, and the boot sets erase and kill to
control characters, because V7's defaults are # and @.
"""
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

import json, os, pty, select, shutil, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
SIM = os.path.join(HERE, "simh-src", "BIN", "pdp11")
DISK = os.path.join(HERE, "kit", "x", "unix_v7_rl.dsk")
WORK = os.path.join(HERE, "run")


class Console:
    def __init__(self):
        os.makedirs(WORK, exist_ok=True)
        shutil.copy(DISK, os.path.join(WORK, "v7.dsk"))
        with open(os.path.join(WORK, "boot.ini"), "w") as f:
            f.write("set cpu 11/45\nset rl0 rl02\nattach rl0 v7.dsk\nboot rl0\n")
        self.master, slave = pty.openpty()
        self.proc = subprocess.Popen([SIM, "boot.ini"], cwd=WORK, stdin=slave,
                                     stdout=slave, stderr=slave, close_fds=True)
        os.close(slave)
        self.buf = b""

    def read_until(self, token, timeout=60):
        token = token.encode()
        end = time.time() + timeout
        while token not in self.buf:
            left = end - time.time()
            if left <= 0:
                raise TimeoutError(f"waiting for {token!r}; got {self.buf[-400:]!r}")
            r, _, _ = select.select([self.master], [], [], left)
            if r:
                self.buf += os.read(self.master, 4096)
        i = self.buf.index(token) + len(token)
        out, self.buf = self.buf[:i], self.buf[i:]
        return out.decode("latin-1")

    def read_prompt(self, timeout=60):
        end = time.time() + timeout
        while not (self.buf.endswith(b"\n# ") or self.buf.endswith(b"\n> ")
                   or self.buf == b"# " or self.buf == b"> "):
            left = end - time.time()
            if left <= 0:
                raise TimeoutError(f"waiting for a prompt; got {self.buf[-400:]!r}")
            r, _, _ = select.select([self.master], [], [], min(left, 0.5))
            if r:
                self.buf += os.read(self.master, 4096)
        out, self.buf = self.buf, b""
        return out.decode("latin-1")

    def send(self, text):
        for ch in text.encode():
            os.write(self.master, bytes([ch]))
            time.sleep(0.002)

    def line(self, text):
        self.send(text + "\r")


def boot():
    c = Console()
    c.read_until("@")
    c.line("boot")
    c.read_until(":")
    c.line("rl(0,0)rl2unix")
    c.read_until("# ")
    c.line("stty erase \\^h kill \\^u")
    c.read_until("# ")
    for setup in ["mkdir /tmp", "chmod 777 /tmp", "cd /tmp"]:
        c.line(setup)
        c.read_prompt()
    return c


def command(c, text):
    c.line(text)
    out = c.read_prompt()
    out = out.replace("\r\n", "\n").replace("\r", "")
    first, _, rest = out.partition("\n")
    return rest.rsplit("\n", 1)[0] if "\n" in rest else ""


def drain(c, quiet=0.6):
    while True:
        r, _, _ = select.select([c.master], [], [], quiet)
        if not r:
            break
        c.buf += os.read(c.master, 4096)


def interactive(c, script):
    c.line("ed f")
    for l in script.rstrip("\n").split("\n"):
        drain(c)
        c.line(l)
    drain(c, 1.5)
    if not c.buf.endswith(b"\n# "):
        c.line("Q")
    out = c.read_prompt().replace("\r\n", "\n").replace("\r", "")
    return out.partition("\n")[2].rsplit("\n", 1)[0]


def run(c, case):
    content = case["content"]
    script = case["script"]
    if content:
        command(c, "cat > f << 'EOT'")
        for l in content.rstrip("\n").split("\n") + ["EOT"]:
            command(c, l)
    else:
        command(c, "cat /dev/null > f")
    if "~" in content:
        command(c, "tr '~' '\\015' < f > g")
        command(c, "mv g f")
    command(c, "cat > s << 'EOT'")
    for l in script.rstrip("\n").split("\n") + ["EOT"]:
        command(c, l)
    if case.get("tty"):
        output = interactive(c, script)
        status = "tty"
    else:
        output = command(c, "ed f < s")
        status = command(c, "echo $?")
    od = command(c, "od -c f")
    return {"name": case["name"], "output": output, "status": status.strip(), "od": od}


def main():
    cases = json.load(open(sys.argv[1]))
    c = boot()
    for case in cases:
        r = run(c, case)
        print(f"### {r['name']}  (exit {r['status']})")
        for l in r["output"].split("\n"):
            print("  | " + l)
        print("  file:")
        for l in r["od"].split("\n"):
            print("    " + l)
    c.line("sync")
    c.proc.kill()


if __name__ == "__main__":
    main()
