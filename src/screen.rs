

use std::cmp;
use std::io::Write; 
use std::time::SystemTime; 

use unicode_width::UnicodeWidthChar;

use crate::color::{Color, TerminalColor};
use crate::row::Row;
use crate::status::Status;
use crate::buffer::TextBuffer;
use crate::input::{ KeySeq, InputSeq};
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

        output.write(b"\x1b[?1049h")?;

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
        self.output.write(bytes)?; 
        self.output.flush()?; 
        Ok(())
    }


    fn draw_status_bar<B: Write>(&self, mut buf: B, status_bar: &Status) -> Result<()>{
        write!(buf, "\x1b[{}H", self.rows() + 1)?;

        buf.write(self.terminal_color.sequence(Color::Invert)); 


        let left = status_bar.left(); 
        let left = &left[..cmp::min(left.len(), self.no_cols)];
        //let hanle multi-byte chars
        buf.write(left.as_bytes())?;


        let rest_len = self.no_cols - left.len(); 
        if rest_len == 0 {
            buf.write(self.terminal_color.sequence(Color::Reset))?; 
            return Ok(()); 
        }


        let right = status_bar.right(); 
        if right.len() > rest_len {
            for _ in 0..rest_len {
                buf.write(b" ")?;
            }

            buf.write(self.terminal_color.sequence(Color::Reset)); 
            return Ok(())
        }

        
        for _ in 0..rest_len - right.len() {
            buf.write(b" ")?; 
        }

        buf.write(right.as_bytes())?;
        buf.write(self.terminal_color.sequence(Color::Reset))?; 
        Ok(())
    }


    fn draw_rows<B: Write>(
        &self, 
        mut buf: B,
        dirty_start: usize, 
        row: &[Row]
    ) -> Result<()> {
        let row_len = row.len(); 

        buf.write(self.terminal_color.sequence(Color::Reset))?; 

        for y in 0..self.rows() {
            let file_row = y + self.row_off; 

            if file_row < dirty_start {
                continue;
            }


            write!(buf, "\x1b[{}H", y + 1)?;


            if file_row >= row_len {
                buf.write(self.terminal_color.sequence(Color::NonText))?;
                buf.write(b"~")?;
            
            }else {
                let row = &row[file_row]; 


                let mut col = 0; 
                let mut prev_color = Color::Reset; 

                for c in row.render_text().chars(){
                    col += c.width_cjk().unwrap_or(1); 
                    if col <= self.col_off{
                        continue;
                    
                    }else if col > self.no_cols + self.col_off {
                        break;
                    }


                    // let color = hl.color();
                    // if color != prev_color {
                    //     if prev_color.has_bg_color() {
                    //         buf.write(self.term_color.sequence(Color::Reset))?;
                    //     }
                    //     buf.write(self.term_color.sequence(color))?;
                    //     prev_color = color;
                    // }

                    write!(buf, "{}", c)?;
                }
            }


            //ensure to the reset color sequence
            buf.write(self.terminal_color.sequence(Color::Reset))?; 


            buf.write(b"\x1b[K")?;
        }


        Ok(())
    }

    fn update_message_bar(&mut self) -> Result<()>{
        if let Some(m) = &self.message {
            if SystemTime::now().duration_since(m.timestamp)?.as_secs() > 5 {
                self.unset_message(); 
            }
        }


        if self.draw_message == DrawMessage::Close {
            self.set_dirty_start(self.no_rows); 
        }

        Ok(())
    }

    pub fn render(
        &mut self,
        buf: &TextBuffer, 
        status_bar: &Status,
    ) -> Result<()> {
        self.do_scroll(buf.rows(), buf.cursor());
        self.update_message_bar()?;
        self.redraw(buf, status_bar)?; 
        self.after_render(); 
        Ok(())
    }


    pub fn redraw(
        &mut self, 
        text_buf: &TextBuffer,
        status_bar: &Status
    ) -> Result<()> {
        let cursor_row = text_buf.cy() - self.row_off + 1; 
        let cursor_col = self.rx - self.col_off + 1; 
        let draw_message = self.draw_message; 

        if self.dirty_start.is_none() 
            && !status_bar.redraw && draw_message == DrawMessage::DoNothing {
                if self.cursor_moved{
                    write!(self.output, "\x1b[{};{}H", cursor_row, cursor_col)?;
                    self.output.flush()?;
                }
        }


        self.write_flush(b"\x1b[?25l")?;

        let mut buf = Vec::with_capacity((self.rows() + 2) * self.no_cols); 
        if let Some(s) = self.dirty_start {
            self.draw_rows(&mut buf, s, text_buf.rows())?; 

        }


        //when the message bar opens/closes, position of status

        if status_bar.redraw || draw_message == DrawMessage::Open || self.draw_message == DrawMessage::Close {
            self.draw_status_bar(&mut buf, status_bar)?;
        }


        if draw_message == DrawMessage::Update || draw_message == DrawMessage::Open {
            if let Some(message) = &self.message {
                self.draw_message_bar(&mut buf, message)?;
            }
        }

        //move cursor even if cursor_moved is false since
        write!(buf, "\x1b[{};{}H", cursor_row, cursor_col)?;

        
        //remove the cursor -h
        buf.write(b"\x1b[?25h")?;

        self.write_flush(&buf)?;

        Ok(())
        
    }

    fn trim_line<S: AsRef<str>>(&self, line: &S) -> String {
        let line = line.as_ref(); 
        if line.len() <= self.col_off {
            return "".to_string(); 
        }

        line.chars().skip(self.col_off).take(self.no_cols).collect()
    }

    pub fn render_welcome(&mut self, status_bar: &Status) -> Result<()> {
        self.write_flush(b"\x1b[?25l")?; // Hide cursor



        let mut buf = Vec::with_capacity(self.rows()); 
        buf.write(self.terminal_color.sequence(Color::Reset))?;


        for y in 0..self.rows() {
            write!(buf, "\x1b[{}H", y + 1)?;




                if y == self.rows() / 3 {
                    let msg_buf = format!("Kiro editor -- version {}", VERSION);
                    let welcome = self.trim_line(&msg_buf);
                    let padding = (self.no_cols - welcome.len()) / 2;
                    if padding > 0 {
                        buf.write(self.terminal_color.sequence(Color::NonText))?;
                        buf.write(b"~")?;
                        buf.write(self.terminal_color.sequence(Color::Reset))?;
                        for _ in 0..padding - 1 {
                            buf.write(b" ")?;
                        }
                    }
                    buf.write(welcome.as_bytes())?;
                } else {
                    buf.write(self.terminal_color.sequence(Color::NonText))?;
                    buf.write(b"~")?;
                } 
        }

        buf.write(self.terminal_color.sequence(Color::Reset))?; 
        self.draw_status_bar(&mut buf, status_bar)?; 
        
        if let Some(message) = &self.message {
            self.draw_message_bar(&mut buf, message)?; 
        }

        write!(buf, "\x1b[H")?; // Set cursor to left-top
        buf.write(b"\x1b[?25h")?; // Show cursor
        self.write_flush(&buf)?; 
        

        self.after_render();
        Ok(())
    }

    
    fn draw_message_bar<B:Write>(&self, mut buf: B, message: &MessageState) -> Result<()>{
        let text = &message.text[..cmp::min(message.text.len(), self.no_cols)]; 


        write!(buf, "\x1b[{}]H", self.no_rows + 2)?;

        if message.kind == StatusMessageKind::Error {
            buf.write(self.terminal_color.sequence(Color::RedBg))?; 
        }


        buf.write(text.as_bytes())?; 

        if message.kind != StatusMessageKind::Info {
            buf.write(self.terminal_color.sequence(Color::Reset))?; 
        }

        buf.write(b"\x1b[K")?;
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

    fn do_scroll(&mut self, rows: &[Row], (cx, cy): (usize, usize)) {
        let prev_rowoff = self.row_off; 
        let prev_coloff = self.col_off; 

        //calculate the x and y coordinate

        if cy < rows.len() {
            // self.rx = rows[cy].rx
            self.rx = rows[cy].rx_from_cx(cx); 
        
        }else {
            self.rx = 0; 
        }

        if cy < self.row_off {
            //scroll up when cursor
            self.row_off = cy; 
        }

        if cy >= self.row_off + self.rows() {
            self.row_off = cy - self.rows() + 1;
        }

        if self.rx < self.col_off {
            self.col_off = self.rx;
        }


        if self.rx >= self.col_off + self.no_rows {
            self.col_off = self.next_coloff(self.rx - self.no_cols + 1, &rows[cy]); 
        }


        if prev_rowoff != self.row_off || prev_coloff != self.col_off {
            self.set_dirty_start(self.row_off); 
        }

    }

    fn next_coloff(&self, stop: usize, row: &Row) -> usize {
        let mut col_off = 0; 

        for x in row.render_text().chars() {
            col_off += x.width_cjk().unwrap_or(1); 
            if col_off >= stop {
                //add next base on the previous screen_size
                break; 
            }
        }

        col_off
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


    fn set_message(&mut self, m: Option<MessageState>){

        let op = match (&self.message, &m) {
            (Some(p), Some(n)) if p.text == n.text => DrawMessage::DoNothing,
            (Some(_), Some(_)) => DrawMessage::Update,
            (None, Some(_)) => DrawMessage::Open,
            (Some(_), None) => DrawMessage::Close,
            (None, None) => DrawMessage::DoNothing,
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
        write!(self.output, "\x1b[{};{}H", row, col)?; 
        self.output.flush()?; 
        Ok(())
    }

    pub fn render_help(&mut self) -> Result<()>{
        let help: Vec<_> = HELP
        .split('\n')
        .skip_while(|s| !s.contains(':'))
        .map(str::trim_start)
        .collect();
    let rows = self.rows();

    let vertical_margin = if help.len() < rows {
        (rows - help.len()) / 2
    } else {
        0
    };
    let help_max_width = help.iter().map(|l| l.len()).max().unwrap();
    let left_margin = if help_max_width < self.no_cols {
        (self.no_cols - help_max_width) / 2
    } else {
        0
    };

    let mut buf = Vec::with_capacity(rows * self.no_cols);

    for y in 0..vertical_margin {
        write!(buf, "\x1b[{}H", y + 1)?;
        buf.write(b"\x1b[K")?;
    }

    let left_pad = " ".repeat(left_margin);
    let help_height = cmp::min(vertical_margin + help.len(), rows);
    for y in vertical_margin..help_height {
        let idx = y - vertical_margin;
        write!(buf, "\x1b[{}H", y + 1)?;
        buf.write(left_pad.as_bytes())?;

        let help = &help[idx][..cmp::min(help[idx].len(), self.no_cols)];
        buf.write(self.terminal_color.sequence(Color::Cyan))?;
        let mut cols = help.split(':');
        if let Some(col) = cols.next() {
            buf.write(col.as_bytes())?;
        }
        buf.write(self.terminal_color.sequence(Color::Reset))?;
        if let Some(col) = cols.next() {
            write!(buf, ":{}", col)?;
        }

        buf.write(b"\x1b[K")?;
    }

    for y in help_height..rows {
        write!(buf, "\x1b[{}H", y + 1)?;
        buf.write(b"\x1b[K")?;
    }

    self.write_flush(&buf)
        
    }

  

    // pub fn render(
    //     &mut self, 
    //     buf: &TextBuffer, 
    //     status_bar: &Status
    // ) -> Result<()> {
        
    //     self.do_scroll(buf.row, _)
    //     // self.after_render(); 
    //     // Ok(())
    // }

}





fn get_window_size<I, W>(input: I, mut output: W) -> Result<(usize, usize)>
    where
        I: Iterator<Item = Result<InputSeq>>, 
        W:Write
{
    if let Some(x) =  term_size::dimensions_stdout() {
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
