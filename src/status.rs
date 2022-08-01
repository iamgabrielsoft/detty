


use std::cmp;
use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::{self, Write, BufRead};
//use std::slice;


use crate::edit_diff::{EditDiff, UndoRedo};
use crate::language::{Language, Indent}; 
use crate::row::Row;
use crate::setter; 
use crate::error::Result;



pub struct Status {
    pub modified: bool, 
    pub filename: String, //the name of the file  
    pub language: Language, 
    pub redraw: bool, 
    pub line_pos: (usize, usize), 
    pub buf_pos: (usize, usize),
}



#[derive(Copy, Clone)]
pub enum CursorDir {
    Left, 
    Right, 
    Up, 
    Down,
}

//creating a filepath for our editor to store files 

pub struct FilePath {
    pub path: PathBuf, 
    pub display: String,
}

impl FilePath {
    fn from<X: AsRef<Path>>(path: X) -> Self {
        let path = path.as_ref(); 
        
        FilePath { 
            path: PathBuf::from(path), 
            display: path.to_string_lossy().to_string(),
        }
    }

    fn from_string<S: Into<String>>(s: S) -> Self {
        let display = s.into(); 
        FilePath{
            path: PathBuf::from(&display),
            display,
        }
    }
}




pub struct TextBuffer {
    cx: usize, 
    cy: usize, 
    row:Vec<Row>,
    undo_count: i32, 
    modified: bool, 
    lang: Language, 
    inserted_undo: bool, 
    dirty_start: Option<usize>,
    file: Option<FilePath>
}


impl TextBuffer {
    pub fn filename(&self) -> &str {
        self.file.as_ref().map(|x| x.display.as_str()).unwrap_or("[NO NAME FOR FILE]")
    }

    pub fn empty() -> Self {
        Self{
            cx:0,
            cy:0, 
            file: None, 
            undo_count: 0, 
            modified: false, 
            lang: Language::Plain, 
            dirty_start:Some(0), 
            inserted_undo: false,
            row:vec![Row::empty()],
        }
    }

    pub fn open<X: AsRef<Path>>(path: X) -> Result<Self>{
        let path = path.as_ref(); 
        let file =  Some(FilePath::from(path)); 


        if !path.exists() {
            //when the file does not exist
            let mut buf = Self::empty(); 
        //    buf.file = file; 
            buf.undo_count = 0; 
            buf.modified = false; 
            buf.lang = Language::detect(path); 
        }


        let row = io::BufReader::new(File::open(path)?)
            .lines()
            .map(|x| Row::new(x?))
            .collect::<Result<_>>()?; 


        Ok(Self {
            cx: 0, 
            cy:0, 
            file, 
            undo_count: 0, 
            modified: false,
            lang: Language::detect(path), 
            inserted_undo: false, 
            dirty_start: Some(0), 
            row,
        })
    }
    

    fn set_dirty_start(&mut self, line: usize) {
        if let Some(x) = self.dirty_start {
            if x <= line {
                return; 
            }
           
        }


        self.dirty_start = Some(line); 
    }
    //when starting the editor from strach
    
    fn apply_diff(&mut self, diff: &EditDiff, which: UndoRedo) {
        let (x, y) = diff.apply(&mut self.row, which); 
        self.set_cursor(x, y); 
        self.set_dirty_start(y); 
    }

    fn new_diff(&mut self, diff: EditDiff){
        self.apply_diff(&diff, UndoRedo::Redo); 
        self.modified = true; 
        //add history here
    }

    fn inserted_undo_point(&mut self) {
        if !self.inserted_undo {
            //when is not inserted into 


            self.modified = false; 
            self.inserted_undo = true; 

        }
    }

    //the method is called when handling one key input at a time
     
    pub fn finish_edit(&mut self) -> Option<usize>{
        self.inserted_undo = false; 
        let dirty_start = self.dirty_start; 
        self.dirty_start = None; 
        dirty_start
    }

