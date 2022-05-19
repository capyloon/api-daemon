use core::str::from_utf8;

#[derive(Clone)]
pub struct Input<'a> {
  data: &'a [u8],
  next: usize,
}

impl<'a> Input<'a> {
  pub fn new(data: &'a str) -> Self {
    Self {
      data: data.trim_matches(Self::ctrl_or_space).as_bytes(),
      next: 0,
    }
  }

  #[inline]
  pub const fn index(&self) -> u32 {
    self.next as u32
  }

  #[inline]
  pub fn exhausted(&self) -> bool {
    self.peek().is_none()
  }

  pub fn peek(&self) -> Option<char> {
    self.data.get(self.next).copied().map(Into::into)
  }

  #[allow(clippy::should_implement_trait)]
  pub fn next(&mut self) -> Option<char> {
    let ch: Option<char> = self.peek();
    self.next = self.next.saturating_add(1);
    ch
  }

  pub fn take(&mut self, amount: usize) -> Option<&str> {
    self
      .data
      .get(self.next..self.next + amount)
      .and_then(|data| {
        self.next = self.next.saturating_add(amount);
        from_utf8(data).ok()
      })
  }

  const fn ctrl_or_space(ch: char) -> bool {
    ch.is_ascii_control() || ch.is_ascii_whitespace()
  }
}
