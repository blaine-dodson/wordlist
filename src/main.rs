


use std::{fs};

const DB_NAME:&str = "word-list.txt";

enum Command {
	Invalid,
	AddFiles(Vec<String>),
	AddStdIn,
	Pick(u32),
}

fn process_cmd_line() -> Command {
	use clap::{App, SubCommand, Arg};
	
	// parse arguments
	let args = App::new("A wordlist manager")
		.version(clap::crate_version!())
		.author(clap::crate_authors!())
		.subcommand(SubCommand::with_name("add")
			.about("Add wordlist files to master file")
			.version(clap::crate_version!())
			.author(clap::crate_authors!())
			.arg(Arg::with_name("PATH")
				.help("word files to read into the wordlist")
				.multiple(true)))
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
				})))
		.get_matches();
	
	return match args.subcommand() {
		("add" , Some(a)) => {
			if a.is_present("PATH") {
				let values = a.values_of("PATH").unwrap();
				
				let mut paths = Vec::new();
				for v in values {
					println!("paths are: {:?}", v);
					paths.push(String::from(v));
				}
				
				Command::AddFiles(paths)
			} else {
				Command::AddStdIn
			}
		},
		("pick", Some(a)) => {
			let cnt = a.value_of("COUNT").unwrap().parse::<u32>().unwrap();
			Command::Pick(cnt)
		},
		_ => Command::Invalid,
	}
}

fn read_list_file(path: &str) -> Result<String,std::io::Error> {
	use std::io::Read;
	
	let mut fd = fs::OpenOptions::new()
		.read(true)
		.open(path)?;
	
	let mut text = String::new();
	
	fd.read_to_string(&mut text)?;
	
	Ok(text)
}


fn cleanup(words : &mut Vec<&str>) {
	let mut bad = Vec::new();
	
	// find everything we dont like
	for (i,word) in words.iter().enumerate() {
		
		// non-alpha
		if ! word.chars().all(char::is_alphabetic) {
			bad.push(i);
			continue;
		}
		
		// too short
		if word.len() < 3 {
			bad.push(i);
			continue;
		}
		
		// 'words with more than 2 repetitions'
		let mut ch = 'm'; // nothing starts with mm
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
	while bad.len() != 0 {
		let i = bad.pop().unwrap();
		words.remove(i);
	}
}

fn write_list_file(path: &str, words : &Vec<&str>) -> Result<(),std::io::Error> {
	use std::io::Write;
	
	let mut fd = fs::OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(path)?;
	
	for word in words {
		fd.write(word.as_bytes())?;
		fd.write(b"\n")?;
	}
	
	Ok(())
}

//fn split_words(words: &mut Vec<&str>, text: &str) {
//	let lines = text.split_whitespace();
//		
//	for line in lines {
//		words.push(line);
//	}
//}

fn add_files(paths:Vec<String>) {
	let mut strings = Vec::new();
	
	// read in old if it exists
	match read_list_file(DB_NAME) {
		Ok(s) => strings.push(s),
		Err(e) => println!("Could not read '{}', {}", DB_NAME, e),
	};
	
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
	
	println!("all files read");
	
	// create slices for each word
	let mut words = Vec::new();
	for string in &strings {
		//let lines = string.lines();
		let lines = string.split_whitespace();
		
		for line in lines {
			words.push(line);
		}
	}
	
	println!("words split");
	
	cleanup(&mut words);
	words.sort();
	words.dedup();
	
	println!("words cleaned");
	
	match write_list_file(DB_NAME, &words){
		Ok(()) => {},
		Err(e) => println!("Could not write '{}', {}", DB_NAME, e),
	}
}

fn pick_words(count:u32) {
	//use rand;
	
	// read wordlist
	let text = match read_list_file(DB_NAME) {
		Ok(s) => s,
		Err(e) => {
			println!("Could not read '{}', {}", DB_NAME, e);
			return;
		},
	};
	
	// split wordlist
	let mut words = Vec::new();
	for line in text.lines() {
		words.push(line);
	}
	
	// entropy calculation
	let from = words.len() as f64;
	let bits = from.log2().floor() as u32;
	println!("Picking {} words from {}. {}-bits per word, {}-bits total entropy", 
		count, words.len(), bits, bits*count);
	
	// pick and print words
	println!("");
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
		},
		Command::AddFiles(p) => {
			add_files(p);
		},
		Command::AddStdIn => {
			println!("from stdin");
		}
		Command::Pick(c) => {
			pick_words(c);
		},
	}
	
	println!("Done!");
}
