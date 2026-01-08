//! # ML Stack
//! 
//! Extensions for machine learning and data science.
//!
//! Features:
//! - Tensor operations with proper error handling
//! - In-place operations for memory efficiency
//! - Numerically stable activations
//! - Broadcasting support (basic)
//! 
//! Uses: ndarray, candle (when stable)

use std::fmt;

/// Errors for tensor operations
#[derive(Debug, Clone)]
pub enum TensorError {
    ShapeMismatch { expected: Vec<usize>, actual: Vec<usize> },
    InvalidAxis { axis: usize, ndim: usize },
    DivisionByZero,
    InvalidShape(String),
    BroadcastError(String),
}

impl fmt::Display for TensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch { expected, actual } =>
                write!(f, "Shape mismatch: expected {:?}, got {:?}", expected, actual),
            Self::InvalidAxis { axis, ndim } =>
                write!(f, "Invalid axis {} for tensor with {} dimensions", axis, ndim),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::InvalidShape(msg) => write!(f, "Invalid shape: {}", msg),
            Self::BroadcastError(msg) => write!(f, "Broadcast error: {}", msg),
        }
    }
}

impl std::error::Error for TensorError {}

pub type TensorResult<T> = Result<T, TensorError>;

/// Tensor with improved operations
#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: Vec<f64>,
    pub shape: Vec<usize>,
    strides: Vec<usize>,
}

impl Tensor {
    /// Create tensor from data and shape with validation
    pub fn new(data: Vec<f64>, shape: Vec<usize>) -> TensorResult<Self> {
        let expected_len: usize = shape.iter().product();
        if expected_len == 0 && !shape.is_empty() && data.is_empty() {
            // Allow empty tensors
        } else if data.len() != expected_len {
            return Err(TensorError::InvalidShape(
                format!("Data length {} doesn't match shape {:?} (expected {})", 
                    data.len(), shape, expected_len)
            ));
        }
        
        let strides = Self::compute_strides(&shape);
        Ok(Tensor { data, shape, strides })
    }
    
    /// Unchecked creation (for internal use)
    fn new_unchecked(data: Vec<f64>, shape: Vec<usize>) -> Self {
        let strides = Self::compute_strides(&shape);
        Tensor { data, shape, strides }
    }
    
    fn compute_strides(shape: &[usize]) -> Vec<usize> {
        let mut strides = vec![1; shape.len()];
        for i in (0..shape.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * shape[i + 1];
        }
        strides
    }
    
    /// Create zeros tensor
    pub fn zeros(shape: &[usize]) -> Self {
        let len: usize = shape.iter().product();
        Self::new_unchecked(vec![0.0; len], shape.to_vec())
    }
    
    /// Create ones tensor
    pub fn ones(shape: &[usize]) -> Self {
        let len: usize = shape.iter().product();
        Self::new_unchecked(vec![1.0; len], shape.to_vec())
    }
    
    /// Create tensor filled with value
    pub fn full(shape: &[usize], value: f64) -> Self {
        let len: usize = shape.iter().product();
        Self::new_unchecked(vec![value; len], shape.to_vec())
    }
    
    /// Create from nested vectors (2D)
    pub fn from_2d(data: &[&[f64]]) -> TensorResult<Self> {
        let rows = data.len();
        if rows == 0 {
            return Self::new(vec![], vec![0, 0]);
        }
        let cols = data[0].len();
        for (i, row) in data.iter().enumerate() {
            if row.len() != cols {
                return Err(TensorError::InvalidShape(
                    format!("Row {} has length {}, expected {}", i, row.len(), cols)
                ));
            }
        }
        let flat: Vec<f64> = data.iter().flat_map(|row| row.iter().copied()).collect();
        Self::new(flat, vec![rows, cols])
    }
    
    /// Create 1D tensor from slice
    pub fn from_slice(data: &[f64]) -> Self {
        Self::new_unchecked(data.to_vec(), vec![data.len()])
    }
    
