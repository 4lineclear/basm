#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullSpan {
    pub line: u32,
    pub offset: u32,
    pub span: Span,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    // NOTE: 'from' must come before 'to' for proper ordering
    pub from: u32,
    pub to: u32,
}

impl From<u32> for Span {
    fn from(value: u32) -> Self {
        Span::point(value)
    }
}
impl From<(u32, u32)> for Span {
    fn from((from, to): (u32, u32)) -> Self {
        Span::new(from, to)
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.from, self.to)
    }
}

impl Span {
    pub fn slice(self, src: &str) -> &str {
        &src[self.from as usize..self.to as usize]
    }
    pub fn to(mut self, other: Self) -> Self {
        self.to = other.to;
        self
    }
    pub fn between(mut self, other: Self) -> Self {
        self.from = self.to;
        self.to = other.from;
        self
    }
    pub fn point(pos: u32) -> Self {
        Self {
            from: pos,
            to: pos + 1,
        }
    }
    pub fn new(from: u32, to: u32) -> Self {
        Self { from, to }
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.from += offset;
        self.to += offset;
        self
    }

    pub fn len(&self) -> u32 {
        self.to - self.from
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
