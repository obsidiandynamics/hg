use std::str::CharIndices;

pub struct NewlineTerminatedChars<'a> {
    char_indices: CharIndices<'a>,
    prev_char: Option<(usize, char)>,
}

impl<'a> NewlineTerminatedChars<'a> {
    #[inline]
    pub fn new(char_indices: CharIndices<'a>) -> Self {
        Self {
            char_indices, prev_char: None
        }
    }
}

impl Iterator for NewlineTerminatedChars<'_> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.char_indices.next();
        match next {
            None => {
                match self.prev_char {
                    None => {
                        self.prev_char = Some((0, '\n'));
                    }
                    Some((_, '\n')) => {
                        self.prev_char = None
                    }
                    Some((index, char)) => {
                        self.prev_char = Some((index + char.len_utf8(), '\n'));
                    }
                }
                self.prev_char
            }
            Some(_) => {
                self.prev_char = next;
                next
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.char_indices.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.char_indices.count()
    }

    #[inline]
    fn last(mut self) -> Option<(usize, char)> {
        self.char_indices.next_back()
    }
}

#[cfg(test)]
mod tests {
    use crate::newline_terminated_chars::NewlineTerminatedChars;
    
    #[test]
    fn size_hint() {
        let str = "hiµ\n";
        let nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!((2, Some(5)), nt.size_hint());
    }
    
    #[test]
    fn count() {
        let str = "hiµ\n";
        let nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!(4, nt.count());
    }
    
    #[test]
    fn last() {
        let str = "hiµ\n";
        let nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!(Some((4, '\n')), nt.last());
    }

    #[test]
    fn empty() {
        let str = "";
        let mut nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!(Some((0, '\n')), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_with_newline() {
        let str = "hiµ\n";
        let mut nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!(Some((0, 'h')), nt.next());
        assert_eq!(Some((1, 'i')), nt.next());
        assert_eq!(Some((2, 'µ')), nt.next());
        assert_eq!(Some((4, '\n')), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_with_extraneous_newline() {
        let str = "hiµ\n\n";
        let mut nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!(Some((0, 'h')), nt.next());
        assert_eq!(Some((1, 'i')), nt.next());
        assert_eq!(Some((2, 'µ')), nt.next());
        assert_eq!(Some((4, '\n')), nt.next());
        assert_eq!(Some((5, '\n')), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_without_newline() {
        let str = "hiµ";
        let mut nt = NewlineTerminatedChars::new(str.char_indices());
        assert_eq!(Some((0, 'h')), nt.next());
        assert_eq!(Some((1, 'i')), nt.next());
        assert_eq!(Some((2, 'µ')), nt.next());
        assert_eq!(Some((4, '\n')), nt.next());
        assert_eq!(None, nt.next());
    }
}