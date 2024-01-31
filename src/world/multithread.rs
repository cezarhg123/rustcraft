use std::sync::mpsc;
use super::job::Job;

pub struct Multithread {
    current_worker: usize,
    senders: Vec<mpsc::Sender<Job>>,
    worker_threads: Vec<std::thread::JoinHandle<()>>
}

impl Multithread {
    pub fn new(thread_count: usize) -> Multithread {
        let mut senders = Vec::new();
        
        let mut worker_threads = Vec::new();
        for _ in 0..thread_count {
            let (sender, receiver) = mpsc::channel::<Job>();
            senders.push(sender);
            worker_threads.push(std::thread::spawn(move || {
                loop {
                    let job = receiver.recv().unwrap();
                    job.do_job();
                }
            }));
        }

        Multithread {
            current_worker: 0,
            worker_threads,
            senders
        }
    }

    pub fn add_job(&mut self, job: Job) {
        self.senders[self.current_worker].send(job).unwrap();
        self.current_worker = (self.current_worker + 1) % self.worker_threads.len();
    }
}
