// Keep already reported keys held during an ambiguous rectangle, but suppress
// new presses until the ambiguity clears. Releases always pass through.
pub fn ghosted(raw: &[u32; 8], row: usize, col: usize) -> bool {
    for other in 0..8 {
        if other != row && (raw[row] & raw[other]).count_ones() >= 2 && raw[other] & (1 << col) != 0 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rectangles_suppress_new_keys_but_two_key_chords_do_not() {
        let mut raw = [0; 8];
        raw[0] = 3;
        assert!(!ghosted(&raw, 0, 0));
        raw[1] = 1;
        assert!(!ghosted(&raw, 0, 0));
        raw[1] = 3;
        assert!(ghosted(&raw, 0, 0));
        assert!(ghosted(&raw, 1, 1));
        raw[2] = 4;
        assert!(!ghosted(&raw, 2, 2));
    }
}
