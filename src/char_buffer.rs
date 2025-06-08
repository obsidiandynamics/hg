use std::borrow::Cow;
use crate::graphemes::Grapheme;

#[derive(Default, Debug)]
pub struct CharBuffer {
    offset: usize,
    len: usize,
    copy: String,
    mode: Mode
}

impl CharBuffer {
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self.mode {
            Mode::Slice => self.len == 0,
            Mode::Copy => self.copy.is_empty()
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self.mode {
            Mode::Slice => self.len,
            Mode::Copy => self.copy.len()
        }
    }

    #[inline]
    pub fn push(&mut self, offset: usize, char: char) {
        match self.mode {
            Mode::Slice => {
                if self.len == 0 {
                    self.offset = offset;
                } else {
                    debug_assert_eq!(self.offset + self.len, offset, "wrong character offset: expected {}, got {}", self.offset + self.len, offset);
                }
                self.len += char.len_utf8();
            }
            Mode::Copy => {
                self.copy.push(char);
            }
        }
    }

    #[inline]
    pub fn push_grapheme(&mut self, offset: usize, grapheme: Grapheme) {
        match self.mode {
            Mode::Slice => {
                if self.len == 0 {
                    self.offset = offset;
                } else {
                    debug_assert_eq!(self.offset + self.len, offset, "wrong character offset: expected {}, got {}", self.offset + self.len, offset);
                }
                self.len += grapheme.len_utf8();
            }
            Mode::Copy => {
                self.copy.push(char::from(grapheme));
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        match self.mode {
            Mode::Slice => {
                self.offset = 0;
                self.len = 0;
            }
            Mode::Copy => {
                self.copy.clear();
                self.mode = Mode::Slice;
            }
        }
    }

    #[inline]
    pub fn as_str<'a: 'b, 'b>(&'a self, bytes: &'b [u8]) -> &'b str {
        match self.mode {
            Mode::Slice => {
                self.make_str_slice(bytes)
            }
            Mode::Copy => {
                self.copy.as_str()
            }
        }
    }

    #[inline]
    pub fn string<'b>(&self, bytes: &'b [u8]) -> Cow<'b, str> {
        match self.mode {
            Mode::Slice => {
                Cow::Borrowed(self.make_str_slice(bytes))
            }
            Mode::Copy => {
                Cow::Owned(self.copy.clone())
            }
        }
    }

    #[inline(always)]
    fn make_str_slice<'b>(&self, bytes: &'b [u8]) -> &'b str {
        unsafe { str::from_utf8_unchecked(&bytes[self.offset..self.offset + self.len])}
    }

    #[inline]
    pub fn copy(&mut self, bytes: &[u8]) {
        if matches!(self.mode, Mode::Slice) {
            self.copy.push_str(self.make_str_slice(bytes));
            self.offset = 0;
            self.len = 0;
            self.mode = Mode::Copy;
        }
    }
}

#[derive(Debug)]
enum Mode {
    Slice,
    Copy,
}

impl Default for Mode {
    #[inline]
    fn default() -> Self {
        Mode::Slice
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use crate::char_buffer::{CharBuffer, Mode};

    #[test]
    fn empty_buf() {
        let buf = CharBuffer::default();
        let str = "hi";
        let bytes = str.as_bytes();
        assert!(matches!(buf.mode, Mode::Slice));
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!("", buf.as_str(&bytes));
        assert_eq!("", buf.string(&bytes));
        assert!(matches!(buf.string(&bytes), Cow::Borrowed(_)));
    }

    #[test]
    fn slice_with_unicode() {
        let mut buf = CharBuffer::default();
        let str = "hiµ\n";
        let bytes = str.as_bytes();

        buf.push(0, 'h');
        assert!(!buf.is_empty());
        assert_eq!(buf.len(), 1);
        buf.push(1, 'i');
        assert!(matches!(buf.mode, Mode::Slice));
        println!("buf: {buf:?}");
        assert_eq!("hi", buf.as_str(&bytes));
        assert_eq!("hi", buf.string(&bytes));
        assert!(matches!(buf.string(&bytes), Cow::Borrowed(_)));

        buf.clear();
        println!("buf: {buf:?}");
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!("", buf.as_str(&bytes));

        buf.push(2, 'µ');
        buf.push(4, '\n');
        println!("buf: {buf:?}");
        assert_eq!("µ\n", buf.as_str(&bytes));
    }

    #[test]
    fn slice_to_region_alternate() {
        let mut buf = CharBuffer::default();
        let str = "hiµ\nhello";
        let bytes = str.as_bytes();

        buf.push(0, 'h');
        buf.push(1, 'i');
        println!("buf: {buf:?}");
        buf.copy(&bytes);
        assert!(!buf.is_empty());
        assert_eq!(buf.len(), 2);
        assert!(matches!(buf.mode, Mode::Copy));
        assert_eq!("hi", buf.as_str(&bytes));
        assert_eq!("hi", buf.string(&bytes));
        assert!(matches!(buf.string(&bytes), Cow::Owned(_)));

        buf.push(0, 'µ');
        buf.push(0, '\n');
        assert_eq!("hiµ\n", buf.as_str(&bytes));
        assert_eq!("hiµ\n", buf.string(&bytes));
        assert!(matches!(buf.string(&bytes), Cow::Owned(_)));

        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert!(matches!(buf.mode, Mode::Slice));
        assert_eq!("", buf.as_str(&bytes));
        assert_eq!("", buf.string(&bytes));
        assert!(matches!(buf.string(&bytes), Cow::Borrowed(_)));

        buf.push(5, 'h');
        buf.push(6, 'e');
        assert!(matches!(buf.mode, Mode::Slice));
        assert_eq!("he", buf.as_str(&bytes));

        buf.copy(&bytes);
        assert_eq!(0, buf.offset);
        assert_eq!(0, buf.len);
        assert!(matches!(buf.mode, Mode::Copy));
        assert_eq!("he", buf.as_str(&bytes));

        buf.push(0, 'l');
        assert!(matches!(buf.mode, Mode::Copy));
        assert_eq!("hel", buf.as_str(&bytes));

        buf.clear();
        assert_eq!("", buf.as_str(&bytes));

        buf.push(2, 'µ');
        buf.push(4, '\n');
        assert!(matches!(buf.mode, Mode::Slice));
        assert_eq!("µ\n", buf.as_str(&bytes));
    }

    #[test]
    #[should_panic(expected = "wrong character offset: expected 1, got 2")]
    fn slice_push_wrong_offset() {
        let mut buf = CharBuffer::default();
        buf.push(0, 'h');
        buf.push(2, 'i');
    }
}