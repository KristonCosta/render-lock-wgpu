use crate::worker::worker::Worker;
use std::marker::PhantomData;
use std::sync::mpsc;

pub struct Pool<D: Sized, I: Sized, W: Worker<D, I> + Clone + Copy> {
    workers: Vec<W>,
    _p1: PhantomData<I>,
    _p2: PhantomData<D>,
}

impl<D: Sized, I: Sized, W: Worker<D, I> + Clone + Copy> Pool<D, I, W> {
    pub fn new(size: usize, bundle: I) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        for id in 0..size {
            workers.push(Worker::new(id, &bundle));
        }
        Self {
            workers,
            _p1: PhantomData,
            _p2: PhantomData,
        }
    }

    pub fn dispatch(&self) {}
}
