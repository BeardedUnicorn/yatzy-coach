pub fn letter_value(ch: char) -> Option<u8> {
    match ch.to_ascii_uppercase() {
        'R' | 'A' | 'E' | 'I' | 'S' | 'T' | 'O' => Some(1),
        'L' | 'U' | 'D' | 'N' => Some(2),
        'H' | 'G' | 'Y' => Some(3),
        'C' | 'B' | 'F' | 'P' | 'W' | 'M' => Some(4),
        'K' | 'V' => Some(5),
        'X' => Some(8),
        'Q' | 'Z' | 'J' => Some(10),
        _ => None,
    }
}
