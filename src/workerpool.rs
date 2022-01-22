use std::sync::{Mutex, Arc, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Stop,
}

pub struct WorkerPool {
    sender: mpsc::Sender<Message>,
    workers: Vec<Option<thread::JoinHandle<()>>>,
}

impl WorkerPool {
    pub fn new(size: usize) -> WorkerPool {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel::<Message>();
        let rx = Arc::new(Mutex::new(rx));
        
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let rx = rx.clone();

            let handle = Some(thread::spawn(move || {
                loop {
                    match rx.lock().unwrap().recv().unwrap() {
                        Message::NewJob(job) => job(),
                        Message::Stop => break,
                    }
                };
            }));

            workers.push(handle);
        };

        WorkerPool {
            sender: tx,
            workers,
        }
    }

    pub fn execute<F>(&self,job: F) 
        where F: FnOnce() + Send + 'static 
    {
        self.sender.send(Message::NewJob(Box::new(job))).unwrap();
    }
}

impl Drop for WorkerPool{
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Stop).unwrap();
        }

        for worker in &mut self.workers {
            let worker = worker.take();
            if let Some(handle) = worker {
                handle.join().unwrap();
            }
        }
    }
}