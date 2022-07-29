use crate::row::{Row, self};


#[derive(Copy, Clone)]
pub enum UndoRedo {
    Undo, 
    Redo,
}



//edit-diff
#[derive(Debug)]
pub enum  EditDiff {
    InsertChar(usize, usize, char), 
    DeleteChar(usize, usize, char),
    Insert(usize, usize, String), 
    Append(usize, String),
    Truncate(usize, String),
    Remove(usize, usize, String),
    Newline, 
    InsertLine(usize, String),
    DeleteLine(usize, String),
}




impl EditDiff {
    pub fn apply(&self, rows: &mut Vec<Row>, which: UndoRedo) -> (usize, usize){
        use UndoRedo::*; 

        match *self {
            EditDiff::InsertChar(x, y, c) => match which {
                Redo => {
                    rows[y].insert_char(x, c); 
                    (x + 1, y)
                }

                Undo => {
                    rows[y].remove_char(x); 
                    (x,y)
                }
            },

            EditDiff::DeleteChar(x, y, c) => match which {
                Redo => {
                    rows[y].remove_char(x - y); 
                    (x - 1, y)
                }

                Undo => {
                    rows[y].insert_char(x - 1, c); 
                    (x - 1, y)
                }
            },

            EditDiff::Append(y, ref s, ) => match which {
                Redo => {
                    let len = rows[y].len(); 
                    rows[y].append(s); 
                    (len, y)
                }

                Undo => {
                    let count = s.chars().count(); 
                    let len = rows[y].len(); 
                    rows[y].remove(len - count, len); 
                    (rows[y].len(), y)
                }
            },

            EditDiff::Truncate(y, ref s) => match which {
                Redo => {
                    let count = s.chars().count(); 
                    let len = rows[y].len(); 
                    (len - count, y)
                }

                Undo => {
                    rows[y].append(s);
                    let x = rows[y].len();  
                    (x, y)
                }
            },

            EditDiff::Insert(x, y, ref c) => match which {
                Redo => {
                    rows[y].insert_str(x, c); 
                    (x + c.chars().count(), y)
                }

                Undo => {
                    rows[y].remove(x, c.chars().count()); 
                    (x, y)
                }
            },

            EditDiff::Remove(x, y, ref c) => match which {
                Redo => {
                    let next_x = x - c.chars().count(); 
                    rows[y].remove(next_x, x); 

                    (next_x, y) //return  a tuple  back
                }

                Undo => {
                    let count = c.chars().count(); 
                    rows[y].insert_str(x - count, c); 
                    (x, y)
                }
            },


            EditDiff::Newline => match which {
                Redo => {
                    rows.push(Row::empty()); 
                    (0, rows.len() - 1)
                }


                Undo => {
                    //debug here 
                    rows.pop(); 
                    (0, rows.len())
                }
            },


            EditDiff::InsertLine(y, ref c) => match  which {
                Redo => {
                    rows.insert(y, Row::new(c).unwrap()); 
                    (0, y)
                }

                Undo => {
                    rows.remove(y); 
                    (rows[y - 1].len(), y -1)
                }
            },

            EditDiff::DeleteLine(y, ref c) => match which {
                Redo => {
                    if y == rows.len() - 1 {
                        rows.pop();

                    }else {
                        rows.remove(y);
                    }

                    (rows[y - 1].len(), y - 1)
                }

                Undo => {
                    if y == rows.len() {
                        rows.push(Row::new(c).unwrap()); 
                    
                    }else {
                        //rows.remove(y); 
                        rows.insert(y, Row::new(c).unwrap())
                    }

                    (0, 1)
                }
            },

        }
    }    
}