    /// Random tensor with uniform distribution [0, 1)
    pub fn rand(shape: &[usize]) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        let len: usize = shape.iter().product();
        let data: Vec<f64> = (0..len).map(|_| {
            // Simple LCG for deterministic pseudo-random
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 33) as f64 / (1u64 << 31) as f64
        }).collect();
        
        Self::new_unchecked(data, shape.to_vec())
    }
    
    /// Random tensor with normal distribution (Box-Muller)
    pub fn randn(shape: &[usize]) -> Self {
        let uniform = Self::rand(shape);
        let mut data = vec![0.0; uniform.numel()];
        
        for i in (0..data.len()).step_by(2) {
            let u1 = uniform.data[i].max(1e-10);
            let u2 = if i + 1 < uniform.data.len() { uniform.data[i + 1] } else { 0.5 };
            
            let r = (-2.0 * u1.ln()).sqrt();
            let theta = 2.0 * std::f64::consts::PI * u2;
            
            data[i] = r * theta.cos();
            if i + 1 < data.len() {
                data[i + 1] = r * theta.sin();
            }
        }
        
        Self::new_unchecked(data, shape.to_vec())
    }
    
    // ==================== Properties ====================
    
    /// Get number of dimensions
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }
    
    /// Get total number of elements
    pub fn numel(&self) -> usize {
        self.data.len()
    }
    
    /// Check if tensor is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    // ==================== Indexing ====================
    
    /// Get element at index
    pub fn get(&self, indices: &[usize]) -> TensorResult<f64> {
        let flat_idx = self.flat_index(indices)?;
        Ok(self.data[flat_idx])
    }
    
    /// Set element at index
    pub fn set(&mut self, indices: &[usize], value: f64) -> TensorResult<()> {
        let flat_idx = self.flat_index(indices)?;
        self.data[flat_idx] = value;
        Ok(())
    }
    
    fn flat_index(&self, indices: &[usize]) -> TensorResult<usize> {
        if indices.len() != self.shape.len() {
            return Err(TensorError::InvalidAxis { 
                axis: indices.len(), 
                ndim: self.shape.len() 
            });
        }
        
        let mut idx = 0;
        for (i, (&index, &stride)) in indices.iter().zip(self.strides.iter()).enumerate() {
            if index >= self.shape[i] {
                return Err(TensorError::InvalidShape(
                    format!("Index {} out of bounds for axis {} with size {}", 
                        index, i, self.shape[i])
                ));
            }
            idx += index * stride;
        }
        Ok(idx)
    }
    
    // ==================== Shape operations ====================
    
    /// Reshape tensor (must have same total elements)
    pub fn reshape(&self, new_shape: &[usize]) -> TensorResult<Self> {
        let new_len: usize = new_shape.iter().product();
        if self.data.len() != new_len {
            return Err(TensorError::InvalidShape(
                format!("Cannot reshape {} elements to shape {:?}", self.data.len(), new_shape)
            ));
        }
        Self::new(self.data.clone(), new_shape.to_vec())
    }
    
    /// Reshape in-place
    pub fn reshape_mut(&mut self, new_shape: &[usize]) -> TensorResult<()> {
        let new_len: usize = new_shape.iter().product();
        if self.data.len() != new_len {
            return Err(TensorError::InvalidShape(
                format!("Cannot reshape {} elements to shape {:?}", self.data.len(), new_shape)
            ));
        }
        self.shape = new_shape.to_vec();
        self.strides = Self::compute_strides(&self.shape);
        Ok(())
    }
    
    /// Transpose 2D tensor
    pub fn transpose(&self) -> TensorResult<Self> {
        if self.ndim() != 2 {
            return Err(TensorError::InvalidAxis { axis: 0, ndim: self.ndim() });
        }
        
        let (rows, cols) = (self.shape[0], self.shape[1]);
        let mut data = vec![0.0; self.numel()];
        
        for i in 0..rows {
            for j in 0..cols {
                data[j * rows + i] = self.data[i * cols + j];
            }
        }
        
        Self::new(data, vec![cols, rows])
    }
    
    /// Flatten to 1D
    pub fn flatten(&self) -> Self {
        Self::new_unchecked(self.data.clone(), vec![self.numel()])
    }
    
    // ==================== Element-wise operations ====================
    
    /// Element-wise addition
    pub fn add(&self, other: &Tensor) -> TensorResult<Self> {
        self.check_shapes(other)?;
        let data: Vec<f64> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Self::new(data, self.shape.clone())
    }
    
    /// In-place addition
    pub fn add_mut(&mut self, other: &Tensor) -> TensorResult<()> {
        self.check_shapes(other)?;
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a += *b;
        }
        Ok(())
    }
    
    /// Element-wise subtraction
    pub fn sub(&self, other: &Tensor) -> TensorResult<Self> {
        self.check_shapes(other)?;
        let data: Vec<f64> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect();
        Self::new(data, self.shape.clone())
    }
    
    /// In-place subtraction
    pub fn sub_mut(&mut self, other: &Tensor) -> TensorResult<()> {
        self.check_shapes(other)?;
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a -= *b;
        }
        Ok(())
    }
    
    /// Element-wise multiplication (Hadamard product)
    pub fn mul(&self, other: &Tensor) -> TensorResult<Self> {
        self.check_shapes(other)?;
        let data: Vec<f64> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .collect();
        Self::new(data, self.shape.clone())
    }
    
    /// In-place multiplication
    pub fn mul_mut(&mut self, other: &Tensor) -> TensorResult<()> {
        self.check_shapes(other)?;
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a *= *b;
        }
        Ok(())
    }
    
    /// Element-wise division with safety
    pub fn div(&self, other: &Tensor) -> TensorResult<Self> {
        self.check_shapes(other)?;
        if other.data.iter().any(|&x| x == 0.0) {
            return Err(TensorError::DivisionByZero);
        }
        let data: Vec<f64> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a / b)
            .collect();
        Self::new(data, self.shape.clone())
    }
    
    /// Scalar addition
    pub fn add_scalar(&self, scalar: f64) -> Self {
        let data: Vec<f64> = self.data.iter().map(|x| x + scalar).collect();
        Self::new_unchecked(data, self.shape.clone())
    }
    
    /// Scalar multiplication
    pub fn scale(&self, scalar: f64) -> Self {
        let data: Vec<f64> = self.data.iter().map(|x| x * scalar).collect();
        Self::new_unchecked(data, self.shape.clone())
    }
    
    /// In-place scalar multiplication
    pub fn scale_mut(&mut self, scalar: f64) {
        for x in self.data.iter_mut() {
            *x *= scalar;
        }
    }
    
    fn check_shapes(&self, other: &Tensor) -> TensorResult<()> {
        if self.shape != other.shape {
            return Err(TensorError::ShapeMismatch {
                expected: self.shape.clone(),
                actual: other.shape.clone(),
            });
        }
        Ok(())
    }
    
    // ==================== Matrix operations ====================
    
    /// Matrix multiplication (2D only)
    pub fn matmul(&self, other: &Tensor) -> TensorResult<Self> {
        if self.ndim() != 2 || other.ndim() != 2 {
            return Err(TensorError::InvalidShape(
                "matmul requires 2D tensors".to_string()
            ));
        }
        if self.shape[1] != other.shape[0] {
            return Err(TensorError::ShapeMismatch {
                expected: vec![self.shape[0], other.shape[0]],
                actual: vec![self.shape[0], self.shape[1]],
            });
        }
        
        let (m, k) = (self.shape[0], self.shape[1]);
        let n = other.shape[1];
        
        let mut data = vec![0.0; m * n];
        
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k {
                    sum += self.data[i * k + p] * other.data[p * n + j];
                }
                data[i * n + j] = sum;
            }
        }
        
        Self::new(data, vec![m, n])
    }
    
    // ==================== Reductions ====================
    
    /// Sum all elements
    pub fn sum(&self) -> f64 {
        self.data.iter().sum()
    }
    
    /// Mean of all elements
    pub fn mean(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        self.sum() / self.numel() as f64
    }
    
    /// Standard deviation
    pub fn std(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self.data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / self.numel() as f64;
        variance.sqrt()
    }
    
    /// Maximum value
    pub fn max(&self) -> f64 {
        self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
    
    /// Minimum value  
    pub fn min(&self) -> f64 {
        self.data.iter().cloned().fold(f64::INFINITY, f64::min)
    }
    
    /// Index of maximum value
    pub fn argmax(&self) -> usize {
        self.data.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    
    // ==================== Activations (numerically stable) ====================
    
    /// Apply function to each element
    pub fn map<F: Fn(f64) -> f64>(&self, f: F) -> Self {
        let data: Vec<f64> = self.data.iter().map(|x| f(*x)).collect();
        Self::new_unchecked(data, self.shape.clone())
    }
    
    /// In-place map
    pub fn map_mut<F: Fn(f64) -> f64>(&mut self, f: F) {
        for x in self.data.iter_mut() {
            *x = f(*x);
        }
    }
    
    /// ReLU activation
    pub fn relu(&self) -> Self {
        self.map(|x| x.max(0.0))
    }
    
    /// Leaky ReLU
    pub fn leaky_relu(&self, alpha: f64) -> Self {
        self.map(|x| if x > 0.0 { x } else { alpha * x })
    }
    
    /// Sigmoid activation (numerically stable)
    pub fn sigmoid(&self) -> Self {
        self.map(|x| {
            if x >= 0.0 {
                1.0 / (1.0 + (-x).exp())
            } else {
                let ex = x.exp();
                ex / (1.0 + ex)
            }
        })
    }
    
    /// Tanh activation  
    pub fn tanh(&self) -> Self {
        self.map(|x| x.tanh())
    }
    
    /// Softmax (numerically stable with max subtraction)
    pub fn softmax(&self) -> Self {
        let max_val = self.max();
        let exp_vals: Vec<f64> = self.data.iter()
            .map(|x| (x - max_val).exp())
            .collect();
        let sum: f64 = exp_vals.iter().sum();
        let data: Vec<f64> = exp_vals.iter().map(|x| x / sum).collect();
        Self::new_unchecked(data, self.shape.clone())
    }
    
    /// Log softmax (more stable for cross-entropy)
    pub fn log_softmax(&self) -> Self {
        let max_val = self.max();
        let shifted: Vec<f64> = self.data.iter().map(|x| x - max_val).collect();
        let log_sum_exp: f64 = shifted.iter().map(|x| x.exp()).sum::<f64>().ln();
        let data: Vec<f64> = shifted.iter().map(|x| x - log_sum_exp).collect();
        Self::new_unchecked(data, self.shape.clone())
    }
    
    /// Clamp values to range
    pub fn clamp(&self, min: f64, max: f64) -> Self {
        self.map(|x| x.max(min).min(max))
    }
}

impl fmt::Display for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ndim() == 1 {
            write!(f, "Tensor([{:.4?}])", self.data)
        } else if self.ndim() == 2 {
            writeln!(f, "Tensor([")?;
            for i in 0..self.shape[0] {
                let start = i * self.shape[1];
                let end = start + self.shape[1];
                writeln!(f, "  {:?},", &self.data[start..end])?;
            }
            write!(f, "], shape={:?})", self.shape)
        } else {
            write!(f, "Tensor(shape={:?})", self.shape)
        }
    }
}

