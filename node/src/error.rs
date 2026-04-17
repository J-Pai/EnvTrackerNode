//! Generic error.

#[derive(Debug)]
pub(crate) struct NodeError {
    _msg: String,
}

impl std::error::Error for NodeError {}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl NodeError {
    pub(crate) fn new(msg: &str) -> Box<NodeError> {
        Box::new(Self {
            _msg: msg.to_string(),
        })
    }
}
