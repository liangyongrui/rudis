use common::connection::parse::{frame::Frame, Parse, ParseError};
use dict::data_type::DataType;

pub fn next_data_type(parse: &Parse) -> Result<DataType, ParseError> {
    match parse.next_frame()? {
        Frame::Integer(i) => Ok(DataType::Integer(i)),
        Frame::Bulk(b) => Ok(DataType::Bytes(b.into())),
        Frame::Simple(s) => Ok(DataType::String(s.into())),
        frame => Err(format!("protocol error;  got {:?}", frame).into()),
    }
}

pub fn data_type_to_frame(dt: DataType) -> Frame<'static> {
    match dt {
        DataType::String(s) => Frame::OwnedSimple(s),
        DataType::Bytes(b) => Frame::OwnedBulk(b),
        DataType::Integer(i) => Frame::Integer(i),
        DataType::Float(f) => Frame::OwnedStringSimple(format!("{}", f.0)),
        DataType::Null => Frame::Null,
        _ => Frame::Error(b"type not support"[..].into()),
    }
}
