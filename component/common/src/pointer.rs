use std::ops::Deref;

///Bypass orphan rule
#[derive(Debug)]
pub struct Bor<T>(T);

impl<T: Clone> Clone for Bor<T> {
    fn clone(&self) -> Self {
        Bor(T::clone(&self.0))
    }
}

impl<T> Deref for Bor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for Bor<T> {
    fn from(t: T) -> Self {
        Bor(t)
    }
}
