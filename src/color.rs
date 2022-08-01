use std::env;
use term::terminfo::TermInfo;

use crate::regent_color; 

//base on preference


#[derive(Clone, Copy)]
pub enum TerminalColor {
    Colors256,
    Colors16,
    TrueColors,
}

//color enum -> description
pub enum Color {
    Green,
    Gray,
    Orange,
    Invert,
    Red,
    Blue,
    Purple,
    Cyan,
    Yellow,
    YellowBg,
    RedBg,
    OrangeBg,
    NonText,
    Reset,
}






impl Color {
    pub fn has_bg_color(self) -> bool {
        use Color::*;
        matches!(self, YellowBg | RedBg | OrangeBg)
    }
}


fn sequence_for_color_16(color: Color, ) -> &'static [u8] {
    use Color::*; 
    match color {
        Reset => b"\x1b[39;0m",
        Red => b"\x1b[91m",
        Green => b"\x1b[32m",
        Gray => b"\x1b[90m",
        Yellow => b"\x1b[93m",
        Orange => b"\x1b[33m", // No orange color in 16 colors. Use darker yellow instead
        Blue => b"\x1b[94m",
        Purple => b"\x1b[95m",
        Cyan => b"\x1b[96m",
        RedBg => b"\x1b[97m\x1b[41m",
        YellowBg => b"\x1b[103m\x1b[30m",
        OrangeBg => b"\x1b[107m\x1b[30m", // White BG color is used instead of orange
        NonText => b"\x1b[37m",
        Invert => b"\x1b[7m",
    }
}

fn sequence_for_color_256(color: Color) -> &'static [u8] {
    use Color::*; 

    match color {
        Reset => b"\x1b[39;0m",
        Red => b"\x1b[91m",
        Green => b"\x1b[32m",
        Gray => b"\x1b[90m",
        Yellow => b"\x1b[93m",
        Orange => b"\x1b[33m", // No orange color in 16 colors. Use darker yellow instead
        Blue => b"\x1b[94m",
        Purple => b"\x1b[95m",
        Cyan => b"\x1b[96m",
        RedBg => b"\x1b[97m\x1b[41m",
        YellowBg => b"\x1b[103m\x1b[30m",
        OrangeBg => b"\x1b[107m\x1b[30m", // White BG color is used instead of orange
        NonText => b"\x1b[37m",
        Invert => b"\x1b[7m", 
    }
}


fn true_colors_sequence(color: Color) -> &'static[u8] {
    use Color::*; 

    match color {
        Invert => b"\x1b[7m",
        Reset => concat!(
            "\x1b[39;0m",
            regent_color!(fg, 0xfb, 0xf1, 0xc7), 
            regent_color!(bg, 0x28, 0x28, 0x28),
        ).as_bytes(),
        Red => regent_color!(fg, 0xfb, 0x49, 0x34).as_bytes(),
        Green => regent_color!(fg, 0xb8, 0xbb, 0x26).as_bytes(),
        Gray => regent_color!(fg, 0xa8, 0x99, 0x84).as_bytes(),
        Yellow => regent_color!(fg, 0xfa, 0xbd, 0x2f).as_bytes(),
        Orange => regent_color!(fg, 0xfe, 0x80, 0x19).as_bytes(),
        Blue => regent_color!(fg, 0x83, 0xa5, 0x98).as_bytes(),
        Purple => regent_color!(fg, 0xd3, 0x86, 0x9b).as_bytes(),
        Cyan => regent_color!(fg, 0x8e, 0xc0, 0x7c).as_bytes(),
        RedBg => concat!(
            regent_color!(fg, 0xfb, 0xf1, 0xc7),
            regent_color!(bg, 0xcc, 0x24, 0x1d),
        ).as_bytes(), 
        YellowBg => concat!(
            regent_color!(fg, 0x28, 0x28, 0x28),
            regent_color!(bg, 0xd7, 0x99, 0x21),
        )
        .as_bytes(),
        OrangeBg => concat!(
            regent_color!(fg, 0x28, 0x28, 0x28),
            regent_color!(bg, 0xd6, 0x5d, 0x0e),
        )
        .as_bytes(),
        NonText => regent_color!(fg, 0x66, 0x5c, 0x54).as_bytes(),
    }
}




impl TerminalColor {
    //the seuence when change the terminal color background
    pub fn sequence(self, color: Color) -> &'static[u8] {
        match self {
            TerminalColor::TrueColors => true_colors_sequence(color),
            TerminalColor::Colors16 => sequence_for_color_16(color),
            TerminalColor::Colors256 => sequence_for_color_256(color),
        }
    }

    pub fn getting_from_env() -> TerminalColor {
        env::var("COLORTERM")
            .ok()
            .and_then(|v| {
                if v == "truecolor" {
                    Some(TerminalColor::TrueColors)
                } else {
                    None

                    //None
                }
            })
            .or_else(|| {
                TermInfo::from_env().ok().and_then(|x|{
                //    println!("what is this {:?}", x); 
                    x.numbers.get("colors").map(|colors| {
                        if *colors == 256 {
                            TerminalColor::Colors256
                        }else {
                            TerminalColor::Colors16
                        }
                    })
                })
            })
            .unwrap_or(TerminalColor::Colors16)
    }
}

// pub trait ColorTrait {
//     fn feature() -> Color {
//         Color::Blue
//     }
// }
