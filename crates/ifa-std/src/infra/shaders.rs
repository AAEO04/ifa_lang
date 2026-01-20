//! # GPU Shader Library
//!
//! Production-quality WGSL compute shaders for Ifá-Lang.
//! Includes tiled matrix multiplication, reduction operations, and elementwise ops.

/// Tiled Matrix Multiplication Shader
/// 
/// Uses workgroup shared memory for cache efficiency.
/// Tile size: 16x16 (256 threads per workgroup)
pub const TILED_MATMUL_SHADER: &str = r#"
// Tiled Matrix Multiplication
// A[M,K] * B[K,N] = C[M,N]
// Uses shared memory tiling for better cache utilization

struct MatmulParams {
    M: u32,
    N: u32,
    K: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> params: MatmulParams;
@group(0) @binding(1) var<storage, read> A: array<f32>;
@group(0) @binding(2) var<storage, read> B: array<f32>;
@group(0) @binding(3) var<storage, read_write> C: array<f32>;

const TILE_SIZE: u32 = 16u;

var<workgroup> tile_A: array<array<f32, 16>, 16>;
var<workgroup> tile_B: array<array<f32, 16>, 16>;

@compute @workgroup_size(16, 16)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let row = global_id.y;
    let col = global_id.x;
    let local_row = local_id.y;
    let local_col = local_id.x;
    
    var sum: f32 = 0.0;
    let num_tiles = (params.K + TILE_SIZE - 1u) / TILE_SIZE;
    
    for (var t: u32 = 0u; t < num_tiles; t = t + 1u) {
        // Load tile from A into shared memory
        let a_col = t * TILE_SIZE + local_col;
        if (row < params.M && a_col < params.K) {
            tile_A[local_row][local_col] = A[row * params.K + a_col];
        } else {
            tile_A[local_row][local_col] = 0.0;
        }
        
        // Load tile from B into shared memory
        let b_row = t * TILE_SIZE + local_row;
        if (b_row < params.K && col < params.N) {
            tile_B[local_row][local_col] = B[b_row * params.N + col];
        } else {
            tile_B[local_row][local_col] = 0.0;
        }
        
        // Synchronize to ensure tiles are loaded
        workgroupBarrier();
        
        // Compute partial dot product
        for (var k: u32 = 0u; k < TILE_SIZE; k = k + 1u) {
            sum = sum + tile_A[local_row][k] * tile_B[k][local_col];
        }
        
        // Synchronize before loading next tile
        workgroupBarrier();
    }
    
    // Write result
    if (row < params.M && col < params.N) {
        C[row * params.N + col] = sum;
    }
}
"#;

/// Parallel Reduction Shader (Sum)
/// 
/// Tree-based reduction for efficient summation.
pub const REDUCE_SUM_SHADER: &str = r#"
struct ReduceParams {
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> params: ReduceParams;
@group(0) @binding(1) var<storage, read_write> data: array<f32>;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_data: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let tid = local_id.x;
    let gid = global_id.x;
    
    // Load into shared memory
    if (gid < params.count) {
        shared_data[tid] = data[gid];
    } else {
        shared_data[tid] = 0.0;
    }
    workgroupBarrier();
    
    // Tree reduction
    for (var stride: u32 = WORKGROUP_SIZE / 2u; stride > 0u; stride = stride / 2u) {
        if (tid < stride) {
            shared_data[tid] = shared_data[tid] + shared_data[tid + stride];
        }
        workgroupBarrier();
    }
    
    // Write result from first thread
    if (tid == 0u) {
        data[group_id.x] = shared_data[0];
    }
}
"#;

/// Parallel Reduction Shader (Max)
pub const REDUCE_MAX_SHADER: &str = r#"
struct ReduceParams {
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> params: ReduceParams;
@group(0) @binding(1) var<storage, read_write> data: array<f32>;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_data: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let tid = local_id.x;
    let gid = global_id.x;
    
