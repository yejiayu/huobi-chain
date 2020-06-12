
use std::error::Error;
use std::ops::{BitAnd, Add, RangeInclusive};
use derive_more::Display;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::collections::{HashMap, VecDeque};
use std::borrow::Borrow;

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

    //below won't occur in expression

    #[display(fmt = "HighestMark,only for mark highest priority")]
    HighestMark,
    //to contain which KYC.TAG is need
    #[display(fmt = "Tag{}",_0)]
    Tag(String),
    //logical result, which is to contain KYC.TAG@VALUE's result
    #[display(fmt = "Result{}",_0)]
    LogicalResult(String)
}

impl Token{
    pub fn to_Node(self,left:Option<Node>,right:Option<Node>) ->Node{
        let mut node = Node{
            token:self,
            left:None,
            right:None,
        };
        if let Some(left_node) = left{
            node.left = Some(Box::new(left_node))
        };
        if let Some(right_node) = right{
            node.right = Some(Box::new(right_node))
        };
        node
    }
    /*
    () > . > @ > ! > & > | > ()
     */
    pub fn get_priority(&self) -> u8{
        match self{
            Token::HighestMark => 6,
            Token::LeftParenthesis => 0,
            Token::RightParenthesis => 6,
            Token::Dot => 5,
            Token::Has => 4,
            Token::Not => 3,//right
            Token::And => 2,
            Token::Or => 1,
            Token::Value(_) => 0,
            Token::Identifier(_) => 0,
            _ => unreachable!(),
        }
    }

    pub fn must_have_left(&self) -> bool{
        match self{
            Token::Has | Token::Dot | Token::And | Token::Or => true,
            _ => false,
        }
    }

    pub fn must_have_right(&self) -> bool{
        match self{
            Token::Has | Token::Dot | Token::And | Token::Or | Token::Not => true,
            _ => false,
        }
    }

    //true for left
    pub fn get_associativity(&self) -> bool{
        match self{
            Token::Dot |
            Token::Has |
            Token::And |
            Token::Or  => true,
            Token::Not => false,
            _ => unreachable!(),
        }
    }
    //1 for left, 2 for right, 0 for error
    pub fn get_associativity_by_priority(priority : u8) -> u8{
        if priority == 3{
            return 2
        }else if priority == 0{
            return 0
        }
        1
    }


