

pub enum DrawMessage {
    Open, 
    Close, 
    Update, 
    DoNothing,
}


impl DrawMessage {
    pub fn fold(self, rhs: Self) -> Self {
        use DrawMessage::*;  //i like this style

        match (self, rhs) {
            (Open, Open) => unreachable!(), 
            (Open, Close) => DoNothing, 
            (Open, Update) => Open, 
            (Close, Open) => Update,
            (Close, Close) => unreachable!(),
            (Close, Update) => unreachable!(), 
            (Update, Open) => unreachable!(), 
            (Update, Close) => Close, 
            (Update, Update) => Update, 
            (DoNothing, rhs) => rhs, 
            (lhs, DoNothing) => lhs,
        }
    }
}