    //insert character at a time into the buffer
    pub fn insert_char(&mut self, ch: char) {
        if self.cy == self.row.len() {
            self.new_diff(EditDiff::Newline); 
        }


        self.new_diff(EditDiff::InsertChar(self.cx, self.cy, ch)); 
    }

    
    pub fn insert_tab(&mut self){
        self.inserted_undo_point(); //inset the tab unto a point
        match self.lang.indent() {
            Indent::AsIs => self.insert_char('\t'), 
            Indent::Fixed(indent) => {
                self.new_diff(EditDiff::Insert(
                    self.cx, 
                    self.cy, 
                    indent.to_string() //changed later to to.owned
                ))
            }
        }
    }

    
    fn concat_next_line(&mut self){
        let removed = self.row[self.cy + 1].buffer().to_owned(); 
        self.new_diff(EditDiff::DeleteLine(self.cy + 1, removed.clone())); 
        self.new_diff(EditDiff::Append(self.cy, removed)); 

    }


    fn squash_to_previous_line(&mut self){
        self.cy -= 1; //backtrack the cursor to previous line


        self.cx = self.row[self.cy].len(); //move cursor column to end of previous
        self.concat_next_line(); 
    }


    
    pub fn delete_word(&mut self) {
        if self.cx == 0 || self.cy == self.row.len() {
            return ;
        }

        self.inserted_undo_point(); 

        let mut x = self.cx - 1;
        let row = &self.row[self.cy]; 

        while x > 0 && row.char_at(x).is_ascii_whitespace() {
            x -= 1;
        }

        while x > 0 && !row.char_at(x -1).is_ascii_whitespace() {
            x -= 1;
        }

        let removed = self.row[self.cy][x..self.cx].to_owned(); 
        self.new_diff(EditDiff::Remove(self.cx, self.cy, removed)); 
    }


    pub fn delete_char(&mut self){
        if self.cy == self.row.len() || self.cx == 0 && self.cy == 0 {
            return ;
        }

        self.inserted_undo_point();//insert at a point 

        if self.cx > 0 {
            let idx = self.cx -1; 
            let deleted = self.row[self.cy].char_at(idx);
            self.new_diff(EditDiff::DeleteChar(self.cx, self.cy, deleted)); 

        }else {
            self.squash_to_previous_line(); 
        }
    }


    pub fn delete_until_end_of_line(&mut self){
        if self.cy == self.row.len(){
            return ;
        }

        self.inserted_undo_point(); 
        let row = &self.row[self.cy]; 


        
        if self.cy == row.len(){
            if self.cy == self.row.len() -1 {
                return ;
            }

            self.concat_next_line();
        
        }else if  self.cx < row.buffer().len() {
            let truncated = row[self.cx..].to_owned();
            self.new_diff(EditDiff::Truncate(self.cy, truncated)); 
        }
    }
    

