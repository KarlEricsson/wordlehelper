use std::collections::HashSet;

use crate::Game;

pub fn words_without_duplicate_letters(possible_words: &[String]) -> Vec<String> {
    let mut words_without_duplicate_letters: Vec<String> = Vec::new();
    let mut wordhash = HashSet::new();
    'word: for word in possible_words {
        wordhash.clear();
        for letter in word.chars() {
            if let false = wordhash.insert(letter) {
                continue 'word;
            }
        }

        words_without_duplicate_letters.push(word.to_string())
    }
    if !words_without_duplicate_letters.is_empty() {
        println!("Filtering out words with duplicate letters...");
    }
    words_without_duplicate_letters
}

pub fn words_with_common_letters(possible_words: &[String], game: &Game) -> Vec<String> {
    let common_letters: Vec<char> = match game.language {
        crate::GameLanguage::Swedish => vec!['e', 'a', 'n', 'r', 't', 's'],
        crate::GameLanguage::English => vec!['e', 't', 'a', 'o', 'i', 'n'],
    };

    let mut used_common_letters: Vec<char> = common_letters;
    used_common_letters.retain(|&f| !game.playfield.contains(&f));
    let mut words_with_common_letters: Vec<String> = Vec::new();
    for word in possible_words {
        let hits: Vec<&char> = used_common_letters
            .iter()
            .filter(|&c| word.contains(*c))
            .collect();
        if hits.len() >= 2 {
            words_with_common_letters.push(word.to_string());
        }
    }
    if !words_with_common_letters.is_empty() {
        println!("Keeping only possible words with two or more common letters...");
    } else {
        for word in possible_words {
            if used_common_letters.iter().any(|&c| word.contains(c)) {
                words_with_common_letters.push(word.to_string());
            }
        }
        if !words_with_common_letters.is_empty() {
            println!("Keeping only possible words with one common letter...")
        }
    }
    words_with_common_letters
}

pub fn words_without_uncommon_letters(possible_words: &[String], game: &Game) -> Vec<String> {
    let uncommon_letters: Vec<char> = match game.language {
        crate::GameLanguage::Swedish => vec!['q', 'z', 'w', 'x', 'j', 'y'],
        crate::GameLanguage::English => vec!['z', 'q', 'x', 'j', 'v', 'b'],
    };

    let mut used_uncommon_letters: Vec<char> = uncommon_letters;
    used_uncommon_letters.retain(|&f| !game.playfield.contains(&f));
    let mut words_without_uncommon_letters: Vec<String> = Vec::new();
    for word in possible_words {
        if !used_uncommon_letters.iter().any(|&c| word.contains(c)) {
            words_without_uncommon_letters.push(word.to_string());
        }
    }
    if !words_without_uncommon_letters.is_empty() {
        println!("Filtering out possible words with uncommon letters...");
    }
    words_without_uncommon_letters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_words_without_duplicate_letters() {
        let words = vec!["spade".to_string(), "ribba".to_string()];
        let returned = words_without_duplicate_letters(&words);
        assert_eq!(returned, ["spade"]);
    }
}
