use connection::parse::{frame::Frame, Parse, ParseError};
use dict::data_type::DataType;

// pub fn frame_to_data_type(frame: Frame) -> common::Result<DataType> {
//     match frame {
//         Frame::Integer(i) => Ok(DataType::Integer(i)),
//         Frame::Bulk(b) => Ok(DataType::Bytes(b)),
//         Frame::Simple(s) => Ok(DataType::String(s)),
//         frame => Err(format!("protocol error;  got {:?}", frame).into()),
//     }
// }

pub fn next_data_type(parse: &mut Parse) -> Result<DataType, ParseError> {
    match parse.next_frame()? {
        Frame::Integer(i) => Ok(DataType::Integer(i)),
        Frame::Bulk(b) => Ok(DataType::Bytes(b)),
        Frame::Simple(s) => Ok(DataType::String(s)),
        frame => Err(format!("protocol error;  got {:?}", frame).into()),
    }
}

pub fn data_type_to_frame(dt: DataType) -> Frame {
    match dt {
        DataType::String(s) => Frame::Simple(s),
        DataType::Bytes(b) => Frame::Bulk(b),
        DataType::Integer(i) => Frame::Integer(i),
        DataType::Float(f) => Frame::Simple(format!("{}", f.0).as_bytes().into()),
        DataType::Null => Frame::Null,
        _ => Frame::Error(b"type not support"[..].into()),
    }
}

pub fn ref_data_type_to_frame(dt: &DataType) -> Frame {
    match dt {
        DataType::String(s) => Frame::Simple(s.clone()),
        DataType::Bytes(b) => Frame::Bulk(b.clone()),
        DataType::Integer(i) => Frame::Integer(*i),
        DataType::Float(f) => Frame::Simple(format!("{}", f.0).as_bytes().into()),
        DataType::Null => Frame::Null,
        _ => Frame::Error(b"type not support"[..].into()),
    }
}
