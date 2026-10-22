//! Text edits in UIKit's terms (`textField:shouldChangeCharactersInRange:replacementString:`).

/// Replace `length` UTF-16 units at `location` with `replacement`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Utf16Edit {
    pub location: usize,
    pub length: usize,
    pub replacement: String,
}

/// The single replacement that turns `old` into `new`: everything between their longest common
/// prefix and suffix, measured in UTF-16 units like `NSRange`.
pub fn edit_between(old: &str, new: &str) -> Utf16Edit {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();

    let prefix = old_chars
        .iter()
        .zip(&new_chars)
        .take_while(|(old_char, new_char)| old_char == new_char)
        .count();
    let max_suffix = old_chars.len().min(new_chars.len()) - prefix;
    let old_rest = old_chars[prefix..].iter().rev();
    let new_rest = new_chars[prefix..].iter().rev();
    let suffix = old_rest
        .zip(new_rest)
        .take(max_suffix)
        .take_while(|(old_char, new_char)| old_char == new_char)
        .count();

    let removed = &old_chars[prefix..old_chars.len() - suffix];
    let inserted = &new_chars[prefix..new_chars.len() - suffix];
    Utf16Edit {
        location: utf16_len(&old_chars[..prefix]),
        length: utf16_len(removed),
        replacement: inserted.iter().collect(),
    }
}

fn utf16_len(chars: &[char]) -> usize {
    chars.iter().map(|character| character.len_utf16()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(location: usize, length: usize, replacement: &str) -> Utf16Edit {
        Utf16Edit { location, length, replacement: replacement.to_owned() }
    }

    #[test]
    fn typing_appends_at_the_end() {
        assert_eq!(edit_between("Oog", "Ooga"), edit(3, 0, "a"));
    }

    #[test]
    fn backspace_removes_one_unit() {
        assert_eq!(edit_between("Ooga", "Oog"), edit(3, 1, ""));
    }

    #[test]
    fn a_middle_replacement_keeps_prefix_and_suffix() {
        assert_eq!(edit_between("Klik", "Klak"), edit(2, 1, "a"));
    }

    #[test]
    fn repeated_letters_are_not_counted_twice() {
        assert_eq!(edit_between("aa", "aaa"), edit(2, 0, "a"));
    }

    #[test]
    fn locations_count_utf16_units() {
        assert_eq!(edit_between("🙂a", "🙂ab"), edit(3, 0, "b"));
    }

    #[test]
    fn clearing_replaces_everything() {
        assert_eq!(edit_between("Tom", ""), edit(0, 3, ""));
    }
}
