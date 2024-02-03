use std::sync::mpsc;
use super::job::Job;

type Busy = bool;

pub struct Multithread {
    current_worker: usize,
    senders: Vec<mpsc::Sender<Job>>,
    done_receivers: Vec<mpsc::Receiver<()>>,
    worker_threads: Vec<(std::thread::JoinHandle<()>, Busy)>
}

impl Multithread {
    pub fn new(thread_count: usize) -> Multithread {
        let mut senders = Vec::new();
        let mut done_receivers = Vec::new();
        
        let mut worker_threads = Vec::new();
        for _ in 0..thread_count {
            let (sender, receiver) = mpsc::channel::<Job>();
            senders.push(sender);

            let (done_sender, done_receiver) = mpsc::channel::<()>();
            done_receivers.push(done_receiver);

            worker_threads.push((std::thread::spawn(move || {
                loop {
                    let job = receiver.recv().unwrap();
                    job.do_job();

                    if let Job::KYS = job {
                        break;
                    }

                    done_sender.send(()).unwrap();
                }
            }), false));
        }

        Multithread {
            current_worker: 0,
            worker_threads,
            senders,
            done_receivers
        }
    }

    pub fn add_job(&mut self, job: Job) {
        self.senders[self.current_worker].send(job).unwrap();
        self.worker_threads[self.current_worker].1 = true;
        self.current_worker = (self.current_worker + 1) % self.worker_threads.len();

        for i in 0..self.worker_threads.len() {
            if let Ok(_) = self.done_receivers[i].try_recv() {
                self.worker_threads[i].1 = false;
            }
        }
    }

    pub fn wait_for_idle(&mut self) {
        for i in 0..self.worker_threads.len() {
            if self.worker_threads[i].1 {
                if let Ok(_) = self.done_receivers[i].recv() {
                    self.worker_threads[i].1 = false;
                }
            }
        }
    }
}

impl Drop for Multithread {
    fn drop(&mut self) {
        for i in 0..self.worker_threads.len() {
            self.senders[i].send(Job::KYS).unwrap();
        }
    }
}
