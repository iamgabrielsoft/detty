

#[macro_export]
//setter macro to re-call every function on the status file
macro_rules! setter {
    ($func:ident, $field:ident, $t:ty) => {
        pub fn $func(&mut self, $field: $t) {
            if self.$field != $field {
                self.redraw = true; 
                self.$field = $field; 
            }
        }
    };

    ($func:ident, $field:ident, $t:ty, $conv:expr) => {
        pub fn $func(&mut self, $field: $t) {
            if self.$field != $field {
                self.redraw = true; 
                self.$field = $conv; 
            }
        }
    }
}



#[macro_export]
/* call color base on the macro */
macro_rules! regent_color {
    (fg, $r:expr, $g:expr, $b:expr) => {
        concat!("\x1b[38;2;", $r, ';', $g, ';', $b, "m")
    };
    (bg, $r:expr, $g:expr, $b:expr) => {
        concat!("\x1b[48;2;", $r, ';', $g, ';', $b, "m")
    };
}