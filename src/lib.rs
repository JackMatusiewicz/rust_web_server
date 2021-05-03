use std::sync::{Arc, Mutex};

/// A type alias for a mutable closure that can be shared between threads.
pub type WorkItem = Box<dyn FnOnce() -> () + Send>;

/// A type alias for a thread's join handle and the associated channel endpoint (the send) that will allow you to send
/// work items to the worker thread.
pub type Worker = std::thread::JoinHandle<()>;

/// A threadpool that operates on a round-robin basis.
pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: std::sync::mpsc::Sender<WorkItem>,
}

impl ThreadPool {
    /// The constructor of a ThreadPool.
    pub fn make(number_of_threads: usize) -> ThreadPool {
        let mut threads = Vec::<Worker>::with_capacity(number_of_threads);
        let (sender, receiver) = std::sync::mpsc::channel::<WorkItem>();
        let wrapped_receiver: Arc<Mutex<std::sync::mpsc::Receiver<WorkItem>>> =
            Arc::new(Mutex::new(receiver));
        for i in 0..number_of_threads {
            let cloned_receiver = Arc::clone(&wrapped_receiver);
            let thread = std::thread::spawn(move || loop {
                let work_item = {
                    let receiver = (*cloned_receiver).lock();
                    match receiver {
                        Ok(receiver) => (*receiver).recv(),
                        Err(_e) => panic!(),
                    }
                };

                match work_item {
                    Ok(work_item) => {
                        println!("Work being done by thread #{}", i);
                        work_item();
                        println!("Work item finished on thread #{}", i);
                    }
                    Err(_e) => (),
                }
            });
            threads.push(thread);
        }

        ThreadPool { threads, sender }
    }

    /// Takes a work items and sends it to the next worker thread.
    pub fn execute<F: FnOnce() -> () + Send + 'static>(&mut self, work_item: F) {
        let boxed_item = Box::new(work_item);
        let _ = self.sender.send(boxed_item);
    }
}
