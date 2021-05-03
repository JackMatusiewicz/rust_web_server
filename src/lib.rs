use std::sync::{Arc, Mutex};

/// A type alias for a mutable closure that can be shared between threads.
pub type WorkItem = Box<dyn FnOnce() -> () + Send>;

pub enum WorkerMessage {
    NewJob(WorkItem),
    Stop,
}

/// A type alias for a thread's join handle and the associated channel endpoint (the send) that will allow you to send
/// work items to the worker thread.
pub type Worker = std::thread::JoinHandle<()>;

/// A threadpool that operates on a round-robin basis.
pub struct ThreadPool {
    threads: Option<Vec<Worker>>,
    sender: std::sync::mpsc::Sender<WorkerMessage>,
}

impl ThreadPool {
    /// The constructor of a ThreadPool.
    pub fn make(number_of_threads: usize) -> ThreadPool {
        let mut threads = Vec::<Worker>::with_capacity(number_of_threads);
        let (sender, receiver) = std::sync::mpsc::channel::<WorkerMessage>();
        let wrapped_receiver: Arc<Mutex<std::sync::mpsc::Receiver<WorkerMessage>>> =
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
                    Ok(WorkerMessage::NewJob(work_item)) => {
                        println!("Work being done by thread #{}", i);
                        work_item();
                        println!("Work item finished on thread #{}", i);
                    }
                    Ok(WorkerMessage::Stop) => {
                        println!("Worker #{} terminating due to shutdown", i);
                        break;
                    }
                    Err(_e) => {
                        println!("Worker {} terminating due to error", i);
                        break;
                    }
                }
            });
            threads.push(thread);
        }

        let threads = Some(threads);
        ThreadPool { threads, sender }
    }

    /// Takes a work items and sends it to the next worker thread.
    pub fn execute<F: FnOnce() -> () + Send + 'static>(&mut self, work_item: F) {
        let boxed_item = Box::new(work_item);
        let _ = self.sender.send(WorkerMessage::NewJob(boxed_item));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Required because we need to take ownership of the vector of JoinHandles,
        // as JoinHandle's join method takes ownership.
        let threads = self.threads.take().unwrap();
        for _ in &threads {
            // We know that once each thread has received this message, it will not requeue, so this will kill all when they are free.
            let _ = self.sender.send(WorkerMessage::Stop);
        }

        for th in threads {
            th.join().unwrap();
        }
    }
}
