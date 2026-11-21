// Fuzz test for ved's dispatcher. Append to a copy of src/main.rs and run
//
//     RUNS=1000000 cargo test --release fuzz_dispatch_never_panics -- --nocapture
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#[cfg(test)]
mod fuzz {
    use super::*;

    fn buf_from(lines: &[&str]) -> Buffer {
        let mut b = Buffer::new();
        for (i, l) in lines.iter().enumerate() {
            b.append_after(i, (*l).to_string());
        }
        b
    }

    #[test]
    fn fuzz_dispatch_never_panics() {
        let alphabet: Vec<char> = "0123456789$.,;+-/\\[]()*^&gvsdpnljmtaciqQH \té×".chars().collect();
        let lines = ["a", "é", "", "aéb", "ab", "\\"];
        let mut seed: u64 = 0x9E3779B97F4A7C15;
        let mut next = move || { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; seed };
        let mut found: Vec<(String, String)> = Vec::new();
        let runs: usize = std::env::var("RUNS").ok().and_then(|s| s.parse().ok()).unwrap_or(200_000);
        std::panic::set_hook(Box::new(|_| {}));
        for _ in 0..runs {
            let n = (next() % 5) as usize;
            let buf_lines: Vec<&str> = (0..n).map(|_| lines[(next() % lines.len() as u64) as usize]).collect();
            let mut buf = buf_from(&buf_lines);
            let k = 1 + (next() % 4) as usize;
            let cmds: Vec<String> = (0..k).map(|_| {
                let len = 1 + (next() % 10) as usize;
                (0..len).map(|_| alphabet[(next() % alphabet.len() as u64) as usize]).collect()
            }).collect();
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for c in &cmds { dispatch(c, &mut buf); }
            }));
            if let Err(e) = r {
                let msg = e.downcast_ref::<String>().cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                let key: String = msg.chars().take(70).collect();
                if !found.iter().any(|(k, _)| *k == key) {
                    found.push((key, format!("{cmds:?} on {buf_lines:?}")));
                }
            }
        }
        let _ = std::panic::take_hook();
        for (k, ex) in &found { eprintln!("PANIC {k}\n   e.g. {ex}"); }
        eprintln!("{} distinct panics in {runs} runs", found.len());
    }
}
