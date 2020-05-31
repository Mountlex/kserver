#[derive(Copy, Clone, Debug)]
pub struct Request {
    pub s: i32,
    pub t: i32,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.s == self.t {
            write!(f, "{}", self.s)
        } else {
            write!(f, "{}->{}", self.s, self.t)
        }
    }
}

impl From<i32> for Request {
    fn from(pos: i32) -> Self {
        Request { s: pos, t: pos }
    }
}

impl From<(i32, i32)> for Request {
    fn from(relocation: (i32, i32)) -> Self {
        Request {
            s: relocation.0,
            t: relocation.1,
        }
    }
}
