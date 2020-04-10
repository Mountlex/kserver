
#[derive(Clone, Debug)]
pub struct SimpleRequest {
    pub pos: i32
}

#[derive(Clone, Debug)]
pub struct RelocationRequest { 
    pub s: i32, 
    pub t: i32 
}

pub trait OnTheLine {
    fn start_pos(&self) -> i32;
}

impl OnTheLine for SimpleRequest {
    fn start_pos(&self) -> i32 {
        self.pos
    }
}
impl OnTheLine for RelocationRequest {
    fn start_pos(&self) -> i32 {
        self.s
    }
}

impl From<i32> for SimpleRequest {
    fn from(pos: i32) -> Self {
        SimpleRequest { pos: pos }
    }
}

impl From<(i32,i32)> for RelocationRequest {
    fn from(relocation: (i32,i32)) -> Self {
        RelocationRequest {s: relocation.0, t: relocation.1 }
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