    if (gid < params.count) {
        shared_data[tid] = data[gid];
    } else {
        shared_data[tid] = -3.402823e+38; // -FLT_MAX
    }
    workgroupBarrier();
    
    for (var stride: u32 = WORKGROUP_SIZE / 2u; stride > 0u; stride = stride / 2u) {
        if (tid < stride) {
            shared_data[tid] = max(shared_data[tid], shared_data[tid + stride]);
        }
        workgroupBarrier();
    }
    
    if (tid == 0u) {
        data[group_id.x] = shared_data[0];
    }
}
"#;

/// Elementwise Map Shader (x * scale + bias)
pub const MAP_SCALE_BIAS_SHADER: &str = r#"
struct MapParams {
    count: u32,
    scale: f32,
    bias: f32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> params: MapParams;
@group(0) @binding(1) var<storage, read> input: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i < params.count) {
        output[i] = input[i] * params.scale + params.bias;
    }
}
"#;

/// ReLU Activation Shader
pub const RELU_SHADER: &str = r#"
struct Params {
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i < params.count) {
        data[i] = max(0.0, data[i]);
    }
}
"#;

/// Softmax Shader (for small vectors, single workgroup)
pub const SOFTMAX_SHADER: &str = r#"
struct Params {
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> data: array<f32>;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_max: array<f32, 256>;
var<workgroup> shared_sum: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    let tid = local_id.x;
    
    // Find max (for numerical stability)
    if (tid < params.count) {
        shared_max[tid] = data[tid];
    } else {
        shared_max[tid] = -3.402823e+38;
    }
    workgroupBarrier();
    
    for (var stride: u32 = WORKGROUP_SIZE / 2u; stride > 0u; stride = stride / 2u) {
        if (tid < stride) {
            shared_max[tid] = max(shared_max[tid], shared_max[tid + stride]);
        }
        workgroupBarrier();
    }
    let max_val = shared_max[0];
    workgroupBarrier();
    
    // Compute exp(x - max) and sum
    if (tid < params.count) {
        let exp_val = exp(data[tid] - max_val);
        data[tid] = exp_val;
        shared_sum[tid] = exp_val;
    } else {
        shared_sum[tid] = 0.0;
    }
    workgroupBarrier();
    
    for (var stride: u32 = WORKGROUP_SIZE / 2u; stride > 0u; stride = stride / 2u) {
        if (tid < stride) {
            shared_sum[tid] = shared_sum[tid] + shared_sum[tid + stride];
        }
        workgroupBarrier();
    }
    let sum_val = shared_sum[0];
    workgroupBarrier();
    
    // Normalize
    if (tid < params.count) {
        data[tid] = data[tid] / sum_val;
    }
}
"#;

/// Vector addition shader
pub const VEC_ADD_SHADER: &str = r#"
struct Params {
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> a: array<f32>;
@group(0) @binding(2) var<storage, read> b: array<f32>;
@group(0) @binding(3) var<storage, read_write> c: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i < params.count) {
        c[i] = a[i] + b[i];
    }
}
"#;

/// Dot product shader (partial, requires reduction)
pub const DOT_PRODUCT_SHADER: &str = r#"
struct Params {
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> a: array<f32>;
@group(0) @binding(2) var<storage, read> b: array<f32>;
@group(0) @binding(3) var<storage, read_write> partial: array<f32>;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_data: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let tid = local_id.x;
    let gid = global_id.x;
    
    if (gid < params.count) {
        shared_data[tid] = a[gid] * b[gid];
    } else {
        shared_data[tid] = 0.0;
    }
    workgroupBarrier();
    
    for (var stride: u32 = WORKGROUP_SIZE / 2u; stride > 0u; stride = stride / 2u) {
        if (tid < stride) {
            shared_data[tid] = shared_data[tid] + shared_data[tid + stride];
        }
        workgroupBarrier();
    }
    
    if (tid == 0u) {
        partial[group_id.x] = shared_data[0];
    }
}
"#;
