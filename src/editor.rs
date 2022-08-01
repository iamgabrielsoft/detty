


// use std::fmt::Write;

use std::io::Write; 
use std::path::Path; 
use crate::error::Result;
use crate::language::Language;
use crate::prompt::{self, PromptResult, Prompt};
use crate::status::{TextBuffer, Status, CursorDir};
use crate::screen::Screen;  
use crate::input::{InputSeq, KeySeq};
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

impl<'a, I, W>Iterator for Edit<'a, I, W>
where 
    I: Iterator<Item = Result<InputSeq>>, 
    W: Write
{
    type Item = Result<InputSeq>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.editor.step() {
            Ok(EditStep::Continue(seq)) => Some(Ok(seq)),
            Ok(EditStep::Quit) => None, 
            Err(error) => Some(Err(error)) //since we dont know the error yet
        }
    }
}




enum EditStep {
    Continue(InputSeq),
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

        let screen = Screen::new(window_size, &mut input, output)?;
        let status_bar = Status::from_buffer(&buf, (1, 1));
        
        Ok(Editor {
            input, 
            qutting: false, 
            bufs: vec![buf], 
            buf_idx: 0,
            screen, 
            status_bar
        })
    }


    fn refresh_statusbar(&mut self) {
        self.status_bar.set_buf_pos((self.buf_idx + 1, self.bufs.len())); 
        self.status_bar.update_from_but(&self.bufs[self.buf_idx]); 
    }


    fn render_screen(&mut self) -> Result<()> {
        self.refresh_statusbar(); 
        self.screen.render(&self.bufs[self.buf_idx], &self.status_bar)?;
        self.status_bar.redraw  = false; 


        Ok(())
        //continue here
    }


    fn will_reset_scroll(&mut self){
        self.screen.set_dirty_start(0); 
        self.screen.row_off = 0; 
        self.screen.col_off = 0; 
    }

    fn will_reset_screen(&mut self) {
        self.screen.set_dirty_start(self.screen.row_off); 
        self.screen.unset_message(); 
        self.status_bar.redraw = true; 
    }

    fn open_buffer(&mut self) -> Result<()>{
       if let PromptResult::Input(input) = self.prompt::<prompt::NoAction>(
        "Open: {} (Empty name for new text buffer", 
        false,
       )? {

        let buf = if input.is_empty() {
            TextBuffer::empty() //when the buffer is empty 
        }else {
            TextBuffer::open(input)?
        }; 

        self.bufs.push(buf); 
        self.buf_idx = self.bufs.len() - 1; 
       }

       Ok(())
    }


    fn switch_buffer(&mut self, idx: usize){
        let len = self.bufs.len(); 

        if len == 1{
            self.screen.set_info_message("No other biffer"); 
            return; 
        }



        self.buf_idx = idx; 
        let buf = self.buf(); 


        //then reset scroll 
        self.will_reset_scroll(); 
    }


    fn next_buffer(&mut self){
        self.switch_buffer(if self.buf_idx == self.bufs.len() - 1 {
            0

        }else {
            self.buf_idx + 1
        })
    }

    fn previous_buffer(&mut self){
        self.switch_buffer(if self.buf_idx == 0 {
            self.bufs.len() - 1
        
        }else {
            self.buf_idx - 1
        })
    }


    fn handle_not_mapped(&mut self, seq: &InputSeq) {
        self.screen.set_error_message(format!("Key ''{} not mapped", seq))
    }

    // fn process_keypress(&mut self, s: InputSeq) -> Result<EditStep>{
    //     use KeySeq::*; 


    //     let rowoff = self.screen.row_off; 
    //     let rows = self.screen.rows(); 
    //     let prev_cursor 
    // }

    fn prompt<A: prompt::Action>(
        &mut self, 
        prompt: &str, 
        empty_is_cancel: bool
    ) -> Result<PromptResult>{

        Prompt::new(
            &mut self.screen, 
            &mut self.bufs[self.buf_idx], 
            &mut self.status_bar, 
            empty_is_cancel,
        )
        .run::<A, _, _>(prompt, &mut self.input)
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
           self.render_screen(); 
        }


        Ok(Edit { editor: self })
    }



    pub fn edit(&mut self) -> Result<()>{
       // self.first_paint(); 

       self.first_paint()?.try_for_each(|x| x.map(|_| ()))
    }


    fn find(&mut self) -> Result<()> {
        //would implement a search struct later on 
        Ok(())
    }

    fn handle_quit(&mut self, s: InputSeq) -> EditStep {
        let modified = self.bufs.iter().any(|b | b.modified()); 
        if !modified || self.qutting {
            EditStep::Quit
        
        }else {
            self.qutting = true; 
            self.screen.set_error_message(
                "Some of your files are unsaved!"
            );

            EditStep::Continue(s)
        }
    }






    fn step(&mut self) -> Result<EditStep>{
        let seq = if let Some(seq) = self.input.next(){
            seq?
        
        } else {
            return Ok(EditStep::Quit)
        }; 


        if self.screen.maybe_resize(&mut self.input)? {
            self.will_reset_screen(); 
        }


        let step = self.process_keypress(seq)?;

        if step.continues(){
            self.render_screen()?
        }


        Ok(step)

    }

    fn show_help(&mut self) -> Result<()> {
        self.screen.render_help()?; 

        // while let Some(seq) = self.input.next() {
        //     if self.screen.maybe_resize(&mut self.input) {
        //         self.screen.render_help()?; 
        //         self.status_bar.redraw = true; 
        //     }


        //     if seq?.key != KeySeq::NotIdentified {
        //         break;
        //     }
        // }


        self.screen.set_dirty_start(self.screen.row_off); 

        Ok(())

    }



    fn process_keypress(&mut self, s: InputSeq) -> Result<EditStep>{
        use KeySeq::*; 


        let rowoff = self.screen.row_off; 
        let rows = self.screen.rows(); 
        let prev_cursor = self.buf().cursor(); 


        match &s {
            InputSeq {
                key: NotIdentified,
                ..
            } => return Ok(EditStep::Continue(s)), 
            InputSeq { key, ctrl: true, ..} => match key {
                Key(b'v') => self.buf_mut().move_cursor_page(CursorDir::Up, rowoff, rows),
                Key(b'f') => self.buf_mut().move_cursor_by_word(CursorDir::Right),
                Key(b'b') => self.buf_mut().move_cursor_by_word(CursorDir::Left),
                Key(b'n') => self.buf_mut().move_cursor_paragraph(CursorDir::Down),
                Key(b'p') => self.buf_mut().move_cursor_paragraph(CursorDir::Up),
                Key(b'x') => self.previous_buffer(),
                Key(b'<') => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Up),
                Key(b'>') => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Down),
                LeftKey => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Left),
                RightKey => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Right),
                _ => self.handle_not_mapped(&s), 
            }

            InputSeq { key, ctrl: true, ..} => match key {
                Key(b'p') => self.buf_mut().move_cursor_one(CursorDir::Up),
                Key(b'b') => self.buf_mut().move_cursor_one(CursorDir::Left),
                Key(b'n') => self.buf_mut().move_cursor_one(CursorDir::Down),
                Key(b'f') => self.buf_mut().move_cursor_one(CursorDir::Right),
                Key(b'v') => self.buf_mut().move_cursor_page(CursorDir::Down, rowoff, rows),

                Key(b'a') => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Left),
                Key(b'e') => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Right),
                Key(b'd') => self.buf_mut().delete_right_char(),
                Key(b'g') => self.find()?,
                Key(b'h') => self.buf_mut().delete_char(),
                Key(b'k') => self.buf_mut().delete_until_end_of_line(),
                Key(b'j') => self.buf_mut().delete_until_head_of_line(),
                Key(b'w') => self.buf_mut().delete_word(),
                Key(b'l') => {
                    self.screen.set_dirty_start(self.screen.row_off); // Clear
                    self.screen.unset_message();
                    self.status_bar.redraw = true;
                }

                Key(b's') => self.save()?, 
                Key(b'i') => self.buf_mut().insert_tab(),
                Key(b'm') => self.buf_mut().insert_line(), 
                Key(b'o') => self.open_buffer()?, 
                Key(b'?') => self.show_help()?, 
                Key(b'x') => self.next_buffer(),
                Key(b']') => self.buf_mut().move_cursor_page(CursorDir::Down, rowoff, rows), 
                Key(b'u') => {
                    //perform undo actions here
                }
                Key(b'r') => {
                    //perform redo operations here
                }

                LeftKey => self.buf_mut().move_cursor_by_word(CursorDir::Left), 
                RightKey => self.buf_mut().move_cursor_by_word(CursorDir::Right), 
                DownKey => self.buf_mut().move_cursor_by_word(CursorDir::Down), 
                Key(b'q') => return Ok(self.handle_quit(s)), 
                //default
                _ => self.handle_not_mapped(&s),
            },

            InputSeq { key, ..} => match key {
                Key(0x1b) => self.buf_mut().move_cursor_page(CursorDir::Up, rowoff, rows), // Clash with Ctrl-[
                Key(0x08) => self.buf_mut().delete_char(), // Backspace
                Key(0x7f) => self.buf_mut().delete_char(), // Delete key is mapped to \x1b[3~
                Key(b'\r') => self.buf_mut().insert_line(),
                Key(b'q') => return Ok(self.handle_quit(s)), 
                _ => self.handle_not_mapped(&s),
            }, 

            InputSeq { key, ..} => match key {
                Key(0x1b) => self.buf_mut().move_cursor_page(CursorDir::Up, rowoff, rows), // Clash with Ctrl-[
                Key(0x08) => self.buf_mut().delete_char(), // Backspace
                Key(0x7f) => self.buf_mut().delete_char(), // Delete key is mapped to \x1b[3~
                Key(b'\r') => self.buf_mut().insert_line(),
                Key(b) if !b.is_ascii_control() => self.buf_mut().insert_char(*b as char),
                Utf8Key(c) => self.buf_mut().insert_char(*c),
                UpKey => self.buf_mut().move_cursor_one(CursorDir::Up),
                LeftKey => self.buf_mut().move_cursor_one(CursorDir::Left),
                DownKey => self.buf_mut().move_cursor_one(CursorDir::Down),
                RightKey => self.buf_mut().move_cursor_one(CursorDir::Right),
                PageUpKey => self.buf_mut().move_cursor_page(CursorDir::Up, rowoff, rows),
                

                PageDownKey => self.buf_mut().move_cursor_page(CursorDir::Down, rowoff, rows), 
                HomeKey => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Left),
                EndKey => self.buf_mut().move_cursor_to_buffer_edge(CursorDir::Right),

                DeleteKey => self.buf_mut().delete_right_char(), 
                _ => self.handle_not_mapped(&s), //default    
            }
            
        }

        if let Some(line) = self.buf_mut().finish_edit() {
            self.screen.set_dirty_start(line);
        }

        if self.buf().cursor() != prev_cursor {
            self.screen.cursor_moved = true; 
        }

        self.qutting = false; 
        Ok(EditStep::Continue(s))
    }



    pub fn set_lang(&mut self, lang: Language){
        let buf = self.buf_mut(); 
        if buf.lang() == lang {
            return ;
        }

        buf.set_lang(lang); 
    }

    pub fn save(&mut self) -> Result<()> {
        let mut create = false; 

        if !self.buf().has_file() {
            //get our template here
            let template = "Save as: {} (^G or ESC to cancel"; 

            if let PromptResult::Input(input) = self.prompt::<prompt::NoAction>(template, true)? {
                let prev_lang = self.buf().lang(); 
                self.buf_mut().set_file(input); //catch the input here
                
                if prev_lang != self.buf().lang() {
                    self.screen.set_dirty_start(self.screen.row_off); 
                }

                create = true;
            } 
        }


        match self.buf_mut().save() {
            Ok(streams) => self.screen.set_info_message(streams), 
            Err(streams) => {
                self.screen.set_error_message(streams);

                if create {
                    self.buf_mut().set_unamed(); 
                }
            }  
        }

        Ok(())
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
            status_bar,
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