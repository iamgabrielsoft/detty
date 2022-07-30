


use std::process::exit;
use std::env; 
use std::io;



use crate::screen::{ HELP, VERSION}; 
use crate::error::{ Result, Error};
use crate::editor::Editor;
use crate::input::StdinMode;
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
mod input;



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