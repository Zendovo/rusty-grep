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

- **Literals**: Basic character matching
- **Wildcard (`.`)**: Matches any character
- **Anchors**: 
  - `^` - Start of string
  - `$` - End of string
- **Character Classes**: 
  - `[abc]` - Matches any of a, b, or c
  - `[^abc]` - Matches any character except a, b, or c
- **Escape Sequences**:
  - `\d` - Matches digits (0-9)
  - `\w` - Matches word characters (alphanumeric + underscore)
- **Quantifiers**:
  - `?` - Zero or one occurrence
  - `+` - One or more occurrences
  - `*` - Zero or more occurrences
- **Alternation**: `|` - Matches either left or right alternative
- **Grouping**: `()` - Groups expressions together

## Algorithm Details

The implementation uses a **backtracking-based approach** with position tracking:

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
```