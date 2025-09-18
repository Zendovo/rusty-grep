use std::collections::HashMap;

use crate::parser::{RegexNode, RepeatKind};

// Match a node against input at position `pos`, returning all possible end positions.
// We use Vec<char> for Unicode-safety; no byte slicing.
pub fn match_node(
    node: &RegexNode,
    input: &[char],
    pos: usize,
    last_group: &mut usize,
    groups: &mut HashMap<usize, (usize, usize)>,
) -> Vec<usize> {
    match node {
        RegexNode::Group { group_num, node: inner } => {
            // Save the start position, match the inner node, and save the end position for each successful match
            let mut results = Vec::new();
            let mut local_groups = groups.clone();
            let ends = match_node(inner, input, pos, last_group, &mut local_groups);
            for end in ends {
                let mut branch_groups = local_groups.clone();
                branch_groups.insert(*group_num, (pos, end));
                // Update the caller's groups only if this path matches
                for (k, v) in branch_groups.iter() {
                    groups.insert(*k, *v);
                }
                results.push(end);
            }
            results
        }
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
                    let res = match_node(n, input, p, last_group, groups);
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
            let mut all_groups: Vec<HashMap<usize, (usize, usize)>> = Vec::new();
            for br in branches {
                let mut branch_groups = groups.clone();
                let res = match_node(br, input, pos, last_group, &mut branch_groups);
                if !res.is_empty() {
                    all_positions.extend(res.iter().copied());
                    all_groups.push(branch_groups);
                }
            }
            all_positions.sort_unstable();
            all_positions.dedup();
            // If any branch matched, update groups to the first successful branch
            if let Some(g) = all_groups.first() {
                for (k, v) in g.iter() {
                    groups.insert(*k, *v);
                }
            }
            all_positions
        }
        RegexNode::Backreference(n) => {
            if let Some((start, end)) = groups.get(n) {
                let length = end - start;
                if pos + length <= input.len() && &input[*start..*end] == &input[pos..pos + length] {
                    vec![pos + length]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        RegexNode::Repeat { node: inner, kind } => match kind {
            RepeatKind::ZeroOrOne => {
                // Either skip it or take one
                let mut positions = vec![pos];
                positions.extend(match_node(inner, input, pos, last_group, groups));
                positions.sort_unstable();
                positions.dedup();
                positions
            }
            RepeatKind::OneOrMore => {
                // Keep applying `inner` as long as we can, collecting all positions
                let mut results = Vec::new();
                let mut frontier = match_node(inner, input, pos, last_group, groups);
                while !frontier.is_empty() {
                    for p in &frontier {
                        if !results.contains(p) {
                            results.push(*p);
                        }
                    }
                    // Advance one more repetition from each frontier point
                    let mut next = Vec::new();
                    for p in &frontier {
                        let step = match_node(inner, input, *p, last_group, groups);
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
                let mut frontier = match_node(inner, input, pos, last_group, groups);
                while !frontier.is_empty() {
                    for p in &frontier {
                        if !results.contains(&p) {
                            results.push(*p);
                        }
                    }
                    let mut next: Vec<usize> = Vec::new();
                    for p in &frontier {
                        let step = match_node(inner, input, *p, last_group, groups);
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
        let mut groups: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut last_group = 0;
        if !match_node(&ast, &input_chars, start, &mut last_group, &mut groups).is_empty() {
            return true;
        }
    }
    false
}