use crate::graphemes::{Grapheme, Graphemes};

pub struct NewlineTerminatedChars<'a> {
    graphemes: Graphemes<'a>,
    prev: Option<(usize, Grapheme)>,
    offset: usize,
}

impl<'a> NewlineTerminatedChars<'a> {
    #[inline]
    pub fn new(graphemes: Graphemes<'a>) -> Self {
        Self {
            graphemes, prev: None, offset: 0,
        }
    }
}

const NEWLINE: Grapheme = Grapheme([b'\n', b'\0', b'\0', b'\0']);

impl Iterator for NewlineTerminatedChars<'_> {
    type Item = (usize, Grapheme);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.graphemes.next();
        match next {
            None => {
                match self.prev {
                    None => {
                        self.prev = Some((self.offset, NEWLINE));
                    }
                    Some((_, NEWLINE)) => {
                        self.prev = None
                    }
                    Some((offset, grapheme)) => {
                        self.prev = Some((offset + grapheme.len_utf8(), NEWLINE));
                    }
                }
            }
            Some(grapheme) => {
                self.prev = Some((self.offset, grapheme));
                self.offset += grapheme.len_utf8();
            }
        }
        self.prev
    }
}

#[cfg(test)]
mod tests {
    use crate::graphemes::{Grapheme, Graphemes};
    use crate::newline_terminated_chars::NewlineTerminatedChars;

    #[test]
    fn empty() {
        let str = "";
        let mut nt = NewlineTerminatedChars::new(Graphemes::from(str));
        assert_eq!(Some((0, Grapheme::from('\n'))), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_with_newline() {
        let str = "hiµ\n";
        let mut nt = NewlineTerminatedChars::new(Graphemes::from(str));
        assert_eq!(Some((0,  Grapheme::from('h'))), nt.next());
        assert_eq!(Some((1,  Grapheme::from('i'))), nt.next());
        assert_eq!(Some((2,  Grapheme::from('µ'))), nt.next());
        assert_eq!(Some((4,  Grapheme::from('\n'))), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_with_extraneous_newline() {
        let str = "hiµ\n\n";
        let mut nt = NewlineTerminatedChars::new(Graphemes::from(str));
        assert_eq!(Some((0,  Grapheme::from('h'))), nt.next());
        assert_eq!(Some((1,  Grapheme::from('i'))), nt.next());
        assert_eq!(Some((2,  Grapheme::from('µ'))), nt.next());
        assert_eq!(Some((4,  Grapheme::from('\n'))), nt.next());
        assert_eq!(Some((5,  Grapheme::from('\n'))), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_without_newline() {
        let str = "hiµ";
        let mut nt = NewlineTerminatedChars::new(Graphemes::from(str));
        assert_eq!(Some((0,  Grapheme::from('h'))), nt.next());
        assert_eq!(Some((1,  Grapheme::from('i'))), nt.next());
        assert_eq!(Some((2,  Grapheme::from('µ'))), nt.next());
        assert_eq!(Some((4,  Grapheme::from('\n'))), nt.next());
        assert_eq!(None, nt.next());
    }
}