    //set the cursor of the user base on x and y coordinate
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cx = x; 
        self.cy = y; 
    }
   
    pub fn delete_until_head_of_line(&mut self){
        if self.cx == 0 && self.cy == 0 || self.cy == self.row.len() {
            return ;
        }

        self.inserted_undo_point();
        if self.cx == 0 {
            self.squash_to_previous_line(); 
        
        }else {

            let removed = self.row[self.cy][..self.cy].to_owned(); 
            self.new_diff(EditDiff::Remove(self.cx, self.cy, removed)); 
        }
    }

    pub fn delete_a_word(&mut self){
        if self.cx == 0 || self.cy == self.row.len() {
            return ;
        }

        self.inserted_undo_point(); 

        let mut x = self.cx - 1; 
        let row = &self.row[self.cy];
        

        while x > 0 && row.char_at(x).is_ascii_whitespace() {
            x -= 1;
        }
        

        let removed = self.row[self.cy][..self.cx].to_owned(); 
        self.new_diff(EditDiff::Remove(self.cx, self.cy, removed))
    }


    pub fn delete_right_char(&mut self){
        if self.cy == self.row.len() || self.cy == self.row.len() - 1 && self.cx == self.row[self.cy].len(){
            return; 
        }


        self.delete_char(); 
    }


    pub fn insert_line(&mut self){
        self.inserted_undo_point(); 
        if self.cy >= self.row.len() {
            self.new_diff(EditDiff::Newline); 

        }else if self.cx >= self.row[self.cy].len() {
            self.new_diff(EditDiff::InsertLine(self.cy + 1, "".to_string())); 

        }else if self.cx <= self.row[self.cy].buffer().len() {
            let truncate = self.row[self.cy][self.cx..].to_owned(); 
            self.new_diff(EditDiff::Truncate(self.cy, truncate.clone())); 
            self.new_diff(EditDiff::InsertLine(self.cy +  1, truncate)); 
        }
    }


    pub fn move_cursor_one(&mut self, dir: CursorDir){
        match dir {
            CursorDir::Up => self.cy = self.cy.saturating_sub(1), 
            CursorDir::Left => {
                if self.cx > 0 {
                    self.cx -= 1; 

                }else if self.cy > 0 {
                    //when moving to left at top of the line
                    self.cy = 1; 
                    self.cx = self.row[self.cy].len()
                }
            }

            //when the cursor move right
            CursorDir::Right => {
                if self.cy < self.row.len() {
                    let len = self.row[self.cy].len(); 
                    if self.cy < len {
                        self.cx += 1; 

                    }else if self.cx >= len {
                        self.cy += 1; 
                        self.cx = 0; 
                    }
                }
            }

            CursorDir::Down => {
                if self.cy < self.row.len() {
                    self.cy += 1;
                }
            }
        }; 

        let len = self.row.get(self.cy).map(Row::len).unwrap_or(0); 
        if self.cx > len {
            self.cx = len; 
        }
    }


    pub fn move_cursor_page(&mut self, dir: CursorDir, rowoff: usize, no_rows: usize){
        self.cy = match dir {
            CursorDir::Up => rowoff, 
            CursorDir::Down => {
                cmp::min(rowoff + no_rows - 1, self.row.len())
            }

            _ => unreachable!(),
        }; 

        for _ in 0..no_rows {
            self.move_cursor_one(dir); //loop through every char
        }
    }

    pub fn move_cursor_to_buffer_edge(&mut self, dir: CursorDir){
        match dir {
            CursorDir::Left => self.cx = 0,
            CursorDir::Right => {
                if self.cx < self.row.len() {
                    self.cx = self.row[self.cy].len(); 
                }
            }

            CursorDir::Up => self.cy = 0, 
            CursorDir::Down => self.cy = self.row.len(),
        }
    }


    pub fn move_cursor_by_word(&mut self, dir: CursorDir) {
       enum CharKind {
        Ident, 
        Punc, 
        Space,
       }


       impl CharKind {
            fn new_at(rows: &[Row], x: usize, y: usize) -> Self {
                rows.get(y)
                .and_then(|r| r.char_at_checked(x))
                .map(|c| {
                    if c.is_ascii_whitespace() {
                        CharKind::Space
                    
                    }else if c == ' ' || c.is_ascii_alphanumeric() {
                        CharKind::Ident

                    }else {
                        CharKind::Punc
                    }
                })
                .unwrap_or(CharKind::Space)
            }

        }


        fn at_word_start(left: &CharKind, right: &CharKind) -> bool {
                matches!(
                    (left, right),
                    (&CharKind::Space, &CharKind::Ident) | 
                    (&CharKind::Space, &CharKind::Punc) | 
                    (&CharKind::Punc, &CharKind::Punc) |
                    (&CharKind::Ident, &CharKind::Punc)
                )
        }


        self.move_cursor_one(dir); 
        let mut prev = CharKind::new_at(&self.row, self.cx, self.cy); 
        self.move_cursor_one(dir); 
        let mut current = CharKind::new_at(&self.row, self.cx, self.cy); 


        loop {
            if self.cy == 0 && self.cx == 0 || self.cy == self.row.len() {
                return ;
            }


            match dir {
                CursorDir::Right if at_word_start(&prev, &current) => return, 
                CursorDir::Left if at_word_start(&current, &prev) => {
                    self.move_cursor_one(CursorDir::Right); //adjust cursor position
                    return ;
                }

                _ => {}
            }

            prev = current; 
            self.move_cursor_one(dir); 
            current = CharKind::new_at(&self.row, self.cx, self.cy); 
        }
    
    
    }


    pub fn move_cursor_paragraph(&mut self, dir: CursorDir){
        loop {
            self.move_cursor_one(dir); 
            if self.cy == 0 || self.cy == self.row.len() || self.row[self.cy - 1].buffer().is_empty() && !self.row[self.cy].buffer().is_empty() {
                break;
            }
        }

        
    }


    pub fn save(&mut self) -> std::result::Result<String, String>{
        self.inserted_undo_point(); 

        let file = if let Some(file) = &self.file {
            file
        
        } else {
            return Ok("".to_string()) //ended
        }; 

        let f = match File::create(&file.path) {
            Ok(d) => d, 
            Err(e) => return Err(format!("Could not save: {}", e)),
        }; 

        let mut f = io::BufWriter::new(f); 
        let mut bytes = 0; 

        for line in self.row.iter() {
            let b = line.buffer(); 
            writeln!(f, "{}", b);
            bytes += b.bytes().len() + 1;
        }


        f.flush().map_err(|e| format!("could not flush to file: {}", e)); 
        

        self.undo_count = 0; 
        self.modified = false; 
        Ok(format!("{} bytes written to {}", bytes, &file.display))
    }


    fn after_undoredo(&mut self, state: Option<(usize, usize, usize, bool)>) -> bool{
        match state {
            Some((x, y, s, _)) => {
                self.set_cursor(x, y);
                self.set_dirty_start(s);
                true
            }


            None => false,
        }
    }


    pub fn rows(&self) -> &[Row] {
        &self.row
    }



    pub fn cursor(&self) -> (usize, usize) {
        (self.cx, self.cy)
    }

    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }


    pub fn set_file<S: Into<String>>(&mut self, file_path: S) {
        let file = FilePath::from_string(file_path); 
     //   self.lang = Language::detect(&file_path); 
        self.file = Some(file); 
    }



    pub fn set_unamed(&mut self){
        self.file = None;
    }



    pub fn from_sctrach(&self) -> bool {
       // self.file.is_none() && self.row.len() == 1 && self.row[0].len() == 0
        self.file.is_none()
    }

    pub fn set_lang(&mut self, lang: Language) {
        self.lang = lang; //assign a lang -> self.lang
    }



    pub fn lang(&self) -> Language {
        self.lang
    }

    pub fn cy(&self) -> usize {
        self.cy
    }

    pub fn modified(&self) -> bool {
        self.undo_count != 0  || self.modified
    }
}




