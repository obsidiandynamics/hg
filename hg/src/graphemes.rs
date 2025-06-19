use std::str::Bytes;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Grapheme(pub [u8; 4]);

impl Grapheme {
    #[inline]
    pub fn len_utf8(&self) -> usize {
        if self.0[1] == 0 {
            1
        } else if self.0[2] == 0 {
            2
        } else if self.0[3] == 0 {
            3
        } else {
            4
        }
    }
}

impl From<Grapheme> for char {
    #[inline]
    fn from(grapheme: Grapheme) -> Self {
        let str = unsafe { str::from_utf8_unchecked(&grapheme.0[..grapheme.len_utf8()]) };
        unsafe { str.chars().next().unwrap_unchecked() }
    }
}

impl From<char> for Grapheme {
    #[inline]
    fn from(char: char) -> Self {
        let mut bytes = [0u8; 4];
        char.encode_utf8(&mut bytes);
        Grapheme(bytes)
    }
}

#[derive(Debug)]
pub struct Graphemes<'a> {
    iter: Bytes<'a>,
}

// static BYTE_MAP: [u8; 256] = [
//     /* 
//     0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F */
//     1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
//     2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
//     2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
//     3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
//     4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 0, 0, // F
// ];

impl<'a> From<&'a str> for Graphemes<'a> {
    #[inline]
    fn from(str: &'a str) -> Self {
        Graphemes {
            iter: str.bytes(),
        }
    }
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = Grapheme;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.iter.next()?;
        if b0 < 0x80 {
            Some(Grapheme([b0, 0, 0, 0]))
        } else { // b0 > 0xC0 assumed, ignoring the 0x80..0xC0 range (continuation byte)
            Some(read_grapheme(b0, &mut self.iter))
        }
    }
}

#[inline(always)]
pub fn read_grapheme(b0: u8, bytes: &mut Bytes) -> Grapheme {
    __read_grapheme(b0, bytes).unwrap()
}

#[inline(always)]
fn __read_grapheme(b0: u8, bytes: &mut Bytes) -> Option<Grapheme> {
    let b1 = bytes.next()?;
    if b0 >= 0xE0 {
        let b2 = bytes.next()?;
        if b0 >= 0xF0 {
            let b3 = bytes.next()?;
            Some(Grapheme([b0, b1, b2, b3]))
        } else {
            Some(Grapheme([b0, b1, b2, 0]))
        }
    } else {
        Some(Grapheme([b0, b1, 0, 0]))
    }
}

#[cfg(test)]
mod tests {
    use crate::graphemes::{Grapheme, Graphemes};

    fn to_chars(str: &str) -> Vec<char> {
        Graphemes::from(str).map(char::from).collect()
    }

    fn to_lens(str: &str) -> Vec<usize> {
        Graphemes::from(str).map(|grapheme| grapheme.len_utf8()).collect()
    }

    #[test]
    fn ascii() {
        let str = "hello\n";
        let graphemes = to_chars(str);
        assert_eq!(vec!['h', 'e', 'l', 'l', 'o', '\n'], graphemes);
        let lens = to_lens(str);
        assert_eq!(vec![1, 1, 1, 1, 1, 1], lens);
    }

    #[test]
    fn ascii_with_2byte() {
        let str = "aµ";
        let graphemes = to_chars(str);
        assert_eq!(vec!['a', 'µ'], graphemes);
        let lens = to_lens(str);
        assert_eq!(vec![1, 2], lens);
    }

    #[test]
    fn ascii_with_3byte() {
        let str = "aµℝ";
        let graphemes = to_chars(str);
        assert_eq!(vec!['a', 'µ', 'ℝ'], graphemes);
        let lens = to_lens(str);
        assert_eq!(vec![1, 2, 3], lens);
    }

    #[test]
    fn ascii_with_4byte() {
        let str = "aµℝ💣";
        let graphemes = to_chars(str);
        assert_eq!(vec!['a', 'µ', 'ℝ', '💣'], graphemes);
        let lens = to_lens(str);
        assert_eq!(vec![1, 2, 3, 4], lens);
    }
    
    #[test]
    fn conversion() {
        let chars = vec!['a', 'µ', 'ℝ', '💣'];
        for char in chars {
            let grapheme = Grapheme::from(char);
            let back_to_char = char::from(grapheme);
            assert_eq!(char, back_to_char);
        }
    }
}

