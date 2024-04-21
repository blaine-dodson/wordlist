const DB_NAME: &str = "word-list.txt";

enum Command {
    Invalid,
    AddFiles { input: Vec<String>, output: String },
    Pick(u32),
}

fn process_cmd_line() -> Command {
    use clap::{command, Arg};

    // parse arguments
    let args = command!()
        // .version(clap::crate_version!())
        // .author(clap::crate_authors!())
        .subcommand(clap::Command::new("add")
            .about("Add text files to a wordlist file")
            .arg(Arg::new("PATH")
                .help("text files to read into the wordlist. leave blank to read from command line.")
                .multiple(true)
            )
            .arg(Arg::new("output")
                .short('o')
                .help("Specify a target wordlist file'")
                .takes_value(true)
                .multiple(false)
                .number_of_values(1)
            )
            .arg(Arg::new("debug")
                .short('d')
                .hide(true)
            )
        )
        .subcommand(clap::Command::new("pick")
            .about("Display random words from the wordlist")
            .arg(Arg::new("COUNT")
                .help("the number of words to display")
                .required(true)
                .validator(|s| match s.parse::<u32>() {
                    Ok(_) => Ok(()),
                    Err(_) => Err(String::from("must be positive integer")),
                })
            )
        )
        .get_matches();

    match args.subcommand() {
        Some(("add", a)) => {
            let mut paths = Vec::new();
            let mut output = String::from(DB_NAME);

            if a.is_present("PATH") {
                let values = a.values_of("PATH").unwrap();
                for v in values {
                    println!("paths are: {:?}", v);
                    paths.push(String::from(v));
                }
            }
            // else paths empty

            if a.is_present("output") {
                output = a.value_of("output").unwrap().to_string();
            }

            Command::AddFiles {
                input: paths,
                output,
            }
        }
        Some(("pick", a)) => {
            let cnt = a.value_of("COUNT").unwrap().parse::<u32>().unwrap();
            Command::Pick(cnt)
        }
        _ => Command::Invalid,
    }
}

fn read_list_file(path: &str) -> Result<String, std::io::Error> {
    use std::{fs, io::Read};

    let mut fd = fs::OpenOptions::new().read(true).open(path)?;

    let mut text = String::new();

    fd.read_to_string(&mut text)?;

    Ok(text)
}

fn cleanup<'a, I: Iterator<Item = &'a str>>(input: I) -> Vec<&'a str> {
    // todo!()
    input
        .filter(|w| w.len() >= 3) // too short
        .filter(|w| w.chars().all(char::is_alphabetic)) // non-alfa
        .filter(|w| {
            let mut last_c = '*'; // non-alfa already removed
            let mut cnt = 0;
            for c in w.chars() {
                if last_c == c {
                    cnt += 1;
                } else {
                    cnt = 1;
                    last_c = c;
                }
                if cnt > 2 {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn write_list_file(path: &str, words: &[&str]) -> Result<(), std::io::Error> {
    use std::{fs, io::Write};

    let mut fd = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    for word in words {
        fd.write_all(word.as_bytes())?;
        fd.write_all(b"\n")?;
    }

    Ok(())
}

fn add_files(paths: Vec<String>, out: &str) {
    use unicode_segmentation::UnicodeSegmentation;

    let mut strings = Vec::new();

    // Read in new text
    if paths.is_empty() {
        use std::io::{self, Read};

        // read from stdin
        let mut text = String::new();
        io::stdin().read_to_string(&mut text).unwrap();
        let text = text.to_lowercase();
        strings.push(text);
    } else {
        // read in new
        for path in paths {
            let text = match read_list_file(&path) {
                Ok(s) => s,
                Err(e) => {
                    println!("Could not read '{}', {}", path, e);
                    continue;
                }
            };

            let text = text.to_lowercase();
            strings.push(text);
        }
    }

    // read in old if it exists
    match read_list_file(out) {
        Ok(s) => strings.push(s),
        Err(e) => println!("Could not read '{}', {}", out, e),
    };

    println!("all input read");

    // create slices for each word
    let mut words = Vec::new();
    for string in &strings {
        let lines = string.unicode_words().collect::<Vec<&str>>();
        for line in lines {
            words.push(line);
        }
    }

    println!("words split");

    // cleanup
    words.sort_unstable();
    words.dedup();
    let words = cleanup(words.into_iter());

    println!("words cleaned");

    // TODO: report how many new words

    match write_list_file(out, &words) {
        Ok(()) => println!("Done"),
        Err(e) => println!("Could not write '{}', {}", out, e),
    }
}

fn join<'a, I: Iterator<Item = &'a str>>(mut input: I) -> String {
    let mut output = String::new();
    if let Some(s) = input.next() {
        output.push_str(s);
    }
    for s in input {
        output.push(' ');
        output.push_str(s);
    }
    output
}

fn pick_words(count: u32) {
    use rand::seq::SliceRandom;

    // read wordlist
    let text = match read_list_file(DB_NAME) {
        Ok(s) => s,
        Err(e) => {
            println!("Could not read '{}', {}", DB_NAME, e);
            return;
        }
    };

    // split wordlist
    let mut words = Vec::new();
    for line in text.lines() {
        words.push(line);
    }

    //pick words
    let mut rng = rand::thread_rng();
    let output = join((0..count).map(|_| *words.choose(&mut rng).unwrap()));

    println!("\n{}\n", output);

    // entropy calculation
    let char_cnt = output.len();
    let char_bits = 26_f32.log2();
    let bits_from_chars = char_bits * char_cnt as f32;

    let list_cnt = words.len();
    let word_bits = (list_cnt as f32).log2();
    let bits_from_words = word_bits * count as f32;
    println!(
        "{} characters at {:.2}-bits per char is {:.2}-bits of entropy
{} words at {:.2}-bits per word is {:.2}-bits of entropy",
        char_cnt, char_bits, bits_from_chars, count, word_bits, bits_from_words,
    );
}

fn main() {
    use std::process;

    match process_cmd_line() {
        Command::Invalid => {
            println!("invalid command");
            process::exit(1)
        }
        Command::AddFiles { input, output, .. } => {
            add_files(input, &output);
        }
        Command::Pick(c) => {
            pick_words(c);
        }
    }
}
