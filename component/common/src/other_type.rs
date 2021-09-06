use std::ops::Bound;

pub type LexRange = (Bound<Box<[u8]>>, Bound<Box<[u8]>>);

#[derive(Debug)]
pub enum ZrangeItem {
    Rank((i64, i64)),
    Socre((Bound<f64>, Bound<f64>)),
    Lex(LexRange),
}
