use crate::parser::{RegexNode, RepeatKind};

// Match a node against input at position `pos`, returning all possible end positions.
// We use Vec<char> for Unicode-safety; no byte slicing.
pub fn match_node(node: &RegexNode, input: &[char], pos: usize) -> Vec<usize> {
    match node {
        RegexNode::Literal(c) => {
            if pos < input.len() && input[pos] == *c {
                vec![pos + 1]
            } else {
                vec![]
            }
        }
        RegexNode::Dot => {
            if pos < input.len() {
                vec![pos + 1]
            } else {
                vec![]
            }
        }
        RegexNode::Digit => {
            if pos < input.len() && input[pos].is_digit(10) {
                vec![pos + 1]
            } else {
                vec![]
            }
        }
        RegexNode::Word => {
            if pos < input.len() && (input[pos].is_alphanumeric() || input[pos] == '_') {
                vec![pos + 1]
            } else {
                vec![]
            }
        }
        RegexNode::CharClass { chars, negated } => {
            if pos >= input.len() {
                return vec![];
            }
            let contains = chars.contains(&input[pos]);
            if (*negated && !contains) || (!*negated && contains) {
                vec![pos + 1]
            } else {
                vec![]
            }
        }
        RegexNode::StartAnchor => {
            if pos == 0 {
                vec![pos]
            } else {
                vec![]
            }
        }
        RegexNode::EndAnchor => {
            if pos == input.len() {
                vec![pos]
            } else {
                vec![]
            }
        }
        RegexNode::Seq(nodes) => {
            // Accumulate possible positions as we progress through the sequence
            let mut positions = vec![pos];
            for n in nodes {
                let mut next_positions = Vec::new();
                for p in positions {
                    let res = match_node(n, input, p);
                    next_positions.extend(res);
                }
                if next_positions.is_empty() {
                    return vec![];
                }
                next_positions.sort_unstable();
                next_positions.dedup();
                positions = next_positions;
            }
            positions
        }
        RegexNode::Alt(branches) => {
            let mut all_positions = Vec::new();
            for br in branches {
                let res = match_node(br, input, pos);
                all_positions.extend(res);
            }
            all_positions.sort_unstable();
            all_positions.dedup();
            all_positions
        }
        RegexNode::Repeat { node: inner, kind } => match kind {
            RepeatKind::ZeroOrOne => {
                // Either skip it or take one
                let mut positions = vec![pos];
                positions.extend(match_node(inner, input, pos));
                positions.sort_unstable();
                positions.dedup();
                positions
            }
            RepeatKind::OneOrMore => {
                // Keep applying `inner` as long as we can, collecting all positions
                let mut results = Vec::new();
                let mut frontier = match_node(inner, input, pos);
                while !frontier.is_empty() {
                    for p in &frontier {
                        if !results.contains(p) {
                            results.push(*p);
                        }
                    }
                    // Advance one more repetition from each frontier point
                    let mut next = Vec::new();
                    for p in &frontier {
                        let step = match_node(inner, input, *p);
                        next.extend(step);
                    }
                    next.sort_unstable();
                    next.dedup();
                    frontier = next;
                }
                results.sort_unstable();
                results.dedup();
                results
            }
            RepeatKind::ZeroOrMore => {
                // Keep the current position as a valid match (zero occurrences)
                let mut results = vec![pos];
                // First occurrence
                let mut frontier = match_node(inner, input, pos);
                while !frontier.is_empty() {
                    for p in &frontier {
                        if !results.contains(&p) {
                            results.push(*p);
                        }
                    }
                    let mut next: Vec<usize> = Vec::new();
                    for p in &frontier {
                        let step = match_node(inner, input, *p);
                        next.extend(step);
                    }
                    next.sort_unstable();
                    next.dedup();
                    frontier = next;
                }
                results.sort_unstable();
                results.dedup();
                results
            }
        },
    }
}

// Try to match at any position (unless ^/$ constrain it via the AST itself)
pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut parser = crate::parser::Parser::new(pattern);
    let ast = parser.parse();
    let input_chars: Vec<char> = input_line.chars().collect();
    for start in 0..=input_chars.len() {
        if !match_node(&ast, &input_chars, start).is_empty() {
            return true;
        }
    }
    false
}