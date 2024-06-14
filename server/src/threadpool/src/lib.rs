use std::{
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub use threadpool_error::ThreadPoolError;
mod threadpool_error;

type Job = Box<dyn FnOnce() + Send + 'static>;
type WorkerId = usize;

const THREAD_WAIT_TIMEOUT: Duration = Duration::from_micros(1000); // 0.001s

#[derive(Clone)]
pub struct ThreadPool {
    job_sender: Sender<Job>,
    _thread_manager_handler: Arc<ManagerHandle>,
}

struct ThreadInfo {
    handler: Option<JoinHandle<()>>,
    job_sender: Sender<Job>,
    alive_receiver: Receiver<bool>,
}

struct ThreadManager {
    threads: Vec<ThreadInfo>,
    ready_receiver: Receiver<WorkerId>,
    job_receiver: Receiver<Job>,
    ready_sender: Sender<WorkerId>,
}

struct ManagerHandle(Option<JoinHandle<()>>);

impl Drop for ManagerHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            let _res = handle.join();
        }
    }
}

impl ThreadManager {
    fn new(amount: usize, job_receiver: Receiver<Job>) -> Self {
        let (ready_sender, ready_receiver) = channel();

        let ready_sender_clone = ready_sender.clone();
        let threads = Self::initialize_threads(amount, ready_sender_clone);
        ThreadManager {
            threads,
            ready_receiver,
            job_receiver,
            ready_sender,
        }
    }

    fn run(&mut self) {
        while let Ok(job) = self.job_receiver.recv() {
            let i = self.get_free_thread();
            let _res = self.threads[i].job_sender.send(job);
        }
    }

    fn get_free_thread(&mut self) -> WorkerId {
        let mut resultado = self.ready_receiver.recv_timeout(THREAD_WAIT_TIMEOUT);
        while resultado.is_err() {
            self.recover_threads();
            resultado = self.ready_receiver.recv_timeout(THREAD_WAIT_TIMEOUT);
        }
        resultado.unwrap()
    }

    fn recover_threads(&mut self) {
        for (id, thread) in self.threads.iter_mut().enumerate() {
            if let Err(mpsc::TryRecvError::Disconnected) = thread.alive_receiver.try_recv() {
                Self::reset_thread(thread, id, self.ready_sender.clone());
            }
        }
    }

    fn reset_thread(thread: &mut ThreadInfo, id: WorkerId, ready_sender: Sender<WorkerId>) {
        if let Some(handle) = thread.handler.take() {
            let _res = handle.join();
        }

        let (alive_sender, alive_receiver) = channel();
        let (job_sender, job_receiver) = channel();

        thread.handler = Some(thread::spawn(move || {
            worker(job_receiver, alive_sender, ready_sender, id)
        }));

        thread.job_sender = job_sender;
        thread.alive_receiver = alive_receiver;
    }

    fn initialize_threads(amount: usize, ready_sender: Sender<WorkerId>) -> Vec<ThreadInfo> {
        let mut threads = Vec::new();
        for i in 0..amount {
            let (alive_sender, alive_receiver) = channel();
            let (job_sender, job_receiver) = channel();
            let rs = ready_sender.clone();

            let handler = thread::spawn(move || worker(job_receiver, alive_sender, rs, i));

            threads.push(ThreadInfo {
                handler: Some(handler),
                alive_receiver,
                job_sender,
            });
        }
        threads
    }
}

impl ThreadPool {
    pub fn new(amount: usize) -> ThreadPool {
        let (sender, receiver): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let handler = thread::spawn(move || {
            ThreadManager::new(amount, receiver).run();
        });

        ThreadPool {
            job_sender: sender,
            _thread_manager_handler: Arc::new(ManagerHandle(Some(handler))),
        }
    }

    pub fn execute<F>(&self, job: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.job_sender.send(Box::new(job))?;
        Ok(())
    }
}

impl Drop for ThreadManager {
    fn drop(&mut self) {
        while let Some(mut thread) = self.threads.pop() {
            if let Some(handle) = thread.handler.take() {
                drop(thread);

                let _ = handle.join();
            }
        }
    }
}

fn worker(
    job_receiver: Receiver<Job>,
    _alive: Sender<bool>,
    ready_sender: Sender<WorkerId>,
    id: WorkerId,
) {
    if let Err(_err) = ready_sender.send(id) {
        return;
    }

    for job in job_receiver {
        job();
        if let Err(_err) = ready_sender.send(id) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadPool;
    use std::{
        sync::{Arc, Mutex},
        thread,
    };

    #[test]
    fn test_sum_numbers() {
        let x = Arc::new(Mutex::new(0));
        let mut y = 0;

        let threadpool = ThreadPool::new(10);
        for i in 0..1000 {
            y += i;
            let x_copy = x.clone();
            let _res = threadpool.execute(move || {
                *x_copy.lock().unwrap() += i;
            });
        }
        drop(threadpool);

        assert_eq!(*x.lock().unwrap(), y);
    }

    #[test]
    fn test_panic() {
        let x = Arc::new(Mutex::new(0));
        let mut y = 0;

        let threadpool = ThreadPool::new(10);

        for _ in 0..50 {
            let _res = threadpool.execute(move || {
                panic!("Test panic");
            });
        }

        for i in 0..1000 {
            y += i;
            let x_copy = x.clone();
            let _res = threadpool.execute(move || {
                *x_copy.lock().unwrap() += i;
            });
        }
        drop(threadpool);

        assert_eq!(*x.lock().unwrap(), y);
    }

    #[test]
    fn test_cloning_threadpool() {
        let x = Arc::new(Mutex::new(0));
        let mut y = 0;

        let threadpool = ThreadPool::new(10);
        let threadpool_2 = threadpool.clone();
        for i in 0..1000 {
            y += i;
            let x_copy = x.clone();
            let mut curr_threadpool = &threadpool;
            if i % 2 == 0 {
                curr_threadpool = &threadpool_2;
            }
            let _res = curr_threadpool.execute(move || {
                *x_copy.lock().unwrap() += i;
            });
        }
        drop(threadpool);
        drop(threadpool_2);

        assert_eq!(*x.lock().unwrap(), y);
    }

    #[test]
    fn test_cloning_threadpool_multithread() {
        let x = Arc::new(Mutex::new(0));

        let threadpool = ThreadPool::new(10);
        let threadpool_clone = threadpool.clone();
        let x_clone = x.clone();
        let handle = thread::spawn(move || {
            sum(x_clone, threadpool_clone);
        });
        let y = sum(x.clone(), threadpool);
        let _res = handle.join();

        assert_eq!(*x.lock().unwrap(), y * 2);
    }

    fn sum(x: Arc<Mutex<i32>>, threadpool: ThreadPool) -> i32 {
        let mut y = 0;
        for i in 0..1000 {
            y += i;
            let x_copy = x.clone();
            let _res = threadpool.execute(move || {
                *x_copy.lock().unwrap() += i;
            });
        }
        y
    }
}
