

use core::fmt;
use std::os::unix::io::AsRawFd;
use std::process::exit;
use std::env; 
use std::io;



use crate::screen::{ HELP, VERSION}; 
use crate::error::{ Result, Error};
use crate::editor::Editor;

/**
 * At some point i got a moved Copy trait error, 
 * I was just lost, i had to trace the error down to every input keyword on my code
 * 
 */


use getopts::Options; 

mod screen;
mod color; 
mod error;
mod language;
mod editor;
mod status;
mod macros;
mod row; 
mod edit_diff;
mod prompt;
mod message;

fn print_help(program: &str, opts: Options) {
    let description = format!(
        "{prog}: A tiny UTF-8 terminal text editor

        Detty is a tiny UTF-8 text editor on terminals for Unix-like systems.
        Specify file paths to edit as a command argument or run without argument to
        start to write a new text.
        Help can show up with key mapping Ctrl-?.

        Usage:
            {prog} [options] [FILES...]

        Mappings:
            {maps}",
                prog = program,
                maps = HELP,
    );


    println!("{}", opts.usage(&description)); 
}


//deriving error from the debug trait 

pub struct InputSequence {
    stdin: StdinMode
}



pub struct StdinMode {
    visual: bool, 
    stdin: io::Stdin, 
  // origin: termios::Termios
}





impl StdinMode  {
    pub fn new() -> Result<StdinMode> {
        use termios::*; 

        let stdin = io::stdin(); 
        let fd = stdin.as_raw_fd(); 
        let mut termios = Termios::from_fd(fd);
        //let origin  = termios?;






        Ok( StdinMode {
            stdin, 
          //  origin,
            visual: true,
        })
    }


    pub fn input_keys(self) -> InputSequence {
        InputSequence {  stdin: self }
    }
}

pub enum KeySeq {
    LeftKey, 
    NotIdentified,
    RightKey,
    Key(u8), 
    UpKey, 
    Utf8Key(char),
    DownKey, 
    PageUpKey, 
    PageDownKey, 
    HomeKey, 
    EndKey,
    DeleteKey, 
    Cursor(usize, usize) // a tuple for this
}



impl fmt::Display for KeySeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KeySeq::*; 


        match self {
            LeftKey => write!(f, "LEFT"), 
            RightKey => write!(f, "RIGHT"), 
            UpKey => write!(f, "UP"), 
            DownKey => write!(f, "DOWN"), 
            DeleteKey => write!(f, "DELETE"), 
            PageUpKey => write!(f, "PAGEUP"), 
            PageDownKey => write!(f, "PAGEDOWN"), 
            EndKey => write!(f, "END"), 
            Cursor(top, bottom) => write!(f, "CURSOR"),
            NotIdentified => write!(f, "NOTIDENTIFIED"),
            Utf8Key(x) => write!(f, "{}", x),
            HomeKey => write!(f, "HOME"),
            Key(b' ') => write!(f, "SPACE"),
            Key(b) => write!(f, "{}", *b as char), 
            Key(b) if b.is_ascii_control() => write!(f, "\\x{:x}", b)
            // /write!(f, "\\x{:x}", b),
        }
    }
}

pub struct InputSeq {
    pub alt: bool, 
    pub ctrl: bool, 
    pub key: KeySeq, 
}



impl InputSeq {
    //new key spec
    pub fn new(key: KeySeq) -> Self {
        Self{
            ctrl: false, 
            alt: false,
            key
        }
    }

    //ctl key spec
    pub fn ctrl(key: KeySeq) -> Self {
        Self {
            ctrl: true, 
            alt: false, 
            key
        }
    }

    //alt key spec
    pub fn alt(key: KeySeq) -> Self {
        Self {
            key, 
            ctrl: false, 
            alt: true,
        }
    }

}



impl fmt::Display for InputSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            write!(f, "C-")?; 
        }

        if self.alt {
            write!(f, "M-")?;
        }

        write!(f, "{}", self.key)
    }
}


impl InputSequence {
    fn read_byte(&mut self) -> Result<Option<u8>> {
        // //let tra: Vec<u8> = Vec::new(); 
        let mut one_byte:[u8; 1] = [0]; //i dont know the meaning of this yet

        // Ok(Some(one_byte[0]))
        // // Ok(if self.stdin.read(&mut one_byte)? == 0 {
        // //     None
        // // }else {
        // //     Some(one_byte[0])
        // // })
        let yes: u8 = 1; 
        Ok(Some(yes))
    }


