//! # CPU Infrastructure (The Scheduler)
//!
//! Wraps `rayon` to provide task parallelism and parallel iterators.

use rayon::ThreadPoolBuilder;

/// Global CPU Context
pub struct CpuContext;

impl CpuContext {
    /// Configure the global thread pool
    pub fn configure(threads: usize) -> Result<(), String> {
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|e| e.to_string())
    }

    /// Execute a closure in the thread pool (Task Parallelism)
    pub fn spawn<F, R>(func: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        rayon::join(|| {}, func).1
    }

    /// Execute an async future in the thread pool (Blocking Bridge)
    /// 
    /// Useful for calling Async I/O from a parallel computation.
    #[cfg(feature = "tokio")]
    pub fn spawn_async<F, Fut, R>(f: F) -> R
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = R> + Send,
        R: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        rayon::spawn(move || {
            // Create a temporary runtime for this task
            // Note: This is heavy. Use sparingly.
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(f());
            let _ = tx.send(result);
        });
        rx.recv().unwrap()
    }

    /// Parallel map over a slice (Data Parallelism)
    pub fn par_map<T, U, F>(data: &[T], map_op: F) -> Vec<U>
    where
        T: Sync,
        U: Send,
        F: Fn(&T) -> U + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter().map(map_op).collect()
    }
}

/// Simple Task Graph for Heterogeneous Parallelism
pub struct TaskGraph {
    tasks: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task<F>(&mut self, task: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.tasks.push(Box::new(task));
    }

    pub fn execute(self) {
        use rayon::prelude::*;
        self.tasks.into_par_iter().for_each(|task| task());
    }
}

#[cfg(feature = "profiling")]
pub fn profile<F, R>(name: &'static str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("[Profile] {}: {:?}", name, duration);
    result
}

// Legacy bench module (kept for compatibility)
pub mod bench {
    use std::time::Instant;

    pub fn time_it<F, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        println!("[BENCH] {} took {:?}", name, duration);
        result
    }
}
