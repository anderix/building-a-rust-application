// Time insertions and deletions at the top and the end of a ved-style
// Vec<String> buffer of a hundred thousand and a million lines.
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![allow(dead_code)]
mod buffer;
use buffer::Buffer;
use std::time::Instant;

fn filled(n: usize) -> Buffer {
    let mut b = Buffer::new();
    for i in 0..n {
        b.append_after(i, format!("line {i} of a file that is long enough to matter"));
    }
    b
}

fn main() {
    for &n in &[100_000usize, 1_000_000] {
        let mut b = filled(n);
        let t = Instant::now();
        for _ in 0..1000 { b.append_after(0, "x".to_string()); }
        let front = t.elapsed();
        let mut b = filled(n);
        let t = Instant::now();
        for _ in 0..1000 { let len = b.len(); b.append_after(len, "x".to_string()); }
        let back = t.elapsed();
        let mut b = filled(n);
        let t = Instant::now();
        for _ in 0..1000 { b.delete_range(1, 1); }
        let del = t.elapsed();
        println!("{n} lines: 1000 inserts at top {:?}, at end {:?}, 1000 deletes at top {:?}", front, back, del);
    }
}
