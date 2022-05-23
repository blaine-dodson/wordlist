// #![deny(clippy::all)]
// #![deny(clippy::correctness)]
// #![deny(clippy::style)]
// #![deny(clippy::complexity)]
// #![deny(clippy::perf)]
// #![deny(clippy::pedantic)]
// #![deny(clippy::nursery)]
// #![deny(clippy::cargo)]

const DB_NAME: &str = "word-list.txt";

enum Command {
    Invalid,
    AddFiles {
        input: Vec<String>,
        output: String,
        debug: bool,
    },
    Pick(u32),
}

fn process_cmd_line() -> Command {
    use clap::{App, Arg, SubCommand};

    // parse arguments
    let args = App::new("A wordlist manager")
		.version(clap::crate_version!())
		.author(clap::crate_authors!())
		.subcommand(SubCommand::with_name("add")
			.about("Add text files to a wordlist file")
			.version(clap::crate_version!())
			.author(clap::crate_authors!())
			.arg(Arg::with_name("PATH")
				.help("text files to read into the wordlist. leave blank to read from command line.")
				.multiple(true)
			)
			.arg(Arg::with_name("output")
				.short("o")
				.help(&format!("Specify a target wordlist file, defaults to '{}'", DB_NAME))
				.takes_value(true)
				.multiple(false)
				.number_of_values(1)
			)
			.arg(Arg::with_name("debug")
				.short("d")
				.hidden(true)
			)
		)
		.subcommand(SubCommand::with_name("pick")
			.about("Display random words from the wordlist")
			.version(clap::crate_version!())
			.author(clap::crate_authors!())
			.arg(Arg::with_name("COUNT")
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
        ("add", Some(a)) => {
            let mut paths = Vec::new();
            let mut output = String::from(DB_NAME);
            let mut debug = false;

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

            if a.is_present("debug") {
                debug = true;
            }

            Command::AddFiles {
                input: paths,
                output,
                debug,
            }
        }
        ("pick", Some(a)) => {
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

fn cleanup(words: &mut Vec<&str>, debug: bool) {
    let mut bad = Vec::new();

    // find everything we dont like
    for (i, word) in words.iter_mut().enumerate() {
        // too short
        if word.len() < 3 {
            bad.push(i);
            continue;
        }

        // remove non-alpha
        if !word.chars().all(char::is_alphabetic) {
            bad.push(i);
            continue;
        }

        // 'words with more than 2 repetitions'
        let mut ch = '0'; // numbers already removed
        let mut cnt = 0;
        for c in word.chars() {
            if ch == c {
                cnt += 1;
            } else {
                cnt = 1;
                ch = c;
            }
            if cnt > 2 {
                bad.push(i);
                break;
            }
        }
    }

    // get rid of it
    while !bad.is_empty() {
        let i = bad.pop().unwrap();
        if debug {
            println!("'{}'", words.remove(i));
        } else {
            words.remove(i);
        }
    }
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

fn add_files(paths: Vec<String>, out: &str, debug: bool) {
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
        //let lines = string.split_whitespace();
        let lines = string.unicode_words().collect::<Vec<&str>>();
        for line in lines {
            words.push(line);
        }
    }

    println!("words split");

    // cleanup
    words.sort();
    words.dedup();
    cleanup(&mut words, debug);

    println!("words cleaned");

    // TODO: report how many new words

    match write_list_file(out, &words) {
        Ok(()) => {}
        Err(e) => println!("Could not write '{}', {}", out, e),
    }
}

fn pick_words(count: u32) {
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

    // entropy calculation
    let from = words.len() as f64;
    let bits = from.log2().floor() as u32;
    println!(
        "Picking {} words from {}. {}-bits per word, {}-bits total entropy",
        count,
        words.len(),
        bits,
        bits * count
    );

    // pick and print words
    println!();
    for _i in 0..count {
        let word = words[rand::random::<usize>() % words.len()];
        print!("{} ", word);
    }
    println!("\n");
}

fn main() {
    use std::process;

    match process_cmd_line() {
        Command::Invalid => {
            println!("invalid command");
            process::exit(1)
        }
        Command::AddFiles {
            input,
            output,
            debug,
        } => {
            add_files(input, &output, debug);
        }
        Command::Pick(c) => {
            pick_words(c);
        }
    }

    println!("Done!");
}
