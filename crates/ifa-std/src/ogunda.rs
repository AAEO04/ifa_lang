//! # Ògúndá Domain (1110)
//! 
//! The Warrior - Arrays and Process Management
//! 
//! Vec with iterators and subprocess spawning via tokio-process.

use std::process::{Command, Output, Stdio};
use ifa_core::error::{IfaError, IfaResult};
use crate::impl_odu_domain;

/// Ògúndá - The Warrior (Arrays/Processes)
pub struct Ogunda;

impl_odu_domain!(Ogunda, "Ògúndá", "1110", "The Warrior - Arrays/Processes");

impl Ogunda {
    // =========================================================================
    // ARRAY OPERATIONS
    // =========================================================================
    
    /// Create new list (ṣẹ̀dá)
    pub fn seda<T>(&self) -> Vec<T> {
        Vec::new()
    }
    
    /// Create list with capacity
    pub fn seda_agbara<T>(&self, capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }
    
    /// Push to list (fí)
    pub fn fi<T>(&self, list: &mut Vec<T>, item: T) {
        list.push(item);
    }
    
    /// Pop from list (mú)
    pub fn mu<T>(&self, list: &mut Vec<T>) -> Option<T> {
        list.pop()
    }
    
    /// Get length (ìwọ̀n)
    pub fn iwon<T>(&self, list: &[T]) -> usize {
        list.len()
    }
    
    /// Check if empty (ṣófo)
    pub fn sofo<T>(&self, list: &[T]) -> bool {
        list.is_empty()
    }
    
    /// Reverse list (padà)
    pub fn pada<T: Clone>(&self, list: &[T]) -> Vec<T> {
        list.iter().cloned().rev().collect()
    }
    
    /// Sort list (tò)
    pub fn to<T: Ord + Clone>(&self, list: &[T]) -> Vec<T> {
        let mut sorted = list.to_vec();
        sorted.sort();
        sorted
    }
    
    /// Concatenate lists (dàpọ̀)
    pub fn dapo<T: Clone>(&self, a: &[T], b: &[T]) -> Vec<T> {
        let mut result = a.to_vec();
        result.extend(b.iter().cloned());
        result
    }
    
    /// Filter list (yàn)
    pub fn yan<T: Clone, F: Fn(&T) -> bool>(&self, list: &[T], predicate: F) -> Vec<T> {
        list.iter().filter(|x| predicate(x)).cloned().collect()
    }
    
    /// Map over list (yí padà)
    pub fn yi_pada<T, U, F: Fn(T) -> U>(&self, list: Vec<T>, mapper: F) -> Vec<U> {
        list.into_iter().map(mapper).collect()
    }
    
    /// Find first matching (wá)
    pub fn wa<'a, T, F: Fn(&T) -> bool>(&self, list: &'a [T], predicate: F) -> Option<&'a T> {
        list.iter().find(|x| predicate(x))
    }
    
    /// Check if any match (èyíkéyìí)
    pub fn eyikeyi<T, F: Fn(&T) -> bool>(&self, list: &[T], predicate: F) -> bool {
        list.iter().any(|x| predicate(x))
    }
    
    /// Check if all match (gbogbo)
    pub fn gbogbo<T, F: Fn(&T) -> bool>(&self, list: &[T], predicate: F) -> bool {
        list.iter().all(|x| predicate(x))
    }
    
    /// Slice list
    pub fn ge<T: Clone>(&self, list: &[T], start: usize, end: usize) -> Vec<T> {
        list.get(start..end).map(|s| s.to_vec()).unwrap_or_default()
    }
    
    // =========================================================================
    // PROCESS OPERATIONS
    // =========================================================================
    
    /// Run command and get output (ṣiṣẹ́)
    pub fn sise(&self, command: &str, args: &[&str]) -> IfaResult<Output> {
        Command::new(command)
            .args(args)
            .output()
            .map_err(|e| IfaError::Custom(format!("Process error: {}", e)))
    }
    
    /// Run command and get stdout as string
    pub fn sise_ka(&self, command: &str, args: &[&str]) -> IfaResult<String> {
        let output = self.sise(command, args)?;
        String::from_utf8(output.stdout)
            .map_err(|e| IfaError::Custom(format!("UTF-8 error: {}", e)))
    }
    
    /// Spawn detached process (bẹ̀rẹ̀)
    pub fn bere(&self, command: &str, args: &[&str]) -> IfaResult<u32> {
        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| IfaError::Custom(format!("Spawn error: {}", e)))?;
        Ok(child.id())
    }
    
    /// Get environment variable
    pub fn ayika(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_ops() {
        let ogunda = Ogunda;
        let mut list = vec![1, 2, 3];
        
        ogunda.fi(&mut list, 4);
        assert_eq!(list, vec![1, 2, 3, 4]);
        
        assert_eq!(ogunda.mu(&mut list), Some(4));
        assert_eq!(ogunda.iwon(&list), 3);
    }
    
    #[test]
    fn test_sort() {
        let ogunda = Ogunda;
        let list = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let sorted = ogunda.to(&list);
        assert_eq!(sorted, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }
    
    #[test]
    fn test_filter() {
        let ogunda = Ogunda;
        let list = vec![1, 2, 3, 4, 5, 6];
        let evens = ogunda.yan(&list, |x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4, 6]);
    }
}
