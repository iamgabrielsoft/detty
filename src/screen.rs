

use std::io::Write; 
use std::time::SystemTime; 

use crate::{InputSeq, KeySeq}; 
use crate::error::{ Error, Result}; 

pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 
pub const HELP: &str = "\
Ctrl-Q                        : Quit
Ctrl-S                        : Save to file
Ctrl-O                        : Open text buffer
Ctrl-X                        : Next text buffer
Alt-X                         : Previous text buffer
Ctrl-P or UP                  : Move cursor up
Ctrl-N or DOWN                : Move cursor down
Ctrl-F or RIGHT               : Move cursor right
Ctrl-B or LEFT                : Move cursor left
Ctrl-A or Alt-LEFT or HOME    : Move cursor to head of line
Ctrl-E or Alt-RIGHT or END    : Move cursor to end of line
Ctrl-[ or Ctrl-V or PAGE DOWN : Next page
Ctrl-] or Alt-V or PAGE UP    : Previous page
Alt-F or Ctrl-RIGHT           : Move cursor to next word
Alt-B or Ctrl-LEFT            : Move cursor to previous word
Alt-N or Ctrl-DOWN            : Move cursor to next paragraph
Alt-P or Ctrl-UP              : Move cursor to previous paragraph
Alt-<                         : Move cursor to top of file
Alt->                         : Move cursor to bottom of file
Ctrl-H or BACKSPACE           : Delete character
Ctrl-D or DELETE              : Delete next character
Ctrl-W                        : Delete a word
Ctrl-J                        : Delete until head of line
Ctrl-K                        : Delete until end of line
Ctrl-U                        : Undo last change
Ctrl-R                        : Redo last undo change
Ctrl-G                        : Search text
Ctrl-M                        : New line
Ctrl-L                        : Refresh screen
Ctrl-?                        : Show this help";




enum StatusMessageKind {
    Info,
    Error, 
}


struct MessageState {
    text: String, 
    timestamp: SystemTime, 
    kind: StatusMessageKind,
}


pub struct Screen<W: Write> {
    output: W, 
    rx: usize,
    no_cols: usize, 
    no_rows: usize, 
    pub cursor_moved: bool, 
    pub row_off: usize,
    pub col_off: usize, 
}



impl <W: Write> Screen<W> {
    pub fn new<I>(size: Option<(usize, usize)>, input: I, mut output: W) -> Result<Self>
    where 
        I: Iterator<Item = Result<InputSeq>>,
    {
        let (width, height) = if let Some(x) = size {
            x

            //why do i need to use "?" at compilation time 
        
        }else {
            get_window_size(input, &mut output)?
        };


        if check_window(width, height) {
            return Err(Error::TooSmallWindow(width, height));
        }

        

        Ok(Self {
            output, 
            no_cols: width, 
            row_off: 0, 
            col_off: 0,
            rx: 0,
            no_rows: height,
            cursor_moved: true, 
        })
    }


    fn write_flush(&mut self, bytes: &[u8]) -> Result<()> {
        self.output.write(bytes); 
        self.output.flush(); 
        Ok(())
    }
}





fn get_window_size<I, W>(input: I, mut output: W) -> Result<(usize, usize)>
    where
        I: Iterator<Item = Result<InputSeq>>, 
        W:Write
{
    if let Some(x) =  term_size::dimensions() {
        return Ok(x); 
    }


    for tita in input {
        if let KeySeq::Cursor(height, width) = tita?.key {
            return Ok((height, width))
        }
    }

    Err(Error::UnknownWindowSize) //when the size is unknown by your terminal
}




//check if window is small
fn check_window(width: usize, height: usize) -> bool {
    width < 1 || height < 3
}