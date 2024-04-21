# Wordlist

**Roll your own diceware!**

Use your own text sources to create a unique list of familiar words for passphrase generation.

![xkcd: Password Strength](https://imgs.xkcd.com/comics/password_strength.png)

Hotlinked from [xkcd](https://xkcd.com/936/), used under a Creative Commons Attribution-NonCommercial 2.5 License.

## Usage

    wordlist add [OPTIONS] [PATH]...
    wordlist pick <COUNT>
    wordlist help [subcommand]...
    wordlist [-V | --version] [-h | --help]

### Flags

    -h, --help       Prints help information
    -V, --version    Prints version information

### Add

Add text files to a wordlist file

#### Options

    -o <output>        Specify a target wordlist file, defaults to 'word-list.txt'

#### Args

    <PATH>...    text files to read into the wordlist. Defaults to reading from the command line if left blank.

### Pick

Display random words from the wordlist

#### Args

    <COUNT>    the number of words to display

### Help

Prints this message or the help of the given subcommand(s)
