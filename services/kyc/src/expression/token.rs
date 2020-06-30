use std::collections::HashMap;

use crate::expression::{
    types::{Node, Token},
    ExpressionError,
};

const HIGHEST_PRIORITY: u8 = 6u8;

impl Token {
    pub fn into_node(self, left: Option<Node>, right: Option<Node>) -> Node {
        let mut node = Node {
            token:  self,
            left:   None,
            right:  None,
            parsed: false,
        };
        if let Some(left_node) = left {
            node.left = Some(Box::new(left_node))
        };
        if let Some(right_node) = right {
            node.right = Some(Box::new(right_node))
        };
        node
    }

    // () > . > @ > ! > & > | > ()
    pub fn get_priority(&self) -> u8 {
        match self {
            Token::LeftParenthesis => 0,
            Token::RightParenthesis => HIGHEST_PRIORITY,
            Token::Dot => 5,
            Token::Has => 4,
            Token::Not => 3, // right
            Token::And => 2,
            Token::Or => 1,
            Token::Value(_) => 0,
            Token::Identifier(_) => 0,
        }
    }

    pub fn get_highest_priority() -> u8 {
        HIGHEST_PRIORITY
    }

    pub fn must_have_left(&self) -> bool {
        match self {
            Token::Has | Token::Dot | Token::And | Token::Or => true,
            _ => false,
        }
    }

    pub fn must_have_right(&self) -> bool {
        match self {
            Token::Has | Token::Dot | Token::And | Token::Or | Token::Not => true,
            _ => false,
        }
    }

    // true for left
    // pub fn get_associativity(&self) -> bool {
    //     match self {
    //         Token::Dot | Token::Has | Token::And | Token::Or => true,
    //         Token::Not => false,
    //         _ => unreachable!(),
    //     }
    // }

    // 1 for left, 2 for right, 0 for error
    pub fn get_associativity_by_priority(priority: u8) -> u8 {
        if priority == 3 {
            return 2;
        } else if priority == 0 {
            return 0;
        }
        1
    }
}

pub fn scan(input: String) -> Result<Vec<Token>, ExpressionError> {
    let mut acute = false;

    let mut position = Vec::<usize>::new();
    position.push(0);
    for (i, c) in input.chars().enumerate() {
        if !c.is_ascii() {
            return Err(ExpressionError::ScanError("not ascii".to_string()));
        }

        match c {
            '!' | '&' | '|' | '(' | ')' | ' ' | '.' | '@' => {
                if !acute {
                    position.push(i);
                    position.push(i + 1)
                }
            }
            // acute is crazy, it eats all chars until see its bro
            '`' => {
                position.push(i);
                position.push(i + 1);
                acute = !acute
            }
            _ => (),
        }
    } // all operators have been marked
    position.push(input.chars().count());

    if acute {
        return Err(ExpressionError::ScanError("unclosed acute".to_string()));
    }

    let mut start: usize;
    let mut end: usize = 0;
    let mut logic_and = false;
    let mut logic_or = false;
    let mut parenthesis = 0i32;
    let temp = input.chars().collect::<Vec<char>>();
    let char_sequence = temp.as_slice();
    let mut tokens = Vec::<Token>::new();
    tokens.push(Token::LeftParenthesis);
    let mut operator_priority_count_map = HashMap::<u8, u32>::new();
    for new_pos in position {
        start = end;
        end = new_pos;
        let token_str = (&char_sequence[start..end]).iter().collect::<String>();

        if acute {
            if token_str == "`" {
                acute = false
            } else {
                let count = operator_priority_count_map
                    .entry(Token::Value("".to_string()).get_priority())
                    .or_insert(0);
                *count += 1;
                tokens.push(Token::Value(token_str.clone()));
            }
            continue;
        }

        if token_str == "" {
            continue;
        }

        if logic_and {
            if token_str != "&" {
                return Err(ExpressionError::ScanError(
                    "single logic_and operator".to_string(),
                ));
            }
            logic_and = false;
            continue;
        }

        if logic_or {
            if token_str != "|" {
                return Err(ExpressionError::ScanError(
                    "single logic_or operator".to_string(),
                ));
            }
            logic_or = false;
            continue;
        }

        if token_str == "`" {
            acute = true;
            continue;
        } else if token_str == "." {
            let count = operator_priority_count_map
                .entry(Token::Dot.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::Dot);
        } else if token_str == "!" {
            let count = operator_priority_count_map
                .entry(Token::Not.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::Not);
        } else if token_str == "|" {
            logic_or = true;

            let count = operator_priority_count_map
                .entry(Token::Or.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::Or);
        } else if token_str == "&" {
            logic_and = true;

            let count = operator_priority_count_map
                .entry(Token::And.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::And);
        } else if token_str == "@" {
            let count = operator_priority_count_map
                .entry(Token::Has.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::Has);
        } else if token_str == "(" {
            let count = operator_priority_count_map
                .entry(Token::LeftParenthesis.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::LeftParenthesis);
            parenthesis += 1;
        } else if token_str == ")" {
            let count = operator_priority_count_map
                .entry(Token::RightParenthesis.get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::RightParenthesis);
            parenthesis -= 1;
        } else if token_str == " " {
            continue;
        } else {
            let count = operator_priority_count_map
                .entry(Token::Identifier("".to_string()).get_priority())
                .or_insert(0);
            *count += 1;
            tokens.push(Token::Identifier(token_str.clone()));
        }
    }

    if acute || logic_or || logic_and || parenthesis != 0 {
        return Err(ExpressionError::ScanError(
            "single logic_or, or single logic_and, or unclosing acute, or unclosing parenthesis"
                .to_string(),
        ));
    }

    tokens.push(Token::RightParenthesis);

    Ok(tokens)
}
