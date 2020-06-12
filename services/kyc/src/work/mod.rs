
use std::error::Error;
use std::ops::BitAnd;
use derive_more::Display;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

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
    Identifier(String)
}

impl Token{
    pub fn to_Node(self,left:Option<Node>,right:Option<Node>,closed:bool) ->Node{
        let mut node = Node{
            token:self,
            left:None,
            right:None,
            closed
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
            Token::Dot => 10,
            Token::Has => 9,
            Token::Not => 8,
            Token::And => 7,
            Token::Or => 6,
            Token::Value(_) => 0,
            Token::Identifier(_) => 0,
            _ => 0,
        }
    }

    // 0 for N/A, 1 for left, 2 for right
    pub fn get_associativity(&self) -> u8 {
        match self{
            Token::And => 1,
            Token::Or => 1,
            Token::Not => 2,
            _ => 0,
        }
    }

    pub fn must_have_left(&self) -> bool{
        match self{
            Token::Has | Token::Dot | Token::And | Token::Or => true,
            _ => false,
        }
    }

    pub fn can_have_right(&self) -> bool{
        match self{
            Token::Has | Token::Dot | Token::And | Token::Or | Token::Not => true,
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
    pub closed : bool,
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
    let mut parenthesis: u8 = 0;

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

    for new_pos in position {
        start = end;
        end = new_pos;
        let token_str = (&char_sequence[start..end]).iter().collect::<String>();

        if acute {
            if token_str == "`" {
                acute = false
            } else {
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
            tokens.push(Token::Dot);

        }else if token_str == "!" {
            tokens.push(Token::Not);
        } else if token_str == "|" {
            if prev_token == "|"{
                continue;
            }
            tokens.push(Token::Or);

        } else if token_str == "&" {
            if prev_token == "&"{
                continue;
            }
            tokens.push(Token::And);

        } else if token_str == "@" {
            tokens.push(Token::Has);

        } else if token_str == "(" {
            tokens.push(Token::LeftParenthesis);

        } else if token_str == ")" {
            tokens.push(Token::RightParenthesis);

        } else if token_str == " "{
            continue
        }else {
            tokens.push(Token::Identifier(token_str.clone()));
        }

        prev_token = token_str;
    }

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
pub fn parse(mut tokens :Vec<Token>){

    let mut nodes = Vec::<Node>::new();

    for token in tokens.into_iter(){
        println!("parsing:{}",token);
        let a = 1;
        match token {
            Token::And | Token::Or | Token::Dot | Token :: Has =>{

                if let Some(mut prev_node) = nodes.pop(){

                    println!("  prev_node:{:?}",prev_node);

                    //prev_node is ident or value, which means it must not have children
                    if prev_node.is_ident_or_value(){

                        let node = token.to_Node(Some(prev_node),None,false);
                        nodes.push(node);

                    }else if prev_node.has_right(){

                        if  prev_node.token.get_priority() < token.get_priority() && !prev_node.closed{
                            //prev has right child
                            // a+b*c
                            //move the ident/value from prev's right to the current's left

                            let right_node = *(prev_node.right.take()).unwrap();
                            let node = token.to_Node(Some(right_node),None,false);
                            nodes.push(prev_node);
                            nodes.push(node);
                        }else if prev_node.token.get_priority() > token.get_priority(){
                            // a* b + 3
                            let node = token.to_Node(Some(prev_node),None,false);
                            nodes.push(node);
                        }else {
                            //in our case, every operator's priority is different
                            //in classic case, every operator having same priority has same associativity
                            if token.get_associativity()  == 1{
                                //left
                                let node = token.to_Node(Some(prev_node),None,false);
                                nodes.push(node);
                            }else if token.get_associativity() == 2{
                                //right
                                let right_node = *prev_node.right.unwrap();
                                let node = token.to_Node(Some(right_node),None,false);
                                nodes.push(node);
                            }else{
                                panic!()
                            }
                        }
                    }else if !prev_node.token.must_have_left(){
                        nodes.push(prev_node);
                        nodes.push(token.to_Node(None,None,false));
                    }else{
                        panic!()
                    }
                }
                else{
                    let node = token.to_Node(None,None,false);
                    if !node.token.must_have_left(){
                        nodes.push(node);
                    }else{
                        panic!()
                    }
                }
            }
            Token ::Not => {
                nodes.push(token.to_Node(None,None,false));
            }
            Token::Identifier(_) | Token::Value(_)=>{
                if let Some(mut prev_node) = nodes.pop(){
                    println!("  prev_node:{:?}",prev_node);

                    // a + b
                    if prev_node.token.can_have_right() && !prev_node.has_right() {
                        prev_node.right = Some(Box::new(token.to_Node(None,None,true)));
                        nodes.push(prev_node)
                    }
                    else {
                        nodes.push(prev_node);
                        nodes.push(token.to_Node(None,None,true));
                    }
                }else{
                    nodes.push(token.to_Node(None,None,true));
                }

            },
            Token::LeftParenthesis=>{
                nodes.push(token.to_Node(None,None,false));
            },
            Token::Whitespace|Token::Acute =>{panic!()},

            Token::RightParenthesis=>{

                loop{

                    if nodes.is_empty(){
                        panic!("unclosing }")
                    }
                    let mut prev_node = nodes.pop().unwrap();
                    println!("  prev_node:{:?}",prev_node);
                    if let Token::LeftParenthesis = prev_node.token {
                        panic!(") follows (, meaningless")
                    }
                    if nodes.is_empty(){
                        panic!("* ), unclosing")
                    }
                    let mut penultimate_node = nodes.pop().unwrap();
                    println!("  penultimate_node:{:?}",penultimate_node);

                    // ( ? )
                    if let Token::LeftParenthesis = penultimate_node.token {
                        //we are on the last step of closing parenthesis

                        if !prev_node.closed {
                            prev_node.closed = true;
                        }
                        //check if the ? inside () is correct
                        //todo

                        penultimate_node.closed = true;

                        penultimate_node.right = Some(Box::new(prev_node));


                        if let Some(mut antepenultimate_node) =nodes.pop(){
                            println!("  antepenultimate_node:{:?}",antepenultimate_node);

                            // + ( ? )
                            //把这个括号挂载到之前的节点
                            if antepenultimate_node.token.can_have_right() && !antepenultimate_node.has_right(){
                                if antepenultimate_node.right.is_none(){
                                    antepenultimate_node.right=Some(Box::new(penultimate_node));
                                    nodes.push(antepenultimate_node);
                                }
                                else{
                                    panic!()
                                }
                            }
                            // ? ( ? )
                            else{
                                nodes.push(antepenultimate_node);
                                nodes.push(penultimate_node)
                            }
                        }
                        //we are at begin
                        else{
                            nodes.push(penultimate_node);
                        }

                        break;
                    }else if penultimate_node.closed == false && penultimate_node.token.can_have_right() && !penultimate_node.has_right(){
                        //we are not on the last step
                        // + ? )
                        //since ) is the lowest, the prenultimate must take the prev
                        penultimate_node.right = Some(Box::new(prev_node));
                        penultimate_node.closed = true;
                        nodes.push(penultimate_node);
                    } else if penultimate_node.closed == true || penultimate_node.right.is_some(){
                        panic!()
                    }
                }
            },
        }
    }

    //do last (), since the whole expr is inside a global ( )

    loop{
        if nodes.len() > 1{
            let mut prev_node = nodes.pop().expect("");
            let mut penultimate_node = nodes.pop().expect("");

            if penultimate_node.closed == false && penultimate_node.token.can_have_right() && !penultimate_node.has_right(){
                penultimate_node.right = Some(Box::new(prev_node));
                nodes.push(penultimate_node);
            }
            else{
                nodes.push(penultimate_node);
                nodes.push(prev_node);
                break;
            }
        }else{
            break;
        }
    }

    println!("nodes:\n{:?}",nodes);

    if nodes.len() != 1{
        println!("nodes.len{:?}",nodes.len());

        panic!()
    }

}
#[derive(Display,Debug)]
pub enum KYCError {
    #[display(fmt = "{}", _0)]
    error(String),
}

impl Error for KYCError {}