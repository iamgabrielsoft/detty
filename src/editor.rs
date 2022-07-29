

// use std::fmt::Write;
use std::io;
use std::io::Write; 
use std::path::Path; 
use crate::InputSeq;
use crate::error::Result;
use crate::language::Language;
use crate::status::{TextBuffer, Status};
use crate::screen::Screen;  

/*
 * This is the Editor file for the Terminal Editor
 * Let's write some code here
 */


pub struct Edit<'a, I, W>
where 
    I: Iterator<Item = Result<InputSeq>>, 
    W: Write,
{
    editor: &'a mut Editor<I, W>,
}



impl<'a, I, W> Edit<'a, I, W>
where
    I: Iterator<Item = Result<InputSeq>>,
    W: Write,
{
    pub fn editor(&self) -> &'_ Editor<I, W>{
        self.editor
    }
}



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
    status_bar: Status, 
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
            screen, 
            status_bar
        })
    }

    

    pub fn new(input: I, output: W, window_size: Option<(usize, usize)>) -> Result<Editor<I, W> >{
        //return the buffer here
        Self::with_buf(TextBuffer::empty(), input, output, window_size)
    }

    pub fn buf(&self) -> &TextBuffer  {
        &self.bufs[self.buf_idx]
        //self.bufs[self.buf_idx]
    }

    pub fn first_paint(&mut self) -> Result<Edit<'_, I, W>>{
        if self.buf().from_sctrach() {
            self.screen.render_welcome(&self.status_bar)?; 
            self.status_bar.redraw = false; 
            
        }else {
           // self.render_screen(); 
           self.render_screen(); 
        }


        Ok(Edit { editor: self })
    }

    
    fn render_screen(&mut self) -> Result<()>{
        //write things here
        
        self.status_bar.redraw = false; 
        
        Ok(())
    }

    pub fn edit(&mut self) -> Result<()>{
       // self.first_paint(); 
       Ok(())
    }



    pub fn set_lang(&mut self, lang: Language){
        let buf = self.buf_mut(); 
        if buf.lang() == lang {
            return ;
        }

        buf.set_Lang(lang); 
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

        //a reference => a reference

        let screen = Screen::new(window_size, &mut input, output)?;
        let bufs: Vec<_> = paths.iter().map(TextBuffer::open).collect::<Result<_>>()?; 
        let status_bar=  Status::from_buffer(&bufs[0], (1, bufs.len())); 


        Ok(Editor {
            input, 
            qutting: false, 
            buf_idx: 0, 
            bufs,
            screen,
            status_bar
        })

    
    }

    
    fn buf_mut(&mut self) -> &mut TextBuffer {
        &mut self.bufs[self.buf_idx]
    }


    pub fn lang(&self) -> Language {
        self.buf().lang()
    }

    pub fn screen(&self) -> &'_ Screen<W> {
        &self.screen
    }


}
    


#[cfg(test)]
mod tests {
    //writing test here
}

//we want to share state here
//how do we solve this 
//we could use generics, we could use trait and eums for various properties
//