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
/// assert_eq!(Request::Simple(4.0), simple_req);
/// let relocation_req = Request::from((2,4));
/// assert_eq!(Request::Relocation(2.0, 4.0), relocation_req);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Request {
    Simple(f32),
    Relocation(f32, f32),
}

impl Request {
    pub fn pos(&self) -> &f32 {
        match self {
            Request::Simple(x) => x,
            Request::Relocation(x,_) => x,
        }
    }

    pub fn is_simple(&self) -> bool {
        matches!(*self, Request::Simple(_))
    }

    pub fn distance_to(&self, other: &f32) -> f32 {
        match self {
            Request::Simple(x) => (x - other).abs(),
            Request::Relocation(_, y) => (y - other).abs(),
        }
    }
    pub fn distance_to_req(&self, other: &Request) -> f32 {
        let pos = other.distance_from(&0.0);
        match self {
            Request::Simple(x) => (x - pos).abs(),
            Request::Relocation(_, y) => (y - pos).abs(),
        }
    }
    pub fn distance_from(&self, other: &f32) -> f32 {
        match self {
            Request::Simple(x) => (x - other).abs(),
            Request::Relocation(x, y) => (x - other).abs(),
        }
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Request::Simple(x) => write!(f, "{}", x),
            Request::Relocation(x, y) => write!(f, "{}->{}", x, y),
        }
    }
}

impl From<f32> for Request {
    fn from(pos: f32) -> Self {
        Request::Simple(pos)
    }
}

impl From<(f32, f32)> for Request {
    fn from(relocation: (f32, f32)) -> Self {
        Request::Relocation(relocation.0, relocation.1)
    }
}

impl From<i32> for Request {
    fn from(pos: i32) -> Self {
        Request::Simple(pos as f32)
    }
}

impl From<(i32, i32)> for Request {
    fn from(relocation: (i32, i32)) -> Self {
        Request::Relocation(relocation.0 as f32, relocation.1 as f32)
    }
}
