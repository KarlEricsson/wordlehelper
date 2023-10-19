use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use std::fs;
use std::io;
use std::io::prelude::*;

pub mod filter;

#[derive(PartialEq)]
enum UserCommands {
    Nothing,
    Exit,
    NewGame,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum GameLength {
    Five = 5,
    Six = 6,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
    fn new_game() -> Game {
        let language = {
            let input = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Game language?")
                .default(0)
                .item("Swedish")
                .item("English")
                .interact()
                .unwrap();

            match input {
                0 => GameLanguage::Swedish,
                1 => GameLanguage::English,
                _ => GameLanguage::Swedish,
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
                .unwrap();

            match input {
                0 => GameLength::Five,
                1 => GameLength::Six,
                _ => GameLength::Five,
            }
        };
        Game {
            language,
            length,
            playfield: vec!['-'; length as usize],
            wrong_letters: vec![],
        }
    }
}

fn main() -> Result<()> {
    loop {
        let code = play_game()?;
        match code {
            UserCommands::Nothing => play_game()?,
            UserCommands::Exit => std::process::exit(0),
            UserCommands::NewGame => play_game()?,
        };
    }
}

fn play_game() -> Result<UserCommands> {
    let mut command: UserCommands = UserCommands::Nothing;
    let mut current_game = Game::new_game();
    let mut possible_words = read_file(&current_game)?;
    while possible_words.len() > 1 && command == UserCommands::Nothing {
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
        print_possible_words(&possible_words);

        let possible_words_without_duplicate_letters =
            filter::words_without_duplicate_letters(&possible_words);
        print_possible_words(&possible_words_without_duplicate_letters);

        let possible_words_without_uncommon_letters = filter::words_without_uncommon_letters(
            &possible_words_without_duplicate_letters,
            &current_game,
        );
        print_possible_words(&possible_words_without_uncommon_letters);

        let possible_words_with_common_letters = filter::words_with_common_letters(
            &possible_words_without_uncommon_letters,
            &current_game,
        );
        print_possible_words(&possible_words_with_common_letters);

        if let Ok(Some(input)) = get_user_input(
            &mut command,
            "Press 3 to print all possible words. Press 4 for latest filtered. Press enter to skip",
        ) {
            if input.trim() == "3" {
                println!(
                    "{:?} Word count: {:?}",
                    &possible_words,
                    &possible_words.len()
                );
            } else if input.trim() == "4" {
                println!(
                    "{:?} Word count: {:?}",
                    &possible_words_with_common_letters,
                    &possible_words_with_common_letters.len()
                )
            }
        }
    }
    Ok(command)
}

fn get_playfield(game: &Game, prompt: &str) -> Result<Option<String>> {
    println!(
        "Use CAPITAL letters for letters in correct slot.\n\
        Use lower case letters for letters in the wrong slot.\n\
        Leave the - if the slot is empty."
    );
    let input: String = Input::new()
        .with_prompt(prompt)
        //.allow_empty(true)
        .with_initial_text(game.playfield.iter().collect::<String>())
        .validate_with(|user_input: &String| -> Result<(), &str> {
            if user_input.chars().count() == game.length as usize {
                Ok(())
            } else {
                Err("To few/many letters in playfield")
            }
        })
        .interact_text()?;

    Ok(Some(input.trim().to_string()))
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

fn get_user_input(command: &mut UserCommands, prompt: &str) -> Result<Option<String>> {
    let mut input = String::new();
    println!("{prompt}");

    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => {
            *command = UserCommands::Exit;
            Ok(None)
        }
        "2" => {
            *command = UserCommands::NewGame;
            Ok(None)
        }
        "" => Ok(None),
        _ => Ok(Some(input.trim().to_string())),
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

            // Ignore words with letters that is known to not be in the word unless part of a locked match.
            if game.wrong_letters.iter().any(|&c| c == letter)
                && letter.to_uppercase().to_string() != game.playfield[index].to_string()
            {
                continue 'nextword;
            }
        }

        new_possible_words.push(word.to_string());
    }
    new_possible_words
}

fn print_possible_words(possible_words: &[String]) {
    let possible_words_ammount = possible_words.len();
    match possible_words_ammount {
        35.. => println!("To many words to print ({possible_words_ammount})."),
        0 => (),
        _ => println!(
            "{:?} words remaining:\n{:?}\n",
            possible_words_ammount,
            possible_words.join(" ")
        ),
    }
}

fn read_file(game: &Game) -> Result<Vec<String>> {
    println!("{:?} {:?}", game, game.language);
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
