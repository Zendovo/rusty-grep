# Rust Grep Implementation

This is a complete Rust implementation of a `grep`-like tool that supports basic regular expression matching.

[Regular expressions](https://en.wikipedia.org/wiki/Regular_expression)
(Regexes, for short) are patterns used to match character combinations in
strings. [`grep`](https://en.wikipedia.org/wiki/Grep) is a CLI tool for
searching using Regexes.

## Implementation Overview

This implementation is structured into three main modules:

### 1. Parser Module (`src/parser.rs`)
Contains the regex parser that converts regex patterns into an Abstract Syntax Tree (AST):

- **RegexNode**: An enum representing different regex constructs (literals, sequences, alternations, repetitions, anchors, character classes, etc.)
- **RepeatKind**: Enum for quantifiers (`?`, `+`, `*`)
- **Parser**: A recursive descent parser that follows this EBNF grammar:
  ```
  alt := seq ('|' seq)*
  seq := repeat*
  repeat := atom ('?' | '+' | '*')?
  atom := '(' alt ')' | '[' '^'? class ']' | '\\' esc | '.' | '^' | '$' | literal
  ```
  ```
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                              RegexNode                                  │
  └─────────────────────────────────────────────────────────────────────────┘
                   │
                   ▼
       ┌───────────────────────────────┐
       │ Alternation                   │  ← alt := seq ('|' seq)*
       └───────────────────────────────┘
                   │
       ┌───────────┴───────────┐
       ▼                       ▼
  ┌───────────────┐     ┌───────────────┐   ... (one or more)
  │   Sequence    │     │   Sequence    │
  └───────────────┘     └───────────────┘
           │
     repeat*  (ordered list)
           │
           ▼
  ┌─────────────────────────┐
  │        Repeat           │ ← repeat := atom ('?' | '+' | '*')?
  └─────────────────────────┘
           │
           ▼
  ┌─────────────────────────┐
  │          Atom           │ ← atom := '(' alt ')' 
  └─────────────────────────┘             | '[' '^'? class ']' 
           │                              | '\' esc 
           ├──────────────────────────────┤ '.' | '^' | '$' | literal
           ▼
   ┌─────────────┬──────────────┬───────────────┬─────────────┐
   │   Group     │   Class      │    Escape     │  Literal    │
   │  (alt)      │  (items)     │   (char)      │  (char)     │
   ├─────────────┼──────────────┼───────────────┼─────────────┤
   │    Dot      │ AnchorStart  │ AnchorEnd     │             │
   └─────────────┴──────────────┴───────────────┴─────────────┘
  
  Class
   ├─ negate? (bool)
   └─ items : [ ClassItem ]
          ├─ Single(char)
          └─ Range(char,char)
  ```

### 2. Matcher Module (`src/matcher.rs`)
Contains the pattern matching engine:

- **match_node**: Core function that matches a regex node against input text, returning all possible end positions
- **match_pattern**: High-level function that tries to match a pattern at any position in the input

### 3. Main Module (`src/main.rs`)
Contains the command-line interface and main application logic.

## Supported Regex Features

  - `^` - Start of string
  - `$` - End of string
  - `[abc]` - Matches any of a, b, or c
  - `[^abc]` - Matches any character except a, b, or c
  - `\d` - Matches digits (0-9)
  - `\w` - Matches word characters (alphanumeric + underscore)
  - `?` - Zero or one occurrence
  - `+` - One or more occurrences
  - `*` - Zero or more occurrences
### Backreferencing Support

- **Backreferencing**: `\1`, `\2`, ... - Matches the same text as previously captured group. Supports nested and recursive backreferences.

#### Implementation Details

- The parser annotates each capturing group with a unique group number in the AST.
- During matching, the engine tracks the start and end positions of each group for every match path.
- When a backreference (e.g., `\1`) is encountered, the matcher checks if the referenced group was matched and compares the current input with the captured substring.
- This supports nested and recursive backreferences, as group maps are cloned and tracked per match path.

Example:
```bash
echo "foo bar foo" | ./your_program.sh -E "(foo) bar \1"
# Matches: 'foo bar foo'
```

### File Input (Multiple Files)

You can pass one or more files as arguments. The program will process each file line by line and print matching lines:

```bash
./your_program.sh -E "pattern" file1.txt file2.txt
```

### Recursive Directory Search

Use the `-r` flag to search through a directory and its subdirectories recursively. Each matching line is printed with a `<filename>:` prefix:

```bash
./your_program.sh -r -E "pattern" dir/
```

Example:
```bash
$ mkdir -p dir/subdir
$ echo "pear" > dir/fruits.txt
$ echo "strawberry" >> dir/fruits.txt
$ echo "celery" > dir/subdir/vegetables.txt
$ echo "carrot" >> dir/subdir/vegetables.txt
$ echo "cucumber" > dir/vegetables.txt
$ echo "corn" >> dir/vegetables.txt

# Find lines ending with 'er'
$ ./your_program.sh -r -E ".*er" dir/

## Algorithm Details


# Find lines ending with 'ar'
$ ./your_program.sh -r -E ".*ar" dir/
The implementation uses a **backtracking-based approach** with position tracking:


# No matches
$ ./your_program.sh -r -E "missing_fruit" dir/
# (prints nothing, exits with code 1)
```
1. **Parsing**: The input regex pattern is parsed into an AST using recursive descent
2. **Matching**: The matcher tries to find matches at every possible starting position
3. **Position Tracking**: Each match operation returns a vector of possible end positions, allowing for non-deterministic matching

This approach handles complex cases like alternation and repetition by exploring all possible match paths.

## Usage

The program accepts input via stdin and takes a regex pattern as a command-line argument:

```bash
echo "input_text" | ./your_program.sh -E "pattern"
```

Examples:
```bash
# Match literal text
echo "hello world" | ./your_program.sh -E "hello"

# Use wildcards
echo "hello world" | ./your_program.sh -E "h.llo"

# Use quantifiers
echo "hellllo world" | ./your_program.sh -E "hel+o"

# Use anchors
echo "hello" | ./your_program.sh -E "^hello$"

# Use character classes
echo "hello123" | ./your_program.sh -E "[0-9]+"

# Use alternation
echo "cat" | ./your_program.sh -E "cat|dog"
```

The program exits with status 0 if a match is found, or status 1 if no match is found.

## Building and Running

1. Ensure you have `cargo` installed locally
2. Run `./your_program.sh` to run your program. This command compiles your Rust project, so it might be slow the first time you run it. Subsequent runs will be fast.
3. Alternatively, you can build and run directly with cargo:
   ```bash
   cargo build --release
   echo "test input" | cargo run -- -E "pattern"
   ```

## Project Structure

```
src/
├── main.rs      # CLI interface and main application logic
├── parser.rs    # Regex parser and AST definitions
└── matcher.rs   # Pattern matching engine
├── cli.rs       # Argument parsing and CLI flags
```