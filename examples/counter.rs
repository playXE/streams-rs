extern crate streams_rs;

use streams_rs::fn_stream::FnStream;
use streams_rs::*;
fn main() {
    let mut count = 0;
    let get = || {
        count += 1;
        StreamResult::Ok(count - 1)
    };
    let mut stream = FnStream::new(get);

    let _ = smatch!(match (stream) {
        [0=>] => {  // if not 0 then we do not match
            println!("Zero!");
        }
    });
    let _ = smatch!(match (stream) {
        [1=>] => { // if not 1 then we do not match
            println!("One!");
        }
    });
    for _ in 0..10 {
        let _ = smatch!(match (stream) {
            [a=>b=>] => { // get two values a and b from stream
                println!("a: {} b: {}",a,b);
            }
        });
    }
}