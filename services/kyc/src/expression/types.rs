use derive_more::Display;
use std::error::Error;

use protocol::types::{Address};

#[derive(Debug,Display)]
pub enum Token {
    #[display(fmt = "LeftParenthesis")]
    LeftParenthesis,
    #[display(fmt = "RightParenthesis")]
    RightParenthesis,
    #[display(fmt = "Whitespace")]
    Whitespace,
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
    #[display(fmt = "Acute")]
    Acute,
    #[display(fmt = "Value{}",_0)]
    Value(String),
    #[display(fmt = "Identifier{}",_0)]
    Identifier(String),
}

#[derive(Debug)]
pub struct Node{
    pub token : Token,
    pub left : Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct CalcContext<'a,DF>{
    pub target_address : Address,
    pub data_feeder: &'a DF,
}

#[derive(Debug)]
pub enum CalcValue {
    KycTag(Vec<String>),
    Bool(bool),
    Ident(String),
    //this value is for the type of Token::Value
    Value(String),
}

#[derive(Display,Debug,PartialEq)]
pub enum CalcErr{
    CalcError(String)
}
impl Error for CalcErr {}
impl Into<KYCError> for CalcErr{
    fn into(self) -> KYCError {
        match self{
            CalcErr::CalcError(str) => KYCError::CalcError(str)
        }
    }
}

pub type CalcResult = Result<CalcValue,CalcErr>;

#[derive(Display,Debug,PartialEq)]
pub enum KYCError {
    #[display(fmt = "{}", _0)]
    ScanError(String),

    #[display(fmt = "{}", _0)]
    ParseError(String),

    #[display(fmt = "{}", _0)]
    CalcError(String),
}

impl Error for KYCError {}


pub type ExpressionResult = Result<bool,KYCError>;