pub(crate) struct Cursor<'i> {
    s: &'i str,
    pos: usize,
}

impl<'i> Cursor<'i> {
    pub(crate) fn new(s: &'i str) -> Self {
        Self { s, pos: 0 }
    }
    #[inline]
    pub(crate) fn remaining(&self) -> &'i str {
        &self.s[self.pos..]
    }
    /// Run a function with a mutable view of the remaining input and advance the
    /// cursor by the consumed amount (difference in remaining length).
    #[inline]
    pub(crate) fn run_with<R>(&mut self, f: impl FnOnce(&mut &'i str) -> R) -> R {
        let mut view = &self.s[self.pos..];
        let out = f(&mut view);
        // advance by consumed bytes
        let before = self.s.len() - self.pos;
        let after = view.len();
        let consumed = before.saturating_sub(after);
        self.pos = self.pos.saturating_add(consumed);
        out
    }
}

