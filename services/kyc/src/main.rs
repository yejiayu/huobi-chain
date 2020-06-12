mod work;

use work::*;
/*
() > . > @ > ! > & > | > ()
 */
fn main(){
    //let str = " ! ( X.Y @ `123 456` )  ".to_string();
    //let str = " !(a|b) && X.Y@3  ".to_string();
    //let str = " A || !X.B && C ".to_string();
    let str = " A.B@`C` || !A.B@`C` && A.B@`C` ".to_string();//true
    println!("processing: {}",str);
    if let Ok(vec)= scan(str){
        println!("{:?}",vec);
        let node = parse(vec);
        let res = calculation(&node);
        println!("res:{:?}",res)
    }
}