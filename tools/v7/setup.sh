#!/bin/bash
# Build the SIMH PDP-11 simulator and fetch the Seventh Edition Unix disk
# that v7ed.py boots. Everything lands next to this script.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

set -e
cd "$(dirname "$0")"
[ -d simh-src ] || git clone --depth 1 https://github.com/open-simh/simh.git simh-src
make -C simh-src pdp11
mkdir -p kit
[ -f kit/uv7swre.zip ] || curl -fsSL -o kit/uv7swre.zip http://simh.trailing-edge.com/kits/uv7swre.zip
python3 -I -c "import zipfile; zipfile.ZipFile('kit/uv7swre.zip').extractall('kit/x')"
ls -l simh-src/BIN/pdp11 kit/x/unix_v7_rl.dsk
