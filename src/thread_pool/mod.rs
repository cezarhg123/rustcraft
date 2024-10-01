pub mod task;

use std::{sync::{mpsc, Arc, RwLock, RwLockWriteGuard}, thread};
use task::Task;
use crate::world::{block::BlockID, blocks::Blocks, chunk::{BlockArray, Chunk}};

pub struct ThreadPool {
    workers: Vec<WorkerThread>,
    index: usize
}

impl ThreadPool {
    pub fn new(threads: usize) -> ThreadPool {
        let mut workers = Vec::new();

        for _ in 0..threads {
            workers.push(WorkerThread::new());
        }

        ThreadPool {
            workers,
            index: 0
        }
    }

    pub fn send_task(&mut self, task: Task) {
        self.workers[self.index].send_task(task);
        self.index = (self.index + 1) % self.workers.len();
    }
}

struct WorkerThread {
    thread: thread::JoinHandle<()>,
    task_sender: mpsc::Sender<Task>
}

impl WorkerThread {
    fn new() -> WorkerThread {
        let (task_sender, task_receiver) = mpsc::channel::<Task>();

        WorkerThread {
            thread: thread::spawn(move || {
                while let Ok(task) = task_receiver.recv() {
                    if task.kys() {
                        return;
                    } else {
                        task.run();
                    }
                }
            }),
            task_sender
        }
    }

    fn send_task(&self, task: Task) {
        self.task_sender.send(task).unwrap();
    }
}
