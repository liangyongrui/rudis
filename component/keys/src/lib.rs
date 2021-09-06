#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]

/// Save 1 byte than `std::sync::Arc<[u8]>`
// pub type Key = servo_arc::Arc<[u8]>;

pub type Key = std::sync::Arc<[u8]>;

#[cfg(test)]
mod test {
    use crate::Key;

    #[test]
    fn test() {
        let len = std::mem::size_of::<Key>();
        let len2 = std::mem::size_of::<std::sync::Arc<[u8]>>();
        dbg!(len, len2);
    }
}
