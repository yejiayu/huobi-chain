use std::collections::VecDeque;

use crate::expression::{
    types::{Node, Token},
    ExpressionError,
};

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
            return Err(ParseError(
                "no identifiers or operators in expression, that's impossible",
            ));
        }

        let node = if let Some(node) = nodes.get_mut(index) {
            node
        } else {
            return Err(ParseError("no more nodes, but parenthesis_stack is not empty, left parenthesis > right parenthesis"));
        };

        match node.token {
            Token::LeftParenthesis => parenthesis_stack.push(index),
            Token::RightParenthesis => {
                let left_parenthesis_index =
                    if let Some(left_parenthesis_index) = parenthesis_stack.pop() {
                        left_parenthesis_index
                    } else {
                        return Err(ParseError("unclosing right parenthesis"));
                    };

                let mut end = nodes.split_off(index + 1);
                let mut piece = nodes.split_off(left_parenthesis_index);

                // we should get something encapped with ()
                if let Token::LeftParenthesis = piece
                    .pop_front()
                    .expect("there should be a left parenthesis")
                    .token
                {
                } else {
                    return Err(ParseError(
                        "assume a piece of expression starting with LeftParenthesis, but fails",
                    ));
                };
                if let Token::RightParenthesis = piece
                    .pop_back()
                    .expect("there should be a right parenthesis")
                    .token
                {
                } else {
                    return Err(ParseError(
                        "assume a piece of expression ending with RightParenthesis, but fails",
                    ));
                };
                // parse (left, right)
                let parsed_parenthesis = parse_internal(piece)?;
                nodes.push_back(parsed_parenthesis);
                nodes.append(&mut end);

                index = left_parenthesis_index;
            }
            _ => (),
        }

        if parenthesis_stack.is_empty() {
            if nodes.get(index + 1).is_none() {
                break;
            } else {
                return Err(ParseError("parenthesis stack is empty, but still has unparsed nodes, that should be more right parenthesis "));
            }
        }

        index += 1;
    }

    if nodes.len() > 1 {
        return Err(ParseError(
            "parse expression error, no operator between more than 1 identifiers",
        ));
    };

    let node = nodes
        .pop_back()
        .expect("parse expression error, no node in nodes, that's weird");

    Ok(node)
}

// we scan all operator from highest priority and combine them. I knew this
// method is quite slow, but is less of faults
fn parse_internal(mut nodes: VecDeque<Node>) -> Result<Node, ParseError> {
    if nodes.is_empty() {
        return Err(ParseError(
            "no identifiers or operators in expression, empty in ()",
        ));
    }

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
            let node = node.expect("weird, we know there is a node");
            if node.token.get_priority() != priority {
                index += 1;
                continue;
            }

            // this is a parsed node, likely comes from another parenthesis's result, and
            // it's dead
            if node.parsed {
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

            if priority == 1 {
                let _a = 0;
            }

            // index is setting to node
            if left {
                let mut node = nodes
                    .remove(index)
                    .expect("we marked there is a node, but it's gone, weird");

                if index < 1 {
                    println!("{}", node);
                    println!("{}", priority);
                    return Err(ParseError("the operator need a left node in left (,or right in revers mode) child node, but there is no more leading nodes"));
                }

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
                    return Err(ParseError("the operator need a left node in left (,or right in revers mode) child node, but there is no more leading nodes"));
                }
            }

            if right {
                let mut node = nodes
                    .remove(index)
                    .expect("we marked there is a node, but it's gone, weird");
                if let Some(right) = nodes.remove(index) {
                    if !reverse_mode {
                        node.right = Some(Box::new(right));
                    } else {
                        node.left = Some(Box::new(right));
                    }
                    nodes.insert(index, node);
                } else {
                    return Err(ParseError("the operator need a right node in left (,or left in revers mode) child node"));
                }
            }

            let mut node = nodes
                .get_mut(index)
                .expect("we marked there is a node, but it's gone, weird");
            node.parsed = true;

            index += 1
        }

        if reverse_mode {
            nodes = reverse(nodes)
        }

        priority -= 1
    }

    if nodes.len() > 1 {
        return Err(ParseError(
            "parse expression error, no operator between more than 1 identifiers",
        ));
    };

    Ok(nodes
        .pop_back()
        .expect("parse expression error, no node in nodes, that's weird"))
}

fn reverse(mut nodes: VecDeque<Node>) -> VecDeque<Node> {
    let mut temp = VecDeque::new();
    while let Some(node) = nodes.pop_front() {
        temp.push_front(node);
    }
    temp
}
