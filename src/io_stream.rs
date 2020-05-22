use std::io::{BufRead,Read};
use crate::*;
use std::borrow::Cow;
use std::io::ErrorKind;
pub struct LineReadStream<'a,T: BufRead> {
    pub value: &'a mut T,
    pub cache: Vec<Option<String>>,
}
impl<'a,T: BufRead> LineReadStream<'a,T> {
    pub fn new(item: &'a mut T) -> Self {
        Self {
            value: item,
            cache: vec![]
        }
    }
}

impl<'b,'a: 'b,T: BufRead> Stream<'a> for LineReadStream<'a,T> {
    type Item = Option<String>;

    fn token(&mut self,x: usize) -> StreamResult<Option<String>> {
        if self.cache.get(x).is_some() {
            return match &self.cache[x] {
                Some(x) => StreamResult::Ok(Some(x.clone())),
                _ => StreamResult::Ok(None)
            }
        }
        let mut n = self.cache.len();
        let mut c = (0..x + 1).map(|_| None).collect::<Vec<_>>();
        while n <= x {
            let mut buf = String::new();
            let p = self.value.read_line(&mut buf);
            match p {
                Ok(0) => {
                    c[n] = None;
                }
                Ok(_) => {
                    c[n] = Some(buf);
                    n += 1;
                }
                Err(e) => return StreamResult::Err(StreamError::Str(e.to_string())),
            }
           
        }
        self.cache = c;
        StreamResult::Ok(self.cache[x].clone())
    }

    fn junk(&mut self,mut x: usize) {
        let c = self.cache.len();
        
        if c >= x {
            self.cache.truncate(c - x);
        } else {
            self.cache = vec![];
            x = x - c;
            while x > 0 {
                
                let _ = self.value.read_line(&mut String::new());
                x = x - 1;
            }
        }
    }

    fn pos(&self) -> usize {
        0
    }
    
}