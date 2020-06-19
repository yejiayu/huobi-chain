use crate::expression::{
    types::{Node, Token},
    ExpressionError,
};

use std::collections::VecDeque;

// FIXME: return ParseError instead of panic without msg
// FIXME: also replace expect with ParseError
pub struct ParseError(&'static str);

impl From<ParseError> for ExpressionError {
    fn from(err: ParseError) -> ExpressionError {
        ExpressionError::ParseError(err.0.to_owned())
    }
}

// Note: we don't have triple-operate
// () > . > @ > ! > & > | > ()
pub fn parse(tokens: Vec<Token>) -> Result<Node, ParseError> {
    let mut nodes = tokens
        .into_iter()
        .map(|token| token.into_node(None, None))
        .collect::<VecDeque<Node>>();

    // because parenthesis is the highest outside but lowest inside, so we
    // separately deal it
    let mut parenthesis_stack = Vec::new();

    let mut index = 0;
    loop {
        if nodes.is_empty() {
            return Err(ParseError(""));
        }

        let node = nodes.get_mut(index).expect("");

        match node.token {
            Token::LeftParenthesis => parenthesis_stack.push(index),
            Token::RightParenthesis => {
                let left_parenthesis_index = parenthesis_stack
                    .pop()
                    .expect("un-closing parenthesis, need left parenthesis");
                let mut end = nodes.split_off(index + 1);
                let mut piece = nodes.split_off(left_parenthesis_index);
                if let Token::LeftParenthesis = piece.pop_front().expect("").token {
                } else {
                    return Err(ParseError(""));
                };
                if let Token::RightParenthesis = piece.pop_back().expect("").token {
                } else {
                    return Err(ParseError(""));
                };
                // parse (left, right)
                let parsed_parenthesis = parse_internal(piece)?;
                nodes.push_back(parsed_parenthesis);
                nodes.append(&mut end);

                index = left_parenthesis_index;
            }
            _ => (),
        }

        index += 1;

        if parenthesis_stack.is_empty() {
            break;
        }
    }

    if nodes.len() > 1 {
        return Err(ParseError(""));
    };

    let node = nodes.pop_back().expect("");

    Ok(node)
}

// we scan all operator from highest and combine them. I new this method is
// quite slow, but is less of faults
fn parse_internal(mut nodes: VecDeque<Node>) -> Result<Node, ParseError> {
    let mut priority = Token::get_highest_priority() - 1;
    while priority > 0 {
        let mut index = 0;

        let reverse_mode = Token::get_associativity_by_priority(priority) == 2;
        if reverse_mode {
            nodes = reverse(nodes)
        }
        loop {
            let node = nodes.get(index);
            if node.is_none() {
                break;
            }
            let node = node.expect("");
            if node.token.get_priority() != priority {
                index += 1;
                continue;
            }

            let mut left = false;
            let mut right = false;
            if node.token.must_have_left() {
                if !reverse_mode {
                    left = true
                } else {
                    right = true
                }
            }
            if node.token.must_have_right() {
                if !reverse_mode {
                    right = true
                } else {
                    left = true
                }
            }
            // index is seting to node
            if left {
                let mut node = nodes.remove(index).expect("");
                if let Some(left) = nodes.remove(index - 1) {
                    if !reverse_mode {
                        node.left = Some(Box::new(left));
                    } else {
                        node.right = Some(Box::new(left));
                    }
                    nodes.insert(index - 1, node);
                    // set index to node
                    index -= 1;
                } else {
                    return Err(ParseError(""));
                }
            }

            if right {
                let mut node = nodes.remove(index).expect("");
                if let Some(right) = nodes.remove(index) {
                    if !reverse_mode {
                        node.right = Some(Box::new(right));
                    } else {
                        node.left = Some(Box::new(right));
                    }
                    nodes.insert(index, node);
                } else {
                    return Err(ParseError(""));
                }
            }

            index += 1
        }

        if reverse_mode {
            nodes = reverse(nodes)
        }

        priority -= 1
    }

    if nodes.len() > 1 {
        return Err(ParseError(""));
    };

    Ok(nodes.pop_back().expect(""))
}

fn reverse(mut nodes: VecDeque<Node>) -> VecDeque<Node> {
    let mut temp = VecDeque::new();
    while let Some(node) = nodes.pop_front() {
        temp.push_front(node);
    }
    temp
}
