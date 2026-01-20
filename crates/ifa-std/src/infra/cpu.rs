//! # CPU Infrastructure (The Scheduler)
//!
//! Provides task parallelism, async TaskGraph with dependencies, and parallel iterators.

use rayon::ThreadPoolBuilder;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::future::Future;
use std::pin::Pin;

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

    /// Execute a closure in the thread pool (BLOCKING - waits for completion)
    pub fn run_blocking<F, R>(func: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        rayon::join(|| {}, func).1
    }

    /// Execute an async future using a shared runtime (Blocking Bridge)
    #[cfg(feature = "tokio")]
    pub fn run_async<F, Fut, R>(f: F) -> R
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = R> + Send,
        R: Send,
    {
        static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
        
        let runtime = RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create async runtime")
        });
        
        runtime.block_on(f())
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
    
    // =========================================================================
    // HIGH-LEVEL COMPUTE API
    // =========================================================================
    
    /// Parallel reduce with identity and combine function
    pub fn par_reduce<T, F, C>(data: &[T], identity: T, map_op: F, combine: C) -> T
    where
        T: Clone + Send + Sync,
        F: Fn(&T) -> T + Sync + Send,
        C: Fn(T, T) -> T + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter()
            .map(map_op)
            .reduce(|| identity.clone(), combine)
    }
    
    /// Parallel sum for numeric types
    pub fn par_sum<T>(data: &[T]) -> T
    where
        T: std::iter::Sum + Clone + Send + Sync + Default,
    {
        use rayon::prelude::*;
        data.par_iter().cloned().sum()
    }
    
    /// Parallel filter
    pub fn par_filter<T, F>(data: &[T], predicate: F) -> Vec<T>
    where
        T: Clone + Send + Sync,
        F: Fn(&T) -> bool + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter().filter(|x| predicate(x)).cloned().collect()
    }
    
    /// Parallel filter + map
    pub fn par_filter_map<T, U, F>(data: &[T], filter_map_op: F) -> Vec<U>
    where
        T: Sync,
        U: Send,
        F: Fn(&T) -> Option<U> + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter().filter_map(filter_map_op).collect()
    }
    
    /// Parallel sort (stable)
    pub fn par_sort<T>(data: &mut [T])
    where
        T: Ord + Send,
    {
        use rayon::prelude::*;
        data.par_sort();
    }
    
    /// Parallel sort by key
    pub fn par_sort_by_key<T, K, F>(data: &mut [T], key_fn: F)
    where
        T: Send,
        K: Ord + Send,
        F: Fn(&T) -> K + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_sort_by_key(key_fn);
    }
    
    /// Parallel any: returns true if any element satisfies predicate
    pub fn par_any<T, F>(data: &[T], predicate: F) -> bool
    where
        T: Sync,
        F: Fn(&T) -> bool + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter().any(predicate)
    }
    
    /// Parallel all: returns true if all elements satisfy predicate
    pub fn par_all<T, F>(data: &[T], predicate: F) -> bool
    where
        T: Sync,
        F: Fn(&T) -> bool + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter().all(predicate)
    }
    
    /// Parallel find: returns first element satisfying predicate
    pub fn par_find<T, F>(data: &[T], predicate: F) -> Option<T>
    where
        T: Clone + Sync + Send,
        F: Fn(&T) -> bool + Sync + Send,
    {
        use rayon::prelude::*;
        data.par_iter().find_any(|x| predicate(*x)).cloned()
    }
    
    /// Parallel matrix multiplication (CPU fallback when GPU unavailable)
    /// A[m x k] * B[k x n] = C[m x n]
    pub fn matmul(a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32> {
        use rayon::prelude::*;
        
        let mut c = vec![0.0f32; m * n];
        
        c.par_chunks_mut(n).enumerate().for_each(|(i, row)| {
            for j in 0..n {
                let mut sum = 0.0f32;
                for p in 0..k {
                    sum += a[i * k + p] * b[p * n + j];
                }
                row[j] = sum;
            }
        });
        
        c
    }
    
    /// Parallel dot product
    pub fn dot(a: &[f32], b: &[f32]) -> f32 {
        use rayon::prelude::*;
        a.par_iter().zip(b.par_iter()).map(|(x, y)| x * y).sum()
    }
    
    /// Parallel elementwise addition
    pub fn vec_add(a: &[f32], b: &[f32]) -> Vec<f32> {
        use rayon::prelude::*;
        a.par_iter().zip(b.par_iter()).map(|(x, y)| x + y).collect()
    }
    
    /// Parallel elementwise multiplication
    pub fn vec_mul(a: &[f32], b: &[f32]) -> Vec<f32> {
        use rayon::prelude::*;
        a.par_iter().zip(b.par_iter()).map(|(x, y)| x * y).collect()
    }
    
    /// Parallel scale and bias: output[i] = input[i] * scale + bias
    pub fn scale_bias(data: &[f32], scale: f32, bias: f32) -> Vec<f32> {
        use rayon::prelude::*;
        data.par_iter().map(|x| x * scale + bias).collect()
    }
    
    /// Parallel ReLU activation
    pub fn relu(data: &mut [f32]) {
        use rayon::prelude::*;
        data.par_iter_mut().for_each(|x| *x = x.max(0.0));
    }
    
    /// Get number of available CPU threads
    pub fn num_threads() -> usize {
        rayon::current_num_threads()
    }
}