/// Simple neural network layer
pub struct Linear {
    pub weights: Tensor,
    pub bias: Tensor,
}

impl Linear {
    /// Create linear layer (input_size -> output_size)
    pub fn new(input_size: usize, output_size: usize) -> Self {
        // Xavier/He initialization
        let scale = (2.0 / input_size as f64).sqrt();
        let weights = Tensor::randn(&[input_size, output_size]).scale(scale);
        
        Linear {
            weights,
            bias: Tensor::zeros(&[output_size]),
        }
    }
    
    /// Forward pass
    pub fn forward(&self, input: &Tensor) -> TensorResult<Tensor> {
        let output = input.matmul(&self.weights)?;
        // Add bias (broadcasting over batch)
        let mut result = output.data.clone();
        let cols = self.bias.numel();
        for i in 0..result.len() {
            result[i] += self.bias.data[i % cols];
        }
        Tensor::new(result, output.shape)
    }
}

/// Loss functions
pub mod loss {
    use super::Tensor;
    
    const EPS: f64 = 1e-7;
    
    /// Mean Squared Error
    pub fn mse(predicted: &Tensor, target: &Tensor) -> f64 {
        if predicted.shape != target.shape {
            return f64::NAN;
        }
        predicted.data.iter()
            .zip(target.data.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>() / predicted.numel() as f64
    }
    
    /// Binary Cross-Entropy (with numerical stability)
    pub fn binary_cross_entropy(predicted: &Tensor, target: &Tensor) -> f64 {
        if predicted.shape != target.shape {
            return f64::NAN;
        }
        -predicted.data.iter()
            .zip(target.data.iter())
            .map(|(p, t)| {
                let p_clipped = p.max(EPS).min(1.0 - EPS);
                t * p_clipped.ln() + (1.0 - t) * (1.0 - p_clipped).ln()
            })
            .sum::<f64>() / predicted.numel() as f64
    }
    
    /// Categorical Cross-Entropy
    pub fn cross_entropy(predicted: &Tensor, target: &Tensor) -> f64 {
        if predicted.shape != target.shape {
            return f64::NAN;
        }
        -predicted.data.iter()
            .zip(target.data.iter())
            .map(|(p, t)| t * p.max(EPS).ln())
            .sum::<f64>() / predicted.numel() as f64
    }
}

/// Optimizer trait
pub trait Optimizer {
    fn step(&mut self, params: &mut [Tensor], grads: &[Tensor]);
}

/// Stochastic Gradient Descent
pub struct SGD {
    pub learning_rate: f64,
}

impl SGD {
    pub fn new(learning_rate: f64) -> Self {
        SGD { learning_rate }
    }
}

impl Optimizer for SGD {
    fn step(&mut self, params: &mut [Tensor], grads: &[Tensor]) {
        for (param, grad) in params.iter_mut().zip(grads.iter()) {
            for (p, g) in param.data.iter_mut().zip(grad.data.iter()) {
                *p -= self.learning_rate * g;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tensor_creation() {
        let t = Tensor::zeros(&[2, 3]);
        assert_eq!(t.shape, vec![2, 3]);
        assert_eq!(t.numel(), 6);
    }
    
    #[test]
    fn test_tensor_error() {
        let result = Tensor::new(vec![1.0, 2.0], vec![3]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_tensor_arithmetic() {
        let a = Tensor::ones(&[2, 2]);
        let b = Tensor::full(&[2, 2], 2.0);
        
        let c = a.add(&b).unwrap();
        assert_eq!(c.data, vec![3.0, 3.0, 3.0, 3.0]);
    }
    
    #[test]
    fn test_inplace_ops() {
        let mut a = Tensor::ones(&[2, 2]);
        let b = Tensor::full(&[2, 2], 2.0);
        
        a.add_mut(&b).unwrap();
        assert_eq!(a.data, vec![3.0, 3.0, 3.0, 3.0]);
    }
    
    #[test]
    fn test_matmul() {
        let a = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = Tensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
        
        let c = a.matmul(&b).unwrap();
        assert_eq!(c.data, vec![1.0, 2.0, 3.0, 4.0]);
    }
    
    #[test]
    fn test_activations() {
        let t = Tensor::new(vec![-1.0, 0.0, 1.0], vec![3]).unwrap();
        
        let relu = t.relu();
        assert_eq!(relu.data, vec![0.0, 0.0, 1.0]);
        
        let sig = t.sigmoid();
        assert!((sig.data[1] - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_softmax_stability() {
        // Large values that would overflow without max subtraction
        let t = Tensor::new(vec![1000.0, 1001.0, 1002.0], vec![3]).unwrap();
        let sm = t.softmax();
        
        // Should not be NaN
        assert!(!sm.data[0].is_nan());
        // Should sum to 1
        let sum: f64 = sm.data.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_linear() {
        let layer = Linear::new(3, 2);
        let input = Tensor::ones(&[1, 3]);
        let output = layer.forward(&input).unwrap();
        
        assert_eq!(output.shape, vec![1, 2]);
    }
}
