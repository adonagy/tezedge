use crate::helpers::*;

#[derive(Debug, Clone)]
pub enum GetCurrentHead {
    Request,
    Response(Option<CurrentHead>),
}

#[derive(Debug, Clone)]
pub enum GetPublicKey {
    Request,
    Response(String),
}
