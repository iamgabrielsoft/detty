

use std::cmp;
use std::io::Write; 
use std::time::SystemTime; 

use crate::color::{Color, TerminalColor};
use crate::status::Status;
use crate::{InputSeq, KeySeq}; 
use crate::error::{ Error, Result}; 
use crate::message::DrawMessage;

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




#[derive(PartialEq)]
enum StatusMessageKind {
    Info,
    Error, 
}





struct MessageState {
    text: String, 
    timestamp: SystemTime, 
    kind: StatusMessageKind,
}

impl MessageState {
    fn new<S: Into<String>>(message: S, kind: StatusMessageKind) -> MessageState {
        MessageState {
            text: message.into(), //convert the type into a string
            timestamp: SystemTime::now(), 
            kind,
        }
    }
}


pub struct Screen<W: Write> {
    output: W, 
    rx: usize,
    no_cols: usize, 
    no_rows: usize, 
    pub cursor_moved: bool, 
    pub row_off: usize,
    pub col_off: usize, 
    terminal_color:TerminalColor,
    message: Option<MessageState>, 
    dirty_start: Option<usize>,
    draw_message: DrawMessage,
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

        output.write(b"\x1b[?1049h");

        Ok(Self {
            output, 
            no_cols: width, 
            row_off: 0, 
            col_off: 0,
            rx: 0,
            no_rows: height,
            cursor_moved: true,
            terminal_color: TerminalColor::getting_from_env(),
            draw_message: DrawMessage::Open, 
            dirty_start: Some(0),
            message: Some(MessageState::new("Ctrl-? for Help", StatusMessageKind::Info))
        
        })
    }


    fn write_flush(&mut self, bytes: &[u8]) -> Result<()> {
        self.output.write(bytes); 
        self.output.flush(); 
        Ok(())
    }


    pub fn render_welcome(&mut self, status_bar: &Status) -> Result<()> {
        
        let mut buf = Vec::with_capacity(0); 
        buf.write(self.terminal_color.sequence(Color::Reset)); 


        let msg_ = format!("Detty Editor --version {}", VERSION); 
        let padding = (self.no_cols - 10) /2; 

        if padding > 0 {
            buf.write(self.terminal_color.sequence(Color::NonText)); 
            buf.write(b"~"); 
            buf.write(self.terminal_color.sequence(Color::Reset));

            for _ in 0..padding -1 {
                buf.write(b" ");
            }

            buf.write(msg_.as_bytes());
        }
        Ok(())
    }

    
    fn draw_message_bar<B:Write>(&self, mut buf: B, message: MessageState) -> Result<()>{
        let text = &message.text[..cmp::min(message.text.len(), self.no_cols)]; 


        write!(buf, "\x1b[{}]H", self.no_rows + 2);

        if message.kind == StatusMessageKind::Error {
            buf.write(self.terminal_color.sequence(Color::RedBg)); 
        }


        buf.write(text.as_bytes()); 

        if message.kind != StatusMessageKind::Info {
            buf.write(self.terminal_color.sequence(Color::Reset)); 
        }

        buf.write(b"\x1b[K");
        Ok(())
    }


    pub fn maybe_resize<I>(&mut self, input: I) -> Result<bool>
    where 
        I: Iterator<Item = Result<InputSeq>>, 
    {

        //do i want the user to be notified when window is shrinking?

        let (w, h) = get_window_size(input, &mut self.output)?; 
        if check_window(w, h){
            return Err(Error::TooSmallWindow(w, h)); 
        }


        self.no_rows = h.saturating_sub(2); 
        self.no_cols = w; 
        self.dirty_start = Some(0); 
        
        Ok(true)
    }

    fn after_render(&mut self) {
        //clear state 

        self.dirty_start = None;
        self.cursor_moved = false; 
        self.draw_message = DrawMessage::DoNothing;
        //self.draw
    }


    pub fn rows(&self) -> usize {
        if self.message.is_some() {
            self.no_cols
        
        }else {
            self.no_rows + 1
        }
    }

    pub fn cols(&self) -> usize {
        self.no_cols
    }

    pub fn set_dirty_start(&mut self, start: usize) {
        if let Some(x) = self.dirty_start {
            if x < start {
                return 
            }
        }

        self.dirty_start = Some(start); 
    }


    //get they message text
    pub fn message_text(&self) -> &str{ 
        self.message.as_ref().map(|x| x.text.as_str()).unwrap_or("")
    }


    pub fn set_message(&mut self, m: Option<MessageState>){

        let op = match (&self.message, &m) {
            (None, None) => DrawMessage::DoNothing, 
            (Some(_), None) => DrawMessage::Close, 
            (None, Some(_)) => DrawMessage::Open, 
            (Some(_), Some(_)) => DrawMessage::Update, 
            (Some(x), Some(n)) if x.text == n.text => DrawMessage::DoNothing, 
        };

        //why fold here? to to receive the messages
        self.draw_message = self.draw_message.fold(op);
        self.message = m; 
    }


    pub fn set_info_message<S: Into<String>>(&mut self, message: S) {
        self.set_message(Some(MessageState::new(message, StatusMessageKind::Info)))
    }

    pub fn set_error_message<S: Into<String>>(&mut self, message: S){
        self.set_message(Some(MessageState::new(message, StatusMessageKind::Error)))
    }


    pub fn unset_message(&mut self) {
        self.set_message(None); 
    }


    pub fn force_set_cursor(&mut self, row: usize, col: usize) -> Result<()> {
        write!(self.output, "\x1b[{};{}H", row, col); 
        self.output.flush(); 
        Ok(())
    }

    pub fn render_help(&mut self) -> Result<()>{
        let help:Vec<_> = HELP.split('\n')
            .skip_while(|x| !x.contains(':'))
            .map(str::trim_start)
            .collect(); 

        let rows = self.rows(); 

        //let show where the user would see it on his screen
        let vertical_margin = if help.len() < rows {
            (rows - help.len()) /2
        }else {
            0
        }; 



       // let help_max_width = help.iter().map()
        let max_width = help.iter().map(|x| x.len()).max().unwrap(); 
        let left_margin = if max_width < self.no_cols {
            (self.no_cols - max_width) /2 
        }else {
            0
        };

        let mut buf = Vec::with_capacity(self.rows() * self.no_cols); 

        for y in 0..vertical_margin {
            write!(buf, "\x1b[{}H", y + 1); 
            buf.write(b"\x1b[K"); 
        }


        let left_pad = " ".repeat(left_margin); 

        
        //self.write_flush(buf); 
        self.write_flush(&buf)
        
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
