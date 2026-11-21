#!/bin/bash
# For each ved issue the book found, run its reproduction against one ved
# binary and print BUG if the binary still has it, ok if not.
#
#   tools/bugs-by-release.sh path/to/ved
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

ved=$(realpath "$1")
work=${WORK:-${TMPDIR:-/tmp}/ved-bugs}

t() {
  local d="$work/$1"
  rm -rf "$d"; mkdir -p "$d"; cd "$d" || exit 1
  printf "$2" > f.txt
  printf "$3" > script
  timeout 5 "$ved" f.txt < script > out 2>&1
  echo $? > status
  if eval "$4"; then r=BUG; else r=ok; fi
  printf "%-4s %s\n" "$1" "$r"
  cd /; rm -rf "$d"
}

t 14 'one\n' 's/one/ONE/\nwq\n' '[ -e q ]'
t 15 'one\n' 'write x\nq\n' '[ -e "rite x" ]'
t 17 'one\n' 'a\nADDED\n.\ne f.txt\nw g.txt\nQ\n' '! grep -q ADDED g.txt 2>/dev/null'
t 18 'one\r\ntwo\r\n' 'w\nq\n' '[ "$(wc -c < f.txt)" = 8 ]'
t 19 'x\na1\ny\na2\n' 'g/a/-1d\nw\nq\n' '! grep -q a2 f.txt'
t 20 'one\ntwo\n' '2\n0a\nNEW\n.\nw\nq\n' '[ "$(head -1 f.txt)" != NEW ]'
t 21 'a.b\naxb\n' '1,2s.a\\..X.\nw\nq\n' '[ "$(sed -n 2p f.txt)" != axb ]'
t 22 'aaa\n' 's/^a/X/g\nw\nq\n' '[ "$(cat f.txt)" = XXX ]'
t 23 'abc\n' 's/a/b\nw\nq\n' '[ "$(cat f.txt)" = abc ]'
t 24 'aaa\n' 's/a/X/2\nw\nq\n' '[ "$(cat f.txt)" != aXa ]'
t 25 'see +ACI- here\n' 's/see/saw/\nw\nQ\n' 'grep -q see f.txt'
t 26 'one\n' 'zz\nq\n' '[ "$(cat status)" = 0 ]'
t 32 'ab\n' 's/a/X/\ns/b/%%/\nw\nq\n' '[ "$(cat f.txt)" != XX ]'
t 33 'one\n' 'a\nX\n.\n' '! grep -q warning out'
t 34 'a1\nb2\na3\nb4\n' 'g/a/d\ns/$/!/\nw\nq\n' '[ "$(head -1 f.txt)" = "b2!" ]'
t 35 'a1\nb2\na3\n' '2j\ns/$/!/\nw\nq\n' '[ "$(sed -n 2p f.txt)" = "b2!" ]'
