#[derive(Debug, PartialEq, Eq)]
pub enum Grapheme {
    Byte(u8),
    Char(char)
}

#[derive(Debug)]
pub struct Graphemes<'a> {
    bytes: &'a [u8],
    offset: usize
}

static BYTE_MAP: [u8; 256] = [
    /* 
    0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F */
    1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 0, 0, // F
];

impl<'a> From<&'a str> for Graphemes<'a> {
    #[inline]
    fn from(str: &'a str) -> Self {
        Graphemes {
            bytes: str.as_bytes(),
            offset: 0,
        }
    }
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = Grapheme;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.bytes.len() {
            None
        } else {
            let next_byte = self.bytes[self.offset];
            // println!("byte0: {next_byte:#x}");
            let len = BYTE_MAP[next_byte as usize];
            let next = match len {
                1 => {
                    Some(Grapheme::Byte(next_byte))
                },
                2 => {
                    // let mut bytes: [u8; 4] = [0; 4];
                    // 'ß'.encode_utf8(&mut bytes);
                    // println!("bytes  {:?}", bytes);
                    // // println!("as u32 {:#x}", '🙈' as u32);
                    // println!("as u32 {:#x}", 'ß' as u32);
                    // println!("byte1: {:#x}", self.bytes[self.offset + 1]);
                    // let utf8 = self.bytes[self.offset + 1] as u32;
                    // println!("got {:#x}", utf8);
                    // let char = unsafe { char::from_u32_unchecked(utf8) };
                    let str = unsafe { str::from_utf8_unchecked(&self.bytes[self.offset..self.offset + 2]) };
                    Some(Grapheme::Char(str.chars().next().unwrap()))
                },
                3 => {
                    // println!("byte1: {:#x}", self.bytes[self.offset + 1]);
                    // println!("byte2: {:#x}", self.bytes[self.offset + 2]);
                    // let utf8 = (self.bytes[self.offset + 1] as u32) << 8 | self.bytes[self.offset + 2] as u32;
                    // let char = unsafe { char::from_u32_unchecked(utf8) };
                    let str = unsafe { str::from_utf8_unchecked(&self.bytes[self.offset..self.offset + 3]) };
                    Some(Grapheme::Char(str.chars().next().unwrap()))
                },
                4 => {
                    let str = unsafe { str::from_utf8_unchecked(&self.bytes[self.offset..self.offset + 4]) };
                    Some(Grapheme::Char(str.chars().next().unwrap()))
                },
                _ => Some(Grapheme::Char('\u{FFFD}')) // 5-byte and 6-byte sequences and continuation bytes
            };
            self.offset += len as usize;
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graphemes::Grapheme::{Byte, Char};
    use crate::graphemes::Graphemes;

    #[test]
    fn ascii() {
        let str = "hello\n";
        let graphemes = Graphemes::from(str).collect::<Vec<_>>();
        assert_eq!(vec![Byte(b'h'), Byte(b'e'), Byte(b'l'), Byte(b'l'), Byte(b'o'), Byte(b'\n')], graphemes);
    }

    #[test]
    fn ascii_with_2byte() {
        let str = "aµ";
        let graphemes = Graphemes::from(str).collect::<Vec<_>>();
        assert_eq!(vec![Byte(b'a'), Char('µ')], graphemes);
    }

    #[test]
    fn ascii_with_3byte() {
        let str = "aµℝ";
        let graphemes = Graphemes::from(str).collect::<Vec<_>>();
        assert_eq!(vec![Byte(b'a'), Char('µ'), Char('ℝ')], graphemes);
    }

    #[test]
    fn ascii_with_4byte() {
        let str = "aµℝ💣";
        let graphemes = Graphemes::from(str).collect::<Vec<_>>();
        assert_eq!(vec![Byte(b'a'), Char('µ'), Char('ℝ'), Char('💣')], graphemes);
    }
}

