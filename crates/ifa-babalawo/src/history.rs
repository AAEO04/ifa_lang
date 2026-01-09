//! # State History Buffer - Time-Travel Debugging
//!
//! 32-step circular buffer for state snapshots.
//! Enables "rewind" debugging capability.
//! Ported from legacy/src/errors.py StateHistoryBuffer

use std::collections::HashMap;

/// A state snapshot at a point in execution
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub step: usize,
    pub line: usize,
    pub variables: HashMap<String, String>,
    pub call_stack: Vec<String>,
    pub timestamp: u64,
}

impl StateSnapshot {
    pub fn new(step: usize, line: usize) -> Self {
        Self {
            step,
            line,
            variables: HashMap::new(),
            call_stack: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    pub fn with_variable(mut self, name: &str, value: &str) -> Self {
        self.variables.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_call(mut self, func: &str) -> Self {
        self.call_stack.push(func.to_string());
        self
    }
}

/// 32-step circular buffer for time-travel debugging
pub struct StateHistoryBuffer {
    buffer: Vec<StateSnapshot>,
    capacity: usize,
    index: usize,
    total_steps: usize,
}

impl Default for StateHistoryBuffer {
    fn default() -> Self {
        Self::new(32)
    }
}

impl StateHistoryBuffer {
    /// Create a new buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            index: 0,
            total_steps: 0,
        }
    }

    /// Push a state snapshot to the buffer
    pub fn push(&mut self, state: StateSnapshot) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(state);
        } else {
            self.buffer[self.index] = state;
        }
        self.index = (self.index + 1) % self.capacity;
        self.total_steps += 1;
    }

    /// Go back N steps in history
    pub fn rewind(&self, steps: usize) -> Option<&StateSnapshot> {
        if self.buffer.is_empty() {
            return None;
        }

        let steps = steps.min(self.buffer.len() - 1);
        let idx = if self.index >= steps + 1 {
            self.index - steps - 1
        } else {
            self.buffer.len() - (steps + 1 - self.index)
        };

        self.buffer.get(idx)
    }

    /// Get the most recent state
    pub fn current(&self) -> Option<&StateSnapshot> {
        self.rewind(0)
    }

    /// Get the number of states in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get total steps recorded (including overwritten)
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.index = 0;
        self.total_steps = 0;
    }

    /// Get all states in chronological order
    pub fn history(&self) -> Vec<&StateSnapshot> {
        if self.buffer.len() < self.capacity {
            self.buffer.iter().collect()
        } else {
            // Reorder from oldest to newest
            let mut result = Vec::with_capacity(self.capacity);
            for i in 0..self.capacity {
                let idx = (self.index + i) % self.capacity;
                result.push(&self.buffer[idx]);
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_rewind() {
        let mut buffer = StateHistoryBuffer::new(5);

        buffer.push(StateSnapshot::new(1, 10));
        buffer.push(StateSnapshot::new(2, 20));
        buffer.push(StateSnapshot::new(3, 30));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.current().unwrap().step, 3);
        assert_eq!(buffer.rewind(1).unwrap().step, 2);
        assert_eq!(buffer.rewind(2).unwrap().step, 1);
    }

    #[test]
    fn test_circular_buffer() {
        let mut buffer = StateHistoryBuffer::new(3);

        buffer.push(StateSnapshot::new(1, 10));
        buffer.push(StateSnapshot::new(2, 20));
        buffer.push(StateSnapshot::new(3, 30));
        buffer.push(StateSnapshot::new(4, 40)); // Overwrites step 1

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.total_steps(), 4);
        assert_eq!(buffer.current().unwrap().step, 4);
    }

    #[test]
    fn test_with_variables() {
        let state = StateSnapshot::new(1, 10)
            .with_variable("x", "42")
            .with_variable("name", "test");

        assert_eq!(state.variables.get("x"), Some(&"42".to_string()));
        assert_eq!(state.variables.get("name"), Some(&"test".to_string()));
    }
}
