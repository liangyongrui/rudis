use std::ops::Bound;
pub trait BoundExt<T> {
    fn map<O, F: FnOnce(T) -> O>(self, f: F) -> Bound<O>;
}

impl<T> BoundExt<T> for Bound<T> {
    fn map<O, F: FnOnce(T) -> O>(self, f: F) -> Bound<O> {
        match self {
            Bound::Included(t) => Bound::Included(f(t)),
            Bound::Excluded(t) => Bound::Excluded(f(t)),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}
