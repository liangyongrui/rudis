//! 模拟客户端, 转Vec<u8>
//!
//! todo 做多种格式兼容, 目前都是转为blob string

use std::usize;

use crate::db::data_type::SimpleType;

pub trait ToVecU8 {
    fn into_vec_u8(self) -> Vec<u8>;
}

impl ToVecU8 for String {
    fn into_vec_u8(self) -> Vec<u8> {
        Vec::from(format!("${}\r\n{}\r\n", self.len(), self).as_bytes())
    }
}

impl ToVecU8 for i64 {
    fn into_vec_u8(self) -> Vec<u8> {
        self.to_string().into_vec_u8()
    }
}

impl ToVecU8 for SimpleType {
    fn into_vec_u8(self) -> Vec<u8> {
        match self {
            SimpleType::Blob(b) => Vec::from(&b[..]),
            SimpleType::SimpleString(s) => s.into_vec_u8(),
            SimpleType::Integer(i) => i.into_vec_u8(),
            SimpleType::Null => vec![b'$', b'-', b'1'],
        }
    }
}

impl ToVecU8 for Vec<SimpleType> {
    fn into_vec_u8(self) -> Vec<u8> {
        let mut res = vec![b'*'];
        res.append(&mut (self.len() as u64).into_vec_u8());
        for s in self {
            res.append(&mut s.into_vec_u8());
        }
        res
    }
}

impl ToVecU8 for u64 {
    fn into_vec_u8(self) -> Vec<u8> {
        self.to_string().into_vec_u8()
    }
}
impl ToVecU8 for Option<i64> {
    fn into_vec_u8(self) -> Vec<u8> {
        match self {
            Some(i) => i.into_vec_u8(),
            None => vec![],
        }
    }
}

impl ToVecU8 for (i64, i64) {
    fn into_vec_u8(self) -> Vec<u8> {
        let mut i0 = self.0.into_vec_u8();
        i0.append(&mut self.1.into_vec_u8());
        i0
    }
}

#[inline]
pub fn build_cmd(cmd: String, args_len: usize) -> Vec<u8> {
    let mut res = vec![b'*'];
    res.extend_from_slice((args_len + 1).to_string().as_bytes());
    res.push(b'\r');
    res.push(b'\n');
    res.append(&mut cmd.into_vec_u8());
    res
}

#[inline]
pub fn append_arg<T: ToVecU8>(cmd: &mut Vec<u8>, arg: T) {
    cmd.append(&mut arg.into_vec_u8())
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        let s = "123".to_string();
        let _t: bytes::Bytes = s.into();
        assert_eq!(vec![b'1', b'2', b'3'], Vec::from(&b"123"[..]));
    }
}