    pub fn is_ident_or_value(&self) -> bool{
        match self{
            Token::Value(_) | Token::Identifier(_) | Token::RightParenthesis => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool{
        match self{
            Token::Dot |
            Token::Has |
            Token::Not |
            Token::And |
            Token::Or  => true,
            _ => false,
        }
    }
}


//we don't have triple-operate
#[derive(Debug)]
pub struct Node{
    pub token : Token,
    pub left : Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl Node {
    fn is_ident_or_value(&self) -> bool{
        match self.token{
            Token::Value(_) | Token::Identifier(_) => true,
            _ => false,
        }
    }

    fn has_right(&self) -> bool{
        self.right.is_some()
    }

    fn has_left(&self) -> bool{
        self.left.is_some()
    }

}


pub fn scan(input: String) -> Result<Vec<Token>, KYCError> {
    let mut acute = false;

    let mut position = Vec::<usize>::new();
    position.push(0);
    for (i, c) in input.chars().enumerate() {
        if !c.is_ascii() {
            return Err(KYCError::error("not ascii".to_string()));
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
    }//all operators have been marked
    position.push(input.chars().count());

    if acute {
        return Err(KYCError::error("unclosing acute".to_string()));
    }

    let mut start: usize = 0;
    let mut end: usize = 0;
    let mut prev_token="".to_string();
    let temp = input.chars().collect::<Vec<char>>();
    let char_sequence = temp.as_slice();
    let mut tokens = Vec::<Token>::new();
    tokens.push(Token::LeftParenthesis);
    let mut operator_priority_count_map = HashMap::<u8,u32>::new();
    for new_pos in position {
        start = end;
        end = new_pos;
        let token_str = (&char_sequence[start..end]).iter().collect::<String>();

        if acute {
            if token_str == "`" {
                acute = false
            } else {
                let mut count  = operator_priority_count_map.entry(Token::Value("".to_string()).get_priority()).or_insert(0);
                *count = *count + 1;
                tokens.push(Token::Value(token_str.clone()));
            }
            continue;
        }

        if token_str == "" {
            continue;
        } else if token_str == "`" {
            acute = true;
            continue;
        } else if token_str == "." {
            let count = operator_priority_count_map.entry(Token::Dot.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::Dot);

        }else if token_str == "!" {
            let count = operator_priority_count_map.entry(Token::Not.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::Not);
        } else if token_str == "|" {
            if prev_token == "|"{
                continue;
            }
            let count = operator_priority_count_map.entry(Token::Or.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::Or);

        } else if token_str == "&" {
            if prev_token == "&"{
                continue;
            }
            let count = operator_priority_count_map.entry(Token::And.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::And);

        } else if token_str == "@" {
            let count = operator_priority_count_map.entry(Token::Has.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::Has);

        } else if token_str == "(" {
            let count = operator_priority_count_map.entry(Token::LeftParenthesis.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::LeftParenthesis);

        } else if token_str == ")" {
            let count = operator_priority_count_map.entry(Token::RightParenthesis.get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::RightParenthesis);

        } else if token_str == " "{
            continue
        }else {
            let count = operator_priority_count_map.entry(Token::Identifier("".to_string()).get_priority()).or_insert(0);
            *count = *count + 1;
            tokens.push(Token::Identifier(token_str.clone()));
        }

        prev_token = token_str;
    }
    tokens.push(Token::RightParenthesis);
    println!("tokens:{:?}",tokens);
    println!("priority_count:{:?}",operator_priority_count_map);

    Ok(tokens)
}

/*
() > . > @ > ! > & > | > ()

Ident Value 只能作为叶子节点
运算符的子节点必须都满足
运算符优先级高的在下
在成为子树的时候 必须closed
当节点的左叶子没有准备好的时候,右叶子被处理了,说明这是不对的
 */
pub fn parse(mut tokens :Vec<Token>) -> Node{


    let mut nodes = tokens.into_iter().map(|token| {
        token.to_Node(None,None)
    }).collect::<VecDeque<Node>>();


    let a = 1;

    //because parenthesis is the highest outside but lowest inside, so we separately deal it
    let mut parenthesis_stack = Vec::new();

    let mut index = 0;
    loop  {

        if nodes.is_empty(){
            panic!();
        }

        let node = nodes.get_mut(index).expect("");
        println!("parsing:{:?}",node.token);
        match node.token{
            Token::LeftParenthesis => parenthesis_stack.push(index),
            Token::RightParenthesis => {
                let left_parenthesis_index = parenthesis_stack.pop().expect("un-closing parenthesis, need left parenthesis");
                let mut end = nodes.split_off(index+1);
                let mut piece = nodes.split_off(left_parenthesis_index);
                if let Token::LeftParenthesis = piece.pop_front().expect("").token {
                }else{
                    panic!()
                };
                if let Token::RightParenthesis = piece.pop_back().expect("").token {
                }else{
                    panic!()
                };
                //parse (left, right)
                let parsed_parenthesis = parse_internal(piece);
                nodes.push_back(parsed_parenthesis);
                nodes.append(&mut end);

                index = left_parenthesis_index;
            }
            _ => ()
        }

        index += 1;

        if parenthesis_stack.is_empty(){
            break;
        }
    }

    if nodes.len() > 1{
        panic!()
    };

    let node = nodes.pop_back().expect("");
    println!("res:\n{:?}", node);
    node
}

//we scan all operator from highest and combine them. I new this method is quite slow, but is less of faults
fn parse_internal(mut nodes : VecDeque<Node>) -> Node{
    let mut priority = Token::HighestMark.get_priority() -1;
    while priority > 0 {
        let mut index = 0;

        let reverse_mode = Token::get_associativity_by_priority(priority) == 2;
        if reverse_mode{
            nodes = reverse(nodes)
        }
        loop{

            let node = nodes.get(index);
            if node.is_none(){
                break
            }
            let node = node.expect("");
            if node.token.get_priority() != priority{
                index += 1;
                continue
            }
            println!("  parsing:{:?}",node);

            let mut left = false ;
            let mut right = false;
            if node.token.must_have_left(){
                if !reverse_mode {
                    left = true
                }else{
                    right =true
                }
            }
            if node.token.must_have_right(){
                if !reverse_mode {
                    right = true
                }else{
                    left = true
                }
            }
            //index is seting to node
            if left {
                let mut node = nodes.remove(index).expect("");
                if let Some(left) = nodes.remove(index-1){
                    if !reverse_mode {
                        node.left = Some(Box::new(left));
                    }
                    else{
                        node.right = Some(Box::new(left));
                    }
                    nodes.insert(index-1,node);
                    //set index to node
                    index -= 1;
                }else{
                    panic!()
                }
            }


            if right{
                let mut node = nodes.remove(index).expect("");
                if let Some(right) = nodes.remove(index){
                    if !reverse_mode{
                        node.right = Some(Box::new(right));
                    }else{
                        node.left = Some(Box::new(right));
                    }
                    nodes.insert(index,node);
                }else{
                    panic!()
                }
            }

            index +=1
        }

        if reverse_mode{
            nodes = reverse(nodes)
        }

        priority -= 1
    }


    if nodes.len() > 1{
        panic!()
    };

    nodes.pop_back().expect("")
}



fn reverse(mut nodes :VecDeque<Node>) -> VecDeque<Node>{
    let mut temp = VecDeque::new();
    while let Some(node) = nodes.pop_front(){
        temp.push_front(node);
    }
   temp
}

pub enum CalcOK {
    KycTag(Vec<String>),
    Bool(bool),
    Ident(String),
    Value(String),
}

type CalcResult = Result<CalcOK,String>;

pub fn calculation(node: &Node) -> Result<bool,String>{
    match calc(node)?{
        CalcOK::Bool(b) => Ok(b),
        _ => Err("calculation fails".to_string())
    }
}

pub fn calc(node: & Node) -> CalcResult {
    match node.token {
        Token::Dot =>{
            calc_dot(node)
        },
        Token::Has => {
            calc_has(node)
        },
        Token::Not =>{
            calc_not(node)
        },
        Token::And =>{
            calc_and(node)
        },
        Token::Or =>{
            calc_or(node)
        },
        Token::Value(_) =>{
            calc_value(node)
        },
        Token::Identifier(_) =>{
            calc_ident(node)
        },
        _ => unreachable!("wrong operation"),
    }
}

fn calc_ident(ident_node : &Node) -> CalcResult {
    if let Token::Identifier(s) = &ident_node.token{
        Ok(CalcOK::Ident(s.to_owned()))
    }else{
        Err("calc_ident get a wrong node".to_string())
    }
}

fn calc_value(value_node : &Node) -> CalcResult {
    if let Token::Value(s) = &value_node.token{
        Ok(CalcOK::Value(s.to_owned()))
    }else{
        Err("calc_value get a wrong node".to_string())
    }
}

fn calc_dot(dot_node:&Node) -> CalcResult {
    let mut left = "".to_string();
    if let Some(kyc_node) = dot_node.left.as_ref(){
        match calc(kyc_node)?{
            CalcOK::Ident(str) => {left = str},
            _ => return Err("dot operation's left performs wrong".to_string())
        }
    }else{
        return Err("dot operation's left param is missing".to_string());
    }

    let mut right = "".to_string();
    if let Some(tag_node) = dot_node.right.as_ref(){
        match calc(tag_node)?{
            CalcOK::Ident(str) => {right = str},
            _ => return Err("dot operation's right performs wrong".to_string())
        }
    }else{
        return Err("dot operation's right param is missing".to_string());
    }
    //todo, calc KYC.TAG
    Ok(CalcOK::KycTag(vec!["KYC.TAG".to_string()]))
}


fn calc_has(has_node:&Node) -> CalcResult {

    let mut kyc_tags = Vec::new();

    if let Some(kyc_tag_node) = has_node.left.as_ref(){
        match calc(kyc_tag_node)?{
            CalcOK::KycTag(values) => {kyc_tags = values},
            _ => return Err("has operation's left performs wrong".to_string())
        }
    }else{
        return Err("has operation's left param is missing".to_string());
    }

    let mut value = "".to_string();
    if let Some(value_node) = has_node.right.as_ref(){
        match calc(value_node)?{
            CalcOK::Value(val) => {value = val},
            _ => return Err("has operation's right performs wrong".to_string())
        }
    }else{
        return Err("has operation's right param is missing".to_string());
    }
    //todo, calc KYC.TAG@VALUE
    Ok(CalcOK::Bool(true))
}


fn calc_not(not_node:&Node) -> CalcResult {
    if let Some(empty_node) = not_node.left.as_ref(){
        return Err("not operation's shouldn't have left param".to_string());
    }else{
    }

    let mut right = false;
    if let Some(expr_node) = not_node.right.as_ref(){
        match calc(expr_node)?{
            CalcOK::Bool(b) => {right = b},
            _ => return Err("not operation's right performs wrong".to_string())
        }
    }else{
        return Err("not operation's right param is missing".to_string());
    }

    Ok(CalcOK::Bool(!right))
}

fn calc_and(and_node:&Node) -> CalcResult {

    let mut left =false;

    if let Some(expr_node) = and_node.left.as_ref(){
        match calc(expr_node)?{
            CalcOK::Bool(b) => {left = b},
            _ => return Err("and operation's left performs wrong".to_string())
        }
    }else{
        return Err("and operation's left param is missing".to_string());
    }

    let mut right = false;
    if let Some(expr_node) = and_node.right.as_ref(){
        match calc(expr_node)?{
            CalcOK::Bool(b) => {right = b},
            _ => return Err("and operation's right performs wrong".to_string())
        }
    }else{
        return Err("and operation's right param is missing".to_string());
    }

    Ok(CalcOK::Bool(left&&right))
}

fn calc_or(or_node:&Node) -> CalcResult {

    let mut left =false;

    if let Some(expr_node) = or_node.left.as_ref(){
        match calc(expr_node)?{
            CalcOK::Bool(b) => {left = b},
            _ => return Err("or operation's left performs wrong".to_string())
        }
    }else{
        return Err("or operation's left param is missing".to_string());
    }

    let mut right = false;
    if let Some(expr_node) = or_node.right.as_ref(){
        match calc(expr_node)?{
            CalcOK::Bool(b) => {right = b},
            _ => return Err("or operation's right performs wrong".to_string())
        }
    }else{
        return Err("or operation's right param is missing".to_string());
    }

    Ok(CalcOK::Bool(left||right))
}

#[derive(Display,Debug)]
pub enum KYCError {
    #[display(fmt = "{}", _0)]
    error(String),
}

impl Error for KYCError {}