use std::sync::{mpsc, Arc, Mutex};

pub trait Worker<D: Sized, I: Sized> {
    fn new(id: usize, bundle: &I, receiver: Arc<Mutex<mpsc::Receiver<D>>>) -> Self;
}
