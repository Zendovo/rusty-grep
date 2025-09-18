// AST for regex
#[derive(Debug, Clone)]
// Minimal AST for the features we support: concat, alternation, ?, +, anchors, ., \d, \w, classes, literals
pub enum RegexNode {
    Seq(Vec<RegexNode>),
    Alt(Vec<RegexNode>), // bool indicates if it's a capturing group
    Repeat {
        node: Box<RegexNode>,
        kind: RepeatKind,
    },
    StartAnchor,
    EndAnchor,
    Dot,
    Digit,
    Word,
    CharClass {
        chars: Vec<char>,
        negated: bool,
    },
    Literal(char),
    Backreference(usize),
    Group {
        group_num: usize,
        node: Box<RegexNode>,
    },
}

#[derive(Debug, Clone, Copy)]
// The only quantifiers we currently support
pub enum RepeatKind {
    ZeroOrOne,
    OneOrMore,
    ZeroOrMore,
}

// A tiny recursive-descent parser (EBNF):
//   alt := seq ('|' seq)*
//   seq := repeat*
//   repeat := atom ('?' | '+' | '*')?
//   atom := '(' alt ')' | '[' '^'? class ']' | '\\' esc | '.' | '^' | '$' | literal
pub struct Parser<'a> {
    pattern: &'a str,
    pos: usize,
    ref_count: usize,
}

impl<'a> Parser<'a> {
    // Create a new parser for the given pattern
    pub fn new(pattern: &'a str) -> Self {
        Self { pattern, pos: 0, ref_count: 0 }
    }

    // Peek at the current character without advancing
    fn peek(&self) -> Option<char> {
        self.pattern[self.pos..].chars().next()
    }

    // Advance the position and return the character at the old position
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    // Expect a specific character, advancing if matched
    fn expect(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    // Parse the pattern, starting from the top-level alternation
    pub fn parse(&mut self) -> RegexNode {
        self.parse_alt()
    }

    // Parse alternation: alt := seq ('|' seq)*
    fn parse_alt(&mut self) -> RegexNode {
        let mut branches = Vec::new();
        branches.push(self.parse_seq());
        while self.peek() == Some('|') {
            self.advance();
            branches.push(self.parse_seq());
        }
        if branches.len() == 1 {
            branches.pop().unwrap()
        } else {
            RegexNode::Alt(branches)
        }
    }

    // Parse sequence: seq := repeat*
    fn parse_seq(&mut self) -> RegexNode {
        let mut nodes = Vec::new();
        while let Some(ch) = self.peek() {
            // Stop at sequence terminators
            if ch == ')' || ch == '|' {
                break;
            }
            nodes.push(self.parse_repeat());
        }
        RegexNode::Seq(nodes)
    }

    // Parse repetition: repeat := atom ('?' | '+' | '*')?
    fn parse_repeat(&mut self) -> RegexNode {
        let atom = self.parse_atom();
        match self.peek() {
            Some('?') => {
                self.advance();
                RegexNode::Repeat {
                    node: Box::new(atom),
                    kind: RepeatKind::ZeroOrOne,
                }
            }
            Some('+') => {
                self.advance();
                RegexNode::Repeat {
                    node: Box::new(atom),
                    kind: RepeatKind::OneOrMore,
                }
            }
            Some('*') => {
                self.advance();
                RegexNode::Repeat {
                    node: Box::new(atom),
                    kind: RepeatKind::ZeroOrMore,
                }
            } 
            _ => atom,
        }
    }

    // Parse atom: atom := '(' alt ')' | '[' '^'? class ']' | '\' esc | '.' | '^' | '$' | literal
    fn parse_atom(&mut self) -> RegexNode {
        match self.peek() {
            // Parenthesized group
            Some('(') => {
                self.advance();
                self.ref_count += 1;
                let group_num = self.ref_count;
                let node = self.parse_alt();
                let _ = self.expect(')');
                RegexNode::Group {
                    group_num,
                    node: Box::new(node),
                }
            }
            // Character class
            Some('[') => self.parse_char_class(),
            // Escape sequences
            Some('\\') => {
                self.advance();
                match self.advance() {
                    Some('d') => RegexNode::Digit,
                    Some('w') => RegexNode::Word,
                    // if digit, then backreference
                    Some(c) if c.is_digit(10) => {
                        // advance till you find non-digit
                        let mut val: usize = c.to_digit(10).unwrap() as usize;
                        while let Some(d) = self.peek().and_then(|ch| ch.to_digit(10)) {
                            self.advance();
                            val = val * 10 + d as usize;
                        }

                        if val > self.ref_count || val == 0 {
                            // Invalid backreference, treat as literal
                            RegexNode::Literal('\\')
                        } else {
                            RegexNode::Backreference(val)
                        }
                    }
                    Some(c) => RegexNode::Literal(c),
                    None => RegexNode::Literal('\\'),
                }
            }
            // Wildcard
            Some('.') => {
                self.advance();
                RegexNode::Dot
            }
            // Anchors
            Some('^') => {
                self.advance();
                RegexNode::StartAnchor
            }
            Some('$') => {
                self.advance();
                RegexNode::EndAnchor
            }
            // Literal character
            Some(c) => {
                self.advance();
                RegexNode::Literal(c)
            }
            // End of pattern
            None => RegexNode::Seq(vec![]),
        }
    }

    // Parse character class: '[' '^'? class ']'
    fn parse_char_class(&mut self) -> RegexNode {
        let _ = self.advance(); // consume '['
        let negated = if self.peek() == Some('^') {
            self.advance();
            true
        } else {
            false
        };
        let mut chars_in_class = Vec::new();
        while let Some(ch) = self.peek() {
            if ch == ']' {
                break;
            }
            chars_in_class.push(self.advance().unwrap());
        }
        let _ = self.expect(']');
        RegexNode::CharClass {
            chars: chars_in_class,
            negated,
        }
    }
}