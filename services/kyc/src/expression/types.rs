use core::fmt;
use std::fmt::Display;

use derive_more::Display;

use protocol::types::Address;

#[derive(Debug, Display)]
pub enum Token {
    #[display(fmt = "LeftParenthesis")]
    LeftParenthesis,
    #[display(fmt = "RightParenthesis")]
    RightParenthesis,
    // #[display(fmt = "Whitespace")]
    // Whitespace,
    #[display(fmt = "And")]
    And,
    #[display(fmt = "Or")]
    Or,
    #[display(fmt = "Not")]
    Not,
    #[display(fmt = "Dot")]
    Dot,
    #[display(fmt = "Has")]
    Has,
    // #[display(fmt = "Acute")]
    // Acute,
    #[display(fmt = "Value{}", _0)]
    Value(String),
    #[display(fmt = "Identifier{}", _0)]
    Identifier(String),
}

#[derive(Debug)]
pub struct Node {
    pub token:  Token,
    pub left:   Option<Box<Node>>,
    pub right:  Option<Box<Node>>,
    // this indicate that this node'token has been parsed, and only can be other's child,
    pub parsed: bool,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.left.is_some() && self.right.is_some() {
            write!(
                f,
                "{{{}, L:{}, R:{}, p:{}}}",
                self.token,
                self.left.as_ref().expect("").as_ref(),
                self.right.as_ref().expect("").as_ref(),
                self.parsed
            )
        } else if self.left.is_none() && self.right.is_some() {
            write!(
                f,
                "{{{}, L:null, R:{}, p:{}}}",
                self.token,
                self.right.as_ref().expect("").as_ref(),
                self.parsed
            )
        } else if self.left.is_some() && self.right.is_none() {
            write!(
                f,
                "{{{}, L:{}, R:null, p:{}}}",
                self.token,
                self.left.as_ref().expect("").as_ref(),
                self.parsed
            )
        } else {
            write!(f, "{{{}, L:null, R:null, p:{}}}", self.token, self.parsed)
        }
    }
}

#[derive(Debug)]
pub struct CalcContext<'a, DF> {
    pub target_address: Address,
    pub data_feeder:    &'a DF,
}

#[derive(Debug)]
pub enum CalcValue {
    KycTag(Vec<String>),
    Bool(bool),
    Ident(String),
    // this value is for the type of Token::Value
    Value(String),
}
