use std::slice;

#[derive(Debug, PartialEq, Eq)]
pub struct Grapheme([u8; 4]);

impl Grapheme {
    #[inline]
    pub fn len(&self) -> usize {
        if self.0[3] != 0 {
            4
        } else if self.0[2] != 0 {
            3
        } else if self.0[1] != 0 {
            2
        } else {
            1
        }
    }
}

impl From<Grapheme> for char {
    #[inline]
    fn from(grapheme: Grapheme) -> Self {
        let str = unsafe { str::from_utf8_unchecked(&grapheme.0[..4]) };
        str.chars().next().unwrap()
    }
}

#[derive(Debug)]
pub struct Graphemes<'a> {
    iter: slice::Iter<'a, u8>,
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
            iter: str.as_bytes().iter(),
        }
    }
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = Grapheme;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let b0 = *self.iter.next()?;
        if b0 < 0x80 {
            Some(Grapheme([b0, 0, 0, 0]))
        } else { // b0 > 0xC0
            let b1 = *self.iter.next()?;
            if b0 >= 0xE0 {
                let b2 = *self.iter.next()?;
                if b0 >= 0xF0 {
                    let b3 = *self.iter.next()?;
                    Some(Grapheme([b0, b1, b2, b3]))
                } else {
                    Some(Grapheme([b0, b1, b2, 0]))
                }
            } else {
                Some(Grapheme([b0, b1, 0, 0]))
            }
        }
        // match self.iter.next() {
        //     None => None,
        //     Some(byte) => {
        //         if *byte < 128 {
        //             // Some(Grapheme(&self.bytes[self.offset..self.offset + 1]))
        //             Some(Grapheme(Default::default()))
        //         } else {
        //             unimplemented!()
        //         }
        //     }
        // }
        
        // if self.offset == self.bytes.len() {
        //     None
        // } else {
        //     let next_byte = self.bytes[self.offset];
        //     let next = if next_byte < 128 {
        //         // Some(Grapheme(&self.bytes[self.offset..self.offset + 1]))
        //         Some(Grapheme(Default::default()))
        //     } else {
        //         unimplemented!()
        //     };
        //     self.offset += 1;
        //     // let len = match next_byte {
        //     //     0x00..0x80 => 1,
        //     //     0x80..0xC0 => 0,
        //     //     0xC0..0xE0 => 2,
        //     //     0xE0..0xF0 => 3,
        //     //     0xF0..0xF8 => 4,
        //     //     0xF8..0xFC => 5,
        //     //     0xFC..0xFE => 6,
        //     //     _ => 0,
        //     // };
        //     // 
        //     // let next = if len >= 1 && len <= 4 {
        //     //     Some(Grapheme(&self.bytes[self.offset..self.offset + len]))
        //     // } else {
        //     //     unimplemented!()
        //     // };
        //     
        //     // self.offset += len;
        //     next
        // }
    }
}

#[cfg(test)]
mod tests {
    use crate::graphemes::Graphemes;

    fn to_graphemes(str: &str) -> Vec<char> {
        Graphemes::from(str).map(char::from).collect()
    }

    #[test]
    fn ascii() {
        let str = "hello\n";
        let graphemes = to_graphemes(str);
        assert_eq!(vec!['h', 'e', 'l', 'l', 'o', '\n'], graphemes);
    }

    #[test]
    fn ascii_with_2byte() {
        let str = "aµ";
        let graphemes = to_graphemes(str);
        assert_eq!(vec!['a', 'µ'], graphemes);
    }

    #[test]
    fn ascii_with_3byte() {
        let str = "aµℝ";
        let graphemes = to_graphemes(str);
        assert_eq!(vec!['a', 'µ', 'ℝ'], graphemes);
    }

    #[test]
    fn ascii_with_4byte() {
        let str = "aµℝ💣";
        let graphemes = to_graphemes(str);
        assert_eq!(vec!['a', 'µ', 'ℝ', '💣'], graphemes);
    }
}

