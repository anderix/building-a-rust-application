// The text buffer that ved edits.
//
// Lines are 1-indexed externally, matching ed's address syntax: line
// 1 is the first line, $ is the last, and 0 is a sentinel meaning
// "before the first line" (which is what `a` on an empty buffer
// effectively writes against).
//
// The `current` field tracks ed's "current line" concept — the line
// that most commands default to when no address is given. After most
// operations current points at the last line touched.

pub struct Buffer {
    lines: Vec<String>,
    current: usize, // 0 means empty buffer / before-first-line
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn current(&self) -> usize {
        self.current
    }

    /// Set the current line. Used by commands like `p` to match ed's
    /// behavior of leaving the current line at the end of whatever
    /// range they touched.
    pub fn set_current(&mut self, n: usize) {
        self.current = n;
    }

    /// Insert `line` after 1-indexed position `after`. `after = 0`
    /// means insert at the very start of the buffer (the case when
    /// you run `a` on an empty buffer). Updates current to the
    /// newly inserted line.
    pub fn append_after(&mut self, after: usize, line: String) {
        self.lines.insert(after, line);
        self.current = after + 1;
    }

    /// Fetch a line by 1-indexed position. Returns None if the
    /// address is out of range.
    pub fn line(&self, n: usize) -> Option<&str> {
        if n == 0 || n > self.lines.len() {
            None
        } else {
            Some(self.lines[n - 1].as_str())
        }
    }
}
