mod work;

use work::*;
/*
() > . > @ > ! > & > | > ()
 */
fn main(){
    //let str = " ! ( X.Y @ `123 456` )  ".to_string();
    //let str = " !(a|b) && X.Y@3  ".to_string();
    let str = " A || !X.B && C ".to_string();
    println!("processing: {}",str);
    if let Ok(vec)= scan(str){
        println!("{:?}",vec);

        parse(vec);
    }
}