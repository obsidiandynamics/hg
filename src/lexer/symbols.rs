pub static SYMBOL_MAP: [bool; 256] = [
    /*
    0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F */
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 0
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 1
    F, T, F, T, T, T, T, F, F, F, T, T, T, T, T, T, // 2
    F, F, F, F, F, F, F, F, F, F, T, T, T, T, T, T, // 3
    T, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 4
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, T, F, // 5
    T, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 6
    F, F, F, F, F, F, F, F, F, F, F, F, T, F, T, F, // 7
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 8
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 9
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // A
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // B
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // C
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // D
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // E
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // F
];

const T: bool = true;
const F: bool = false;

#[cfg(test)]
mod tests {
    use crate::lexer::symbols::SYMBOL_MAP;
    
    const EXPECTED_SYMBOLS: &str = "!#$%&*+,-./:;<=>?@^`|~";

    #[test]
    fn all_symbols_in_table() {
        for byte in EXPECTED_SYMBOLS.bytes() {
            assert!(SYMBOL_MAP[byte as usize], "for byte {byte:#x}");
        }
    }
    
    #[test]
    fn no_extraneous_symbols_in_table() {
        let expected_symbol_bytes = EXPECTED_SYMBOLS.as_bytes();
        for (index, &symbol) in SYMBOL_MAP.iter().enumerate() {
            if symbol {
                assert!(expected_symbol_bytes.contains(&(index as u8)), "for index {index:#x}");
            }
        }
    }
}