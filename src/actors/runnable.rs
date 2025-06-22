use std::{sync::Arc, thread::JoinHandle};

pub trait Runnable {
    fn run(self: Arc<Self>) -> JoinHandle<()>;
}
