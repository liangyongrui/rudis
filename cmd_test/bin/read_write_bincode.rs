//! 主要测试 连接过程

use std::fs::File;
#[allow(clippy::many_single_char_names)]
fn main() {
    {
        let a: Option<i32> = Some(1234);
        let f = File::create("./target/bincode.test").unwrap();
        bincode::serialize_into(&f, &a).unwrap();
        bincode::serialize_into(&f, &Some("abc".to_owned())).unwrap();
        bincode::serialize_into(&f, &Some(1.23_f64)).unwrap();
        bincode::serialize_into(&f, &Option::<f32>::None).unwrap();
        bincode::serialize_into(&f, &Some(67_i32)).unwrap();
    }
    {
        let f = File::open("./target/bincode.test").unwrap();

        let a: Option<i32> = bincode::deserialize_from(&f).unwrap();
        let b: Option<String> = bincode::deserialize_from(&f).unwrap();
        let c: Option<f64> = bincode::deserialize_from(&f).unwrap();
        let d: Option<f32> = bincode::deserialize_from(&f).unwrap();
        let e: Option<i32> = bincode::deserialize_from(&f).unwrap();

        dbg!(a, b, c, d, e);
    }
}
