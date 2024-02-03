use std::sync::mpsc;
use super::job::Job;

enum Task {
    Job(Job),
    WaitForIdle,
    // Kill Yourself
    KYS
}

pub struct TaskThread {
    task_sender: mpsc::Sender<Task>,
    join_handle: std::thread::JoinHandle<()>
}

struct WorkerThread {
    job_sender: mpsc::Sender<Job>,
    status_receiver: mpsc::Receiver<()>,
    join_handle: std::thread::JoinHandle<()>,
    busy: bool
}

impl TaskThread {
    pub fn new(thread_count: usize) -> TaskThread {
        let (task_sender, task_receiver) = mpsc::channel();

        let join_handle = std::thread::spawn(move || {
            let mut workers = Vec::new();
            let mut current_worker_index = 0;
            for _ in 0..thread_count {
                workers.push(WorkerThread::new());
            }

            while let Ok(task) = task_receiver.recv() {
                match task {
                    Task::Job(job) => {
                        workers[current_worker_index].job_sender.send(job).unwrap();
                        workers[current_worker_index].busy = true;
                        current_worker_index = (current_worker_index + 1) % workers.len();
                    }
                    Task::WaitForIdle => {
                        for worker in &mut workers {
                            // if worker is busy then wait
                            if worker.busy {
                                worker.status_receiver.recv().unwrap();                                
                            }
                        }
                    }
                    Task::KYS => {
                        for worker in &mut workers {
                            worker.job_sender.send(Job::KYS).unwrap();
                            break;
                        }
                    }
                }

                // check if any worker is idle
                for worker in &mut workers {
                    if let Ok(_) = worker.status_receiver.try_recv() {
                        worker.busy = false;
                    }
                }
            }

            // suicide
            for worker in workers {
                worker.join_handle.join().unwrap();
            }
        });

        TaskThread {
            task_sender,
            join_handle
        }
    }

    pub fn add_job(&self, job: Job) {
        self.task_sender.send(Task::Job(job)).unwrap();
    }

    pub fn wait_for_idle(&self) {
        self.task_sender.send(Task::WaitForIdle).unwrap();
    }
}

impl Drop for TaskThread {
    fn drop(&mut self) {
        self.task_sender.send(Task::KYS).unwrap();
    }
}

impl WorkerThread {
    pub fn new() -> WorkerThread {
        let (job_sender, job_receiver) = mpsc::channel::<Job>();
        let (status_sender, status_receiver) = mpsc::channel();

        let join_handle = std::thread::spawn(move || {
            while let Ok(job) = job_receiver.recv() {
                if let Job::KYS = job {
                    break;
                }

                job.do_job();
                status_sender.send(()).unwrap();
            }
        });

        WorkerThread {
            job_sender,
            status_receiver,
            join_handle,
            busy: false
        }
    }
}