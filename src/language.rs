use std::{path::Path, ffi::OsStr};

use crate::language;




pub enum Indent {
    AsIs, 
    Fixed(&'static str),
}

pub enum Language {
    Plain, 
    Rust, 
}


impl Language {
    pub fn name(self) -> &'static str {
        use Language::*; 

        match self {
            Plain => "plain",
            Rust => "rust", 
        }
    }


    fn file_exts(self) -> &'static [&'static str] {
        use Language::*; 

        match self {
            Plain => &[], 
            Rust => &["rs"],
        }
    }

    //how the language are been indented
    pub fn indent(self) -> Indent {
        //let bring Language in scope
        use Language::*; 

        match self {
            Plain => Indent::AsIs, 
            Rust => Indent::Fixed("    "),
        }
    }

    pub fn detect<P: AsRef<Path>>(params: P) -> Language {
        use Language::*; 
        let language  = vec![Rust, Plain]; 
        if let Some(extension) = params.as_ref().extension().and_then(OsStr::to_str) {
            for lang in language {
                if lang.file_exts().contains(&extension) {
                    return lang; 
                }
            }
        }
        Plain
    }


}