

// use std::fmt::Write;
use std::io;
use std::io::Write; 
use std::path::Path; 
use crate::InputSeq;
use crate::error::Result;
use crate::status::{TextBuffer, Status};
use crate::screen::Screen;  

/*
 * This is the Editor file for the Terminal Editor
 * Let's write some code here
 */

enum EditStep {
    Continue(io::Stdin),
    Quit, 
}


impl EditStep {
    fn continues(&self) -> bool {
        match self {
            EditStep::Continue(_) => true, //default input from the user 
            EditStep::Quit => false, 
        }
    }
}





pub struct Editor<I: Iterator<Item = Result<InputSeq>>, W: Write>{
    input: I, 
    qutting: bool, 
    screen: Screen<W>,
    bufs: Vec<TextBuffer>, 
    buf_idx: usize,
    //status_bar: Status, 
}


impl<I, W> Editor<I, W>
    where
        I: Iterator<Item = Result<InputSeq>>, 
        W: Write,
{
    fn with_buf(
        buf: TextBuffer, 
        mut input: I, 
        output: W,
        window_size: Option<(usize, usize)>,
    ) -> Result<Editor<I, W>> {

        let status_bar = Status::from_buffer(&buf, (1, 1)); 
        let screen = Screen::new(window_size, &mut input, output)?; 

     

        Ok(Editor {
            input, 
            qutting: false, 
            bufs: vec![buf], 
            buf_idx: 0,
            screen
        })
    }

    pub fn new(input: I, output: W, window_size: Option<(usize, usize)>) -> Result<Editor<I, W> >{
        //return the buffer here
        Self::with_buf(TextBuffer::empty(), input, output, window_size)
    }

    pub fn buf(&self)  {
        //self.bufs[self.buf_idx]
    }

    pub fn buf_mut(&mut self)  {
        //&mut self.bufs[self.buf_idx]
    }

    pub fn open<P: AsRef<Path>>(
        mut input: I, 
        output: W, 
        window_size: Option<(usize, usize)>, 
        paths: &[P], 
    ) -> Result<Editor<I, W>> {

        if paths.is_empty() {
            return Self::new(input, output, window_size); 
        }

        

        let screen = Screen::new(window_size, &mut input, output)?;
        let bufs: Vec<_> = paths.iter().map(TextBuffer::open).collect::<Result<_>>()?; 
        

        Ok(Editor {
            input, 
            qutting: false, 
            buf_idx: 0, 
            bufs,
            screen,

        })
    }
 }
    



//we want to share state here
//how do we solve this 
//we could use generics, we could use trait and eums for various properties
//