    fn decode_escape_sequence(&mut self)-> Result<InputSeq>{
        use KeySeq::*; 

        
        match self.read_byte()? {
            // Some(_) => todo!(),
            // None => todo!(),
            Some(b'[') => { /* fall through */ }
            Some(streams) => {
                let mut seq = self.decode(streams)?; 
                seq.alt = true; 
                return Ok(seq); 
            }


            Some(b) if b.is_ascii_control() => return Ok(InputSeq::new(Key(0x1b))),
            None => return Ok(InputSeq::new(Key(0x1b)))
        }; 


        let mut buf = vec![]; 
        let cmd = loop {
            if let Some(stream) = self.read_byte()? {
                match stream {
                    b'A' | b'B' | b'C' | b'D' | b'F' | b'H' | b'K' | b'J' | b'R' | b'c' | b'f'
                    | b'g' | b'h' | b'l' | b'm' | b'n' | b'q' | b't' | b'y' | b'~' => break stream,
                    _ => buf.push(stream)
                }
            
            }else {
                return Ok(InputSeq::new(NotIdentified));
            }
        };


        let mut args = buf.split(|b|*b == b';'); 
        match cmd {
            b'H' | b'F' => {
                let key = match cmd {
                    b'H' => HomeKey, 
                    b'F' => EndKey,
                    _ => unreachable!(),
                }; 

                let ctrl = args.next() == Some(b"1") && args.next() == Some(b"5");
                let alt = false; 
                Ok(InputSeq { key, ctrl, alt })
            }

            b'~' => {
                match args.next() {
                    Some(b"5") => Ok(InputSeq::new(PageUpKey)), 
                    Some(b"6") => Ok(InputSeq::new(PageDownKey)), 
                    Some(b"1") | Some(b"7") => Ok(InputSeq::new(HomeKey)), 
                    Some(b"4") | Some(b"8") => Ok(InputSeq::new(EndKey)), 
                    Some(b"3") => Ok(InputSeq::new(DeleteKey)), 
                    _ => Ok(InputSeq::new(NotIdentified))
                }
            }

            _ => unreachable!(),
        }
    }


    fn decode(&mut self, streams: u8) -> Result<InputSeq> {
        use KeySeq::*; 

        //how do we write the actual decode formula here
        //

        match streams {
          //  0xa00..=0xff => self.decode_utf8(streams),
            0x80..=0x9f => Ok(InputSeq::new(KeySeq::NotIdentified)),
            0x20..=0x7f => Ok(InputSeq::new(Key(streams))),
            0xa0..=0xff => self.decode_utf8(streams), 
            
            
            0x00..=0x1f => match streams {
                0x1b => self.decode_escape_sequence(), 
                0x00 | 0x1f => Ok(InputSeq::ctrl(Key(streams | 0b0010_0000))), 
                0x01c | 0x01d => Ok(InputSeq::ctrl(Key(streams | 0b0100_0000))),

                _ => Ok(InputSeq::ctrl(Key(streams | 0b0110_0000)))
            }

            0x20..=0x7f => Ok(InputSeq::new(Key(streams))), 
            0x80..=0x9f => Ok(InputSeq::new(KeySeq::NotIdentified))
        }
    }

    //decode utf-8 
    fn decode_utf8(&mut self, data: u8) -> Result<InputSeq>{
        //to decode the utf8 
        //we need to read the byte comming from the editor 
        //then loop through every byte base string 

        let mut buf = [0;4]; 
        buf[0] = data; //assign the data i32 -> u8
        let mut len = 1; 
        let fill = buf[..len].to_vec(); 


        
        loop {
            
            if let Some(x) = self.read_byte()? {
                buf[len] = data; 
                len += 1; 
            
            }else {
                return Err(Error::NotUtf8iInput(fill))
            }    


            if let Ok(x) = std::str::from_utf8(&buf){
                return Ok(InputSeq::new(KeySeq::Utf8Key(x.chars().next().unwrap())))
            }

            if len == 4 {
                return Err(Error::NotUtf8iInput(buf.to_vec()))
            }
        }
    }

    
    fn read_seq(&mut self) -> Result<InputSeq> {
        // if let Some(x) = self.read_byte()? {
        //     None
        // } else {
        //     Ok()
        // }

        if let Some(data) = self.read_byte()? {
            self.decode(data) 

        }else {
            Ok(InputSeq::new(KeySeq::NotIdentified))
        }
    }
}



impl Iterator for InputSequence {
    type Item = Result<InputSeq>;

    fn next(&mut self) -> Option<Self::Item> {
        //but we could read the next-byte from the missing member from the iterator crate
        //self.read_byte()
        Some(self.read_seq())
    }
}




fn edit(files: Vec<String>) -> Result<()>{
   // Editor::open(input, output, window_size, paths)

   let input = StdinMode::new()?.input_keys();
   let output = io::stdout();  
    Editor::open(input, output, None, &files)?.edit(); 

   Ok(())
}




fn main() {
    let mut argv = env::args(); 
    let program = argv.next().unwrap(); 


    let mut opts = Options::new(); 
    opts.optflag("v", "version", "Print"); 
    opts.optflag("h", "help", "Print this help");
   // println!("{}", argv)


   let matches = match opts.parse(argv) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Error: {}. Please see --help", err); 
            exit(1); 
        }
    }; 

    //print out the editor version
    if matches.opt_present("v") {
        println!("{}", VERSION); 

        return; 
    }


    //print out the help screen
    if matches.opt_present("h") {
        print_help(&program, opts); 
        return; 
    }



    //more coming up soon


    if let Err(err) = edit(matches.free) {
       eprintln!("Something happened {}", err); 
        exit(1); 
    }

}