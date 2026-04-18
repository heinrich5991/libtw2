pub trait OptionExt {
    type Inner;
    fn is_none_or(self, f: impl FnOnce(Self::Inner) -> bool) -> bool;
}

impl<T> OptionExt for Option<T> {
    type Inner = T;
    fn is_none_or(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            None => true,
            Some(inner) => f(inner),
        }
    }
}
