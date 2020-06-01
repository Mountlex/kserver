/// Represents a request for a server problem on the line.
///
/// A request consists of two parts: `s` and `t`. If `s==t`, the requests is called simple, other wise relocating.
/// For the k-server problem on the line, only simple requests are allowed. The k-taxi problem allows both types of requests.
/// If a server wants to server a request, it has to move to `s` and then gets relocated to `t`.
///
/// ## Examples
///
/// Requests can directly be derived from single integers or tuples:
/// ```
/// # use serversim::request::Request;
/// let simple_req = Request::from(4);
/// assert_eq!(4, simple_req.s);
/// assert_eq!(4, simple_req.t);
/// assert!(simple_req.is_simple());
/// let relocation_req = Request::from((2,4));
/// assert_eq!(2, relocation_req.s);
/// assert_eq!(4, relocation_req.t);
/// assert!(!relocation_req.is_simple());
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Request {
    pub s: i32,
    pub t: i32,
}

impl Request {
    pub fn is_simple(&self) -> bool {
        self.s == self.t
    }
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
