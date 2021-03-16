use crate::worker::worker::Worker;
use anyhow::*;
use std::sync::mpsc;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub struct Pool<D: Sized, I: Sized, W: Worker<D, I>> {
    workers: Vec<W>,
    sender: mpsc::Sender<D>,
    _p1: PhantomData<I>,
}

impl<D: Sized, I: Sized, W: Worker<D, I>> Pool<D, I, W> {
    pub fn new(size: usize, bundle: I) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, &bundle, Arc::clone(&receiver)));
        }
        Self {
            workers,
            sender,
            _p1: PhantomData,
        }
    }

    pub fn dispatch(&mut self, data: D) -> Result<()> {
        self.sender.send(data).unwrap();
        Ok(())
    }
}