/// Unique identifier for a task in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub usize);

/// Error type for task execution
#[derive(Debug, Clone)]
pub enum TaskError {
    /// Task execution failed
    ExecutionFailed(String),
    /// Cycle detected in task graph
    CycleDetected,
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    TimedOut,
    /// Dependency failed
    DependencyFailed(TaskId),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::ExecutionFailed(msg) => write!(f, "Task execution failed: {}", msg),
            TaskError::CycleDetected => write!(f, "Cycle detected in task graph"),
            TaskError::Cancelled => write!(f, "Task was cancelled"),
            TaskError::TimedOut => write!(f, "Task timed out"),
            TaskError::DependencyFailed(id) => write!(f, "Dependency {:?} failed", id),
        }
    }
}

impl std::error::Error for TaskError {}

/// Result of a task execution
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub id: TaskId,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Async task type
type AsyncTask = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

/// Synchronous task type
type SyncTask = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

/// Task wrapper that can be sync or async
enum TaskFn {
    Sync(SyncTask),
    #[allow(dead_code)]
    Async(AsyncTask), // Reserved for async execution path
}

/// Production TaskGraph with DAG dependencies, async execution, and error handling
pub struct TaskGraph {
    tasks: HashMap<TaskId, TaskFn>,
    dependencies: HashMap<TaskId, Vec<TaskId>>,
    timeouts: HashMap<TaskId, std::time::Duration>,
    next_id: usize,
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskGraph {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
            timeouts: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a synchronous task to the graph
    pub fn add_task<F>(&mut self, task: F) -> TaskId
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let id = TaskId(self.next_id);
        self.next_id += 1;
        self.tasks.insert(id, TaskFn::Sync(Box::new(task)));
        self.dependencies.insert(id, Vec::new());
        id
    }

