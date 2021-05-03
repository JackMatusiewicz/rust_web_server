/// A type alias for a mutable closure that can be shared between threads.
pub type WorkItem = Box<dyn FnOnce() -> () + Send + Sync>;

/// A type alias for a thread's join handle and the associated channel endpoint (the send) that will allow you to send
/// work items to the worker thread.
pub type ThreadAndChannel = (
    std::thread::JoinHandle<()>,
    std::sync::mpsc::Sender<WorkItem>,
);

/// A threadpool that operates on a round-robin basis.
pub struct ThreadPool {
    threads: Vec<ThreadAndChannel>,
    next_thread: usize,
}

impl ThreadPool {
    /// The constructor of a ThreadPool.
    pub fn make(number_of_threads: u32) -> ThreadPool {
        let mut v = Vec::<ThreadAndChannel>::new();
        for i in 0..number_of_threads {
            let (sender, receiver) = std::sync::mpsc::channel::<WorkItem>();
            let thread = std::thread::spawn(move || {
                for work_item in receiver {
                    println!("Work being done by thread #{}", i);
                    work_item()
                }
            });
            v.push((thread, sender));
        }

        ThreadPool {
            threads: v,
            next_thread: 0,
        }
    }

    /// Takes a work items and sends it to the next worker thread.
    pub fn execute<F: FnOnce() -> () + Send + Sync + 'static>(&mut self, work_item: F) {
        match self.threads.get(self.next_thread) {
            Some(v) => {
                let sender = &v.1;
                let boxed_item = Box::new(work_item);
                let _ = sender.send(boxed_item);
                self.next_thread = self.next_thread + 1;
                if self.next_thread >= self.threads.len() {
                    self.next_thread = 0;
                }
            }
            None => panic!("Not reachable"),
        }
    }
}
