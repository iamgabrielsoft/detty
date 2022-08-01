
use std::io; 
use std::fmt; 
use std::io::Read;
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::unix::io::AsRawFd; 
use std::str; 



use crate::error::{ Error, Result}; 


//deriving error from the debug trait 



pub struct InputSequence {
    stdin: StdinMode
}



pub struct StdinMode {
   // visual: bool, 
    stdin: io::Stdin, 
     origin: termios::Termios
}





impl StdinMode  {
    pub fn new() -> Result<StdinMode> {
        use termios::*; 

        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let mut termios = Termios::from_fd(fd)?;
        let origin = termios;



        // Set terminal raw mode. Disable echo back, canonical mode, signals (SIGINT, SIGTSTP) and Ctrl+V.
        termios.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
        // Disable control flow mode (Ctrl+Q/Ctrl+S) and CR-to-NL translation
        termios.c_iflag &= !(IXON | ICRNL | BRKINT | INPCK | ISTRIP);
        // Disable output processing such as \n to \r\n translation
        termios.c_oflag &= !OPOST;
        // Ensure character size is 8bits
        termios.c_cflag |= CS8;
        // Implement blocking read for efficient reading of input
        termios.c_cc[VMIN] = 1;
        termios.c_cc[VTIME] = 0;
        // Apply terminal configurations
        tcsetattr(fd, TCSAFLUSH, &termios)?;



        Ok( StdinMode {
            stdin, 
           origin,
            //visual: true,
        })
    }


    pub fn input_keys(self) -> InputSequence {
        InputSequence {  stdin: self }
    }
}


#[derive(PartialEq)]
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


        Ok(if self.stdin.read(&mut one_byte)? == 0 {
            None
        } else {
            Some(one_byte[0])
        })
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


impl Drop for StdinMode {
    fn drop(&mut self) {
        //restoration 
        termios::tcsetattr(self.stdin.as_raw_fd(), termios::TCSAFLUSH, &self.origin).unwrap();
    }
}

impl Deref for StdinMode {
    type Target = io::Stdin;

    fn deref(&self) -> &Self::Target {
        &self.stdin
    }
}


impl DerefMut for StdinMode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stdin
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


