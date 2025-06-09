use std::str::Bytes;

pub struct NewlineTerminatedBytes<'a> {
    pub(crate) bytes: Bytes<'a>,
    prev: Option<(usize, u8)>,
    offset: usize,
}

impl<'a> NewlineTerminatedBytes<'a> {
    #[inline(always)]
    pub fn new(bytes: Bytes<'a>) -> Self {
        Self {
            bytes, prev: None, offset: 0,
        }
    }
}

impl Iterator for NewlineTerminatedBytes<'_> {
    type Item = (usize, u8);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bytes.next();
        match next {
            None => {
                match self.prev {
                    None => {
                        self.prev = Some((self.offset, b'\n'));
                    }
                    Some((_, b'\n')) => {
                        self.prev = None
                    }
                    Some((offset, _)) => {
                        self.prev = Some((offset + 1, b'\n'));
                    }
                }
            }
            Some(grapheme) => {
                self.prev = Some((self.offset, grapheme));
                self.offset += 1;
            }
        }
        self.prev
    }
}

#[cfg(test)]
mod tests {
    use crate::newline_terminated_bytes::NewlineTerminatedBytes;

    #[test]
    fn empty() {
        let str = "";
        let mut nt = NewlineTerminatedBytes::new(str.bytes());
        assert_eq!(Some((0, b'\n')), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_with_newline() {
        let str = "hit\n";
        let mut nt = NewlineTerminatedBytes::new(str.bytes());
        assert_eq!(Some((0,  b'h')), nt.next());
        assert_eq!(Some((1,  b'i')), nt.next());
        assert_eq!(Some((2,  b't')), nt.next());
        assert_eq!(Some((3,  b'\n')), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_with_extraneous_newline() {
        let str = "hit\n\n";
        let mut nt = NewlineTerminatedBytes::new(str.bytes());
        assert_eq!(Some((0,  b'h')), nt.next());
        assert_eq!(Some((1,  b'i')), nt.next());
        assert_eq!(Some((2,  b't')), nt.next());
        assert_eq!(Some((3,  b'\n')), nt.next());
        assert_eq!(Some((4,  b'\n')), nt.next());
        assert_eq!(None, nt.next());
    }

    #[test]
    fn ending_without_newline() {
        let str = "hit";
        let mut nt = NewlineTerminatedBytes::new(str.bytes());
        assert_eq!(Some((0,  b'h')), nt.next());
        assert_eq!(Some((1,  b'i')), nt.next());
        assert_eq!(Some((2,  b't')), nt.next());
        assert_eq!(Some((3,  b'\n')), nt.next());
        assert_eq!(None, nt.next());
    }
}