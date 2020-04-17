#[derive(Copy, Clone, Debug)]
pub struct SimpleRequest {
    pub pos: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct RelocationRequest {
    pub s: i32,
    pub t: i32,
}

#[derive(Copy, Clone, Debug)]
pub enum Request {
    Simple(SimpleRequest),
    Relocation(RelocationRequest),
}

impl Request {
    pub fn get_request_pos(&self) -> i32 {
        match self {
            Request::Simple(r) => r.pos,
            Request::Relocation(r) => r.s,
        }
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Request::Relocation(req) => write!(f, "{}->{}", req.s, req.t),
            Request::Simple(req) => write!(f, "{}", req.pos),
        }
    }
}

impl From<i32> for Request {
    fn from(pos: i32) -> Self {
        Request::Simple(SimpleRequest { pos: pos })
    }
}

impl From<i32> for SimpleRequest {
    fn from(pos: i32) -> Self {
        SimpleRequest { pos: pos }
    }
}

impl From<(i32, i32)> for Request {
    fn from(relocation: (i32, i32)) -> Self {
        Request::Relocation(RelocationRequest {
            s: relocation.0,
            t: relocation.1,
        })
    }
}
impl From<(i32, i32)> for RelocationRequest {
    fn from(relocation: (i32, i32)) -> Self {
        RelocationRequest {
            s: relocation.0,
            t: relocation.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_request_from_int_works() {
        let req: SimpleRequest = 3.into();
        assert_eq!(3, req.pos)
    }
}