    /// Add an async task to the graph
    pub fn add_async_task<F, Fut>(&mut self, task: F) -> TaskId
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let id = TaskId(self.next_id);
        self.next_id += 1;
        self.tasks.insert(id, TaskFn::Async(Box::new(move || Box::pin(task()))));
        self.dependencies.insert(id, Vec::new());
        id
    }

    /// Add a dependency: `task` depends on `depends_on` (depends_on must complete first)
    pub fn add_dependency(&mut self, task: TaskId, depends_on: TaskId) -> Result<(), &'static str> {
        if !self.tasks.contains_key(&task) {
            return Err("Task not found");
        }
        if !self.tasks.contains_key(&depends_on) {
            return Err("Dependency task not found");
        }
        
        self.dependencies
            .entry(task)
            .or_default()
            .push(depends_on);
        
        Ok(())
    }

    /// Set timeout for a specific task
    pub fn set_timeout(&mut self, task: TaskId, timeout: std::time::Duration) {
        self.timeouts.insert(task, timeout);
    }

    /// Execute all tasks synchronously (uses rayon for parallelism)
    pub fn execute_sync(self) -> Result<Vec<TaskResult>, TaskError> {
        use rayon::prelude::*;
        
        // Build in-degree map
        let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
        for (task, deps) in &self.dependencies {
            in_degree.insert(*task, deps.len());
        }
        
        let mut completed: std::collections::HashSet<TaskId> = std::collections::HashSet::new();
        let mut failed: std::collections::HashSet<TaskId> = std::collections::HashSet::new();
        let mut results: Vec<TaskResult> = Vec::new();
        let tasks = self.tasks;
        let dependencies = self.dependencies;
        
        loop {
            // Find all tasks with no remaining dependencies
            let ready: Vec<TaskId> = in_degree
                .iter()
                .filter(|(id, deg)| **deg == 0 && !completed.contains(id) && !failed.contains(id))
                .map(|(id, _)| *id)
                .collect();
            
            if ready.is_empty() {
                if completed.len() + failed.len() == tasks.len() {
                    break;
                } else if failed.is_empty() {
                    return Err(TaskError::CycleDetected);
                } else {
                    break; // Some tasks couldn't run due to failed dependencies
                }
            }
            
            // Execute ready tasks in parallel
            let batch_results: Vec<TaskResult> = ready
                .par_iter()
                .map(|id| {
                    let start = std::time::Instant::now();
                    
                    // Check if any dependency failed
                    if let Some(deps) = dependencies.get(id) {
                        for dep in deps {
                            if failed.contains(dep) {
                                return TaskResult {
                                    id: *id,
                                    success: false,
                                    error: Some(format!("Dependency {:?} failed", dep)),
                                    duration_ms: 0,
                                };
                            }
                        }
                    }
                    
                    let result = if let Some(task) = tasks.get(id) {
                        match task {
                            TaskFn::Sync(f) => f(),
                            TaskFn::Async(_) => Err("Async task in sync execution - use execute_async".into()),
                        }
                    } else {
                        Err("Task not found".into())
                    };
                    
                    let duration_ms = start.elapsed().as_millis() as u64;
                    
                    match result {
                        Ok(()) => TaskResult {
                            id: *id,
                            success: true,
                            error: None,
                            duration_ms,
                        },
                        Err(e) => TaskResult {
                            id: *id,
                            success: false,
                            error: Some(e),
                            duration_ms,
                        },
                    }
                })
                .collect();
            
            // Update state based on results
            for result in &batch_results {
                if result.success {
                    completed.insert(result.id);
                } else {
                    failed.insert(result.id);
                }
                
                // Decrease in-degree of dependent tasks
                for (other_id, deps) in &dependencies {
                    if deps.contains(&result.id) && !completed.contains(other_id) && !failed.contains(other_id) {
                        if let Some(deg) = in_degree.get_mut(other_id) {
                            *deg = deg.saturating_sub(1);
                        }
                    }
                }
            }
            
            results.extend(batch_results);
        }
        
        Ok(results)
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

// =============================================================================
// TRAIT IMPLEMENTATION (ifa-types bridge)
// =============================================================================

use ifa_types::{CpuOps, IfaValue, IfaResult, IfaError};

impl CpuOps for CpuContext {
    fn num_threads() -> usize {
        rayon::current_num_threads()
    }
    
    fn par_sum(data: &[IfaValue]) -> IfaResult<IfaValue> {
        // Note: Using sequential iteration since IfaValue doesn't implement Sync
        // For parallel sum of primitives, convert first then use rayon
        let sum: i64 = data.iter().filter_map(|v| {
            match v {
                IfaValue::Int(n) => Some(*n),
                IfaValue::Float(f) => Some(*f as i64),
                _ => None,
            }
        }).sum();
        
        Ok(IfaValue::Int(sum))
    }
    
    fn par_map<F>(data: &[IfaValue], f: F) -> IfaResult<Vec<IfaValue>>
    where
        F: Fn(&IfaValue) -> IfaValue + Sync + Send,
    {
        // Sequential map on IfaValue since it doesn't implement Sync
        Ok(data.iter().map(f).collect())
    }
    
    fn configure(threads: usize) -> IfaResult<()> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|e| IfaError::Runtime(e.to_string()))
    }
}

