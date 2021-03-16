pub trait Worker<D: Sized, I: Sized> {
    fn new(id: usize, bundle: &I) -> Self;
}
