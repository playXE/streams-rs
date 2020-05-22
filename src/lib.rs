pub mod fn_stream;
pub mod io_stream;
pub type StreamGetFn<'a,T> = dyn FnMut() -> Option<T> + 'a; 

pub enum StreamResult<T> {
    Ok(T),
    Err(StreamError)
}

impl<T: fmt::Debug> fmt::Debug for StreamResult<T> {
    fn fmt(&self,f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok(v) => write!(f,"Ok({:?})",v),
            Self::Err(e) => write!(f,"Err({:?})",e),
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum StreamError {
    EmptyStream,
    NotHandledPattern,
    Str(String),
}

use std::fmt;
impl fmt::Display for StreamError {
    fn fmt(&self,f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::EmptyStream => write!(f,"empty stream"),
            StreamError::NotHandledPattern => write!(f,"stream pattern not handled"),
            StreamError::Str(s) => write!(f,"stream error: '{}'",s),
        }
    }
}


/// Suppose you need to process each line of a text file. One way to do this is to read the file in as a single large string and use something
/// like `split` to turn it into a list. This works when the file is small, but because the entire file is loaded into memory, it does not
/// scale well when the file is large.
/// 
/// More commonly, the read_line function can be used to read one line at a time from a channel. This typically looks like:
/// ```rust 
///     // LineReadStream is just example, you can implement your own stream for reading files.
///     use streams_rs::*;
///     use io_stream::*;
///     let mut c = std::io::Cursor::new(b"a\n b\n c");
///     let mut stream = LineReadStream::new(&mut c);
///     let _ = smatch!(match (stream) {
///         [a=> b=> c =>] => {
///             println!("{:?} {:?} {:?}",a,b,c);
///         } 
///     }); 
/// ```
/// 
pub trait Stream<'a> {
    type Item;
    /// Returns token at `x`. It's possible to cache values. FnStream for example caches all tokens returned from getter.
    fn token(&mut self,x: usize) -> StreamResult<Self::Item>;
    /// Basically same as truncating vector.
    fn junk(&mut self,x: usize);
    /// Return stream position, does not used by macro or streams-rs, but might be usefull for someone.
    fn pos(&self) -> usize;
}

/// Macro for matching on streams. 
/// 
/// This macro will not catch any error for you so your getter should return `StreamResult<Result>` if you might have any errors. 
/// If value not matched then `StreamResult::Err(StreamError::EmptyStream)` is returned.
/// 
#[macro_export]
macro_rules! smatch {
    (@p $body: expr;$stream: expr,$cnt: expr;  -> $($pat: tt)*) => {{

            smatch!(@p $body; ; $stream,$cnt    ;$($pat)*)

    }
    };
    (@p $body: expr; $($assign: ident),*; $stream: expr,$cnt: expr; _ => $($rest:tt)*) => {
        {let res = $stream.token($cnt);
            match res {
                StreamResult::Ok(_) => {smatch!(@p $body; $name; $stream,$cnt + 1;$($rest)*)},
                _ => (None,false)
            }
        /*$(
            let $assign = $assign;
        )**/
        

        }
    };
    (@p $body: expr; $($assign: ident),*; $stream: expr,$cnt: expr; $name : ident => $($rest:tt)*) => {
        {let p = $stream.token($cnt);
        if let StreamResult::Ok($name) = p {
            smatch!(@p $body; $name; $stream,$cnt + 1;$($rest)*)
        } else {
            (None,false)
        }
        }
    };
    (@p $body:expr;$($assign: ident),*;$stream: expr,$cnt: expr; $p: pat => $($rest:tt)*) => {{
        let p = $stream.token($cnt);
        if let StreamResult::Ok($p) = p {
            
            $(
                let $assign = $assign;
            )*
            smatch!(@p $body; ; $stream,$cnt + 1;$($rest)*)
        } else {
            
            (None,false)
        }
    }
    };
    /*(@p $body: expr;$($assign: ident),*;$stream: expr,$cnt: expr;  $name : ident = $p: pat => $($rest:tt)*) => {
        {let x = $stream.token($cnt);
        if let $p = x {
            let $name = x;
            $(
                let $assign = $assign;
            )*
            smatch!(@p $body; $($assign),*$name; $stream,$cnt + 1;$($rest)*)
        } else {
            (None,false)
        }}
    };*/
    (@p $body: expr; $($assign: ident),*; $stream: expr,$cnt: expr; $name : ident = $p: expr => $($rest:tt)*) => {
        {let $name = $p;
        /*$(
            let $assign = $assign;
        )**/
        smatch!(@p $body; $name; $stream,$cnt + 1;$($rest)*)
        }
    };

    (@p $body: expr; $($assign: ident),*;$stream: expr,$cnt: expr;) => {
        {$stream.junk($cnt);
        (Some($body),true)
        }
    };

    (match ($s: expr) {
        $(
            [ $($p: tt)* ] => $body: expr
        ),*
    }) => {
        loop {
        $(
            let res =  smatch!(@p $body; $s,0; -> $($p)*);
            if let (Some(val),true) = res {
                break $crate::StreamResult::Ok(val);
            }
        )*
        break $crate::StreamResult::Err($crate::StreamError::EmptyStream)
        }
    };

}