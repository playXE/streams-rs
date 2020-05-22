use crate::*;
/// Basic stream type. To use it you should provide getter function: 
/// ```rust
/// use streams_rs::{*,fn_stream::*};
/// let get = (|| StreamResult::Ok(1));
/// let mut stream = FnStream::new(get);
/// stream.token(0); // 1
/// stream.token(1); // 1
/// ```
pub struct FnStream<'a, T: Clone> {
    get: Box<dyn FnMut() -> StreamResult<T> + 'a>,
    pos: usize,
    cache: Vec<Option<T>>,
}

impl<'a, T: Clone> FnStream<'a, T> {
    pub fn new(get: impl FnMut() -> StreamResult<T> + 'a) -> Self {
        Self {
            get: Box::new(get),
            pos: 0,
            cache: vec![],
        }
    }
    
}
impl<'a,T: Clone> Stream<'a> for FnStream<'a,T> {
    type Item = T;
     fn token(&mut self, x: usize) -> StreamResult<T> {
        if let Some(Some(tmp)) = self.cache.get(x) {
            return StreamResult::Ok(tmp.clone());
        }
        let mut n = self.cache.len();
        let mut c = (0..x + 1).map(|_| None).collect::<Vec<_>>();
        while n <= x {
            let p =(self.get)();
            if let StreamResult::Ok(val) = p {
                c[n] = Some(val);
                n += 1;
            } else if let StreamResult::Err(StreamError::Str(_)) = p {
                return p;
            } else {
                c[n] = None;
            }
        }
        self.cache = c;
        StreamResult::Ok(self.cache[x].as_ref().unwrap().clone())
    }

    fn junk(&mut self, mut x: usize) {
        let c = self.cache.len();
        self.pos += x;
        if c >= x {
            self.cache.truncate(c - x);
        } else {
            self.cache = vec![];
            x = x - c;
            while x > 0 {
                (self.get)();
                x = x - 1;
            }
        }
    }

    fn pos(&self) -> usize {
        self.pos
    }
}