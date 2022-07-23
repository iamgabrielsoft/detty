use std::path::PathBuf;
use crate::language::Language; 
use crate::setter; 



pub struct Status {
    pub modified: bool, 
    pub filename: String, //the name of the file  
    pub language: Language, 
    pub redraw: bool, 
    pub line_pos: (usize, usize), 
    pub buf_pos: (usize, usize),
}



//creating a filepath for our editor to store files 
pub struct FilePath {
    pub path: PathBuf, 
    pub display: String,
}


struct TextBuffer {
    cx: usize, 
    cy: usize, 
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


    pub fn right(self) -> String {
        //like destructuring in rust -> likeedn to javascript 
        let (lang, (y, len)) = (self.language, self.line_pos); 
        format!("{} {}/{}", lang.name(), y, len)
    }

    pub fn update_from_but(&mut self, buf:TextBuffer) {
        self.set_modified(buf.modified); 
        self.set_language(buf.lang); 
        self.set_filename(buf.filename()); 
        self.set_line_pos((buf.cy, buf.cx))
    }
}