use std::{fs, io, io::prelude::*};

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

pub mod filter;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum GameLength {
    Five = 5,
    Six = 6,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameLanguage {
    Swedish,
    English,
}

#[derive(Debug)]
pub struct Game {
    language: GameLanguage,
    length: GameLength,
    playfield: Vec<char>,
    wrong_letters: Vec<char>,
}

impl Game {
    fn new_game() -> Self {
        let language = {
            let input = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Game language?")
                .default(0)
                .item("Swedish")
                .item("English")
                .interact_opt()
                .expect("English, Swedish or exit should be only choices.");

            match input {
                Some(0) => GameLanguage::Swedish,
                Some(1) => GameLanguage::English,
                _ => std::process::exit(0),
            }
        };
        let length = if language == GameLanguage::English {
            GameLength::Five
        } else {
            let input = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Playfield size?")
                .default(0)
                .item("Five letters")
                .item("Six letters")
                .interact()
                .expect("Should only be able to select five or six letters.");

            match input {
                0 => GameLength::Five,
                1 => GameLength::Six,
                _ => unreachable!(),
            }
        };
        Self {
            language,
            length,
            playfield: vec!['-'; length as usize],
            wrong_letters: vec![],
        }
    }
}

fn main() -> Result<()> {
    loop {
        println!("Welcome to Wordlehelper! Press q or <Esc> to quit.");
        play_game()?;
    }
}

fn play_game() -> Result<()> {
    let mut current_game = Game::new_game();
    let mut possible_words = read_file(&current_game)?;

    println!(
        "Use CAPITAL letters for letters in correct slot.\n\
        Use lower case letters for letters in the wrong slot.\n\
        Leave the - or use space if the slot is empty.\n"
    );

    while possible_words.len() > 1 {
        let user_input = get_playfield(&current_game, "Enter current playfield");
        if let Ok(Some(input)) = user_input {
            current_game.playfield = input.chars().collect();
        }

        possible_words = solve(&current_game, &possible_words);

        let user_input = get_chars_not_in_word(&current_game, "Characters not in word?");
        if let Ok(Some(input)) = user_input {
            current_game.wrong_letters = input.chars().collect();
        }

        clearscreen::clear().expect("Failed to clear screen");

        possible_words = solve(&current_game, &possible_words);
        println!("All possible words:");
        print_words(&possible_words, true);

        let possible_words_without_duplicate_letters =
            filter::words_without_duplicate_letters(&possible_words);
        print_words(&possible_words_without_duplicate_letters, true);

        let possible_words_without_uncommon_letters = filter::words_without_uncommon_letters(
            &possible_words_without_duplicate_letters,
            &current_game,
        );
        print_words(&possible_words_without_uncommon_letters, true);

        let possible_words_with_common_letters = filter::words_with_common_letters(
            &possible_words_without_uncommon_letters,
            &current_game,
        );

        println!("\nBest current guesses:");
        print_words(&possible_words_with_common_letters, false);

        let input = Select::with_theme(&ColorfulTheme::default())
            .default(0)
            .item("Update playfield")
            .item("Show all possible words")
            .interact_opt()
            .expect("Should only be able to select index 0 or 1.");
        if let Some(index) = input {
            match index {
                0 => (),
                1 => {
                    print_words(&possible_words, false);
                    // TODO: Make prompt not display [y/n] use another/no library?
                    Confirm::new()
                        .with_prompt("Press enter to update playfield")
                        .default(false)
                        .report(false)
                        .show_default(false)
                        .wait_for_newline(true)
                        .interact_opt()
                        .unwrap();
                }
                _ => unreachable!(),
            }
        } else {
            possible_words.clear();
        }
        clearscreen::clear().expect("Failed to clear screen");
    }
    Ok(())
}

fn get_playfield(game: &Game, prompt: &str) -> Result<Option<String>> {
    let input: String = Input::new()
        .with_prompt(prompt)
        .with_initial_text(game.playfield.iter().collect::<String>())
        .validate_with(|user_input: &String| -> Result<(), &str> {
            if user_input.chars().count() == game.length as usize {
                Ok(())
            } else {
                Err("To few/many letters in playfield")
            }
        })
        .interact_text()?;

    Ok(Some(input.trim_matches('\n').to_string().replace(' ', "-")))
}

fn get_chars_not_in_word(game: &Game, prompt: &str) -> Result<Option<String>> {
    let input: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .with_initial_text(game.wrong_letters.iter().collect::<String>())
        .interact_text()?;

    let trimmed_input = input.trim().to_lowercase();

    if trimmed_input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed_input))
    }
}

fn solve(game: &Game, possible_words: &[String]) -> Vec<String> {
    let mut new_possible_words: Vec<String> = Vec::with_capacity(4096);
    'nextword: for word in possible_words {
        // Ignore words without known correct characters in correct slot
        for (index, letter) in word.chars().enumerate() {
            if game.playfield[index].is_uppercase()
                && letter.to_string() != game.playfield[index].to_lowercase().to_string()
            {
                continue 'nextword;
            }

            if (game.playfield[index].is_lowercase()) && !word.contains(game.playfield[index])
                || letter == game.playfield[index]
            {
                continue 'nextword;
            }

            // Ignore words with letters that is known to not be in the word unless part of a locked match,
            // or if the letter is known to be somewhere in the word but currently in the wrong slot.
            if game.wrong_letters.iter().any(|&c| c == letter)
                && letter.to_uppercase().to_string() != game.playfield[index].to_string()
                && !game.playfield.iter().any(|&c| c == letter)
            {
                continue 'nextword;
            }
        }

        new_possible_words.push(word.to_string());
    }
    new_possible_words
}

fn print_words(words: &[String], limit: bool) {
    let word_count = words.len();
    let chunk_size = match word_count {
        x if x < 6 => 3,
        x if x < 20 => 4,
        x if x < 40 => 5,
        _ => 6,
    };
    if limit && word_count > 30 {
        println!("To many words to print ({}).", word_count);
    } else {
        println!();
        for chunk in words.chunks(chunk_size) {
            for string in chunk {
                print!("{}\t\t", string);
            }
            println!();
            if word_count < 20 {
                println!();
            }
        }
    }
}

fn read_file(game: &Game) -> Result<Vec<String>> {
    let file: fs::File = if matches!(game.language, GameLanguage::English) {
        fs::File::open("english5.txt")?
    } else {
        match game.length {
            GameLength::Six => fs::File::open("svenska6.txt")?,
            GameLength::Five => fs::File::open("svenska5.txt")?,
        }
    };
    let possible_words: Vec<String> = io::BufReader::new(file)
        .lines()
        .collect::<io::Result<_>>()?;
    Ok(possible_words)
}