impl Status {

    setter!(set_buf_pos, buf_pos, (usize, usize)); 
    setter!(set_modified, modified, bool); 
    setter!(set_filename, filename, &str, filename.to_string());
    setter!(set_language, language, Language); 
    setter!(set_line_pos, line_pos, (usize, usize)); 


    pub fn from_buffer(buf:&TextBuffer, buf_pos: (usize, usize)) -> Self {
        Self {
            modified: buf.modified, 
            filename: buf.filename().to_string(), //passing a string a string here
            language:buf.lang, 
            line_pos: (buf.cy +1, buf.cx), 
            redraw: false,
            buf_pos,
        }
    }

    pub fn left(&self) -> String {
        format!(
            "{:<20?} - {}/{} {}", 
            self.filename, 
            self.buf_pos.0, //its picking the first tuple
            self.buf_pos.1, 
            if self.modified { "(modified)" } else { " " }
        )
    }


    pub fn right(&self) -> String {
        //like destructuring in rust -> likeedn to javascript 
        let (lang, (y, len)) = (self.language, self.line_pos); 
        format!("{} {}/{}", lang.name(), y, len)
    }

    pub fn update_from_but(&mut self, buf: &TextBuffer) {
        self.set_modified(buf.modified); 
        self.set_language(buf.lang); 
        self.set_filename(buf.filename()); 
        self.set_line_pos((buf.cy, buf.cx))
    }
}