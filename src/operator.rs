#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operator {
    pub fn apply(&self, captor: i32, captured: i32) -> i32 {
        match self {
            Operator::Add => captor + captured,
            Operator::Sub => captor - captured,
            Operator::Mul => captor * captured,
            Operator::Div => {
                if captured == 0 {
                    0
                }
                else {
                    captor / captured
                }
            }
        }
    }
}