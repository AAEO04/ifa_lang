//! # Ògúndá Domain (1110)
//!
//! The Warrior - Arrays and Process Management
//!
//! Vec with iterators and subprocess spawning via tokio-process.

use crate::impl_odu_domain;
use ifa_core::error::{IfaError, IfaResult};
use ifa_core::value::IfaValue;
use std::process::{Command, Output, Stdio};

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
        list.iter().any(predicate)
    }

    /// Check if all match (gbogbo)
    pub fn gbogbo<T, F: Fn(&T) -> bool>(&self, list: &[T], predicate: F) -> bool {
        list.iter().all(predicate)
    }

    /// Slice list
    pub fn ge<T: Clone>(&self, list: &[T], start: usize, end: usize) -> IfaResult<Vec<T>> {
        if start > end {
            return Err(IfaError::IndexOutOfBounds {
                index: start as i64,
                length: list.len(),
            });
        }
        if end > list.len() {
            return Err(IfaError::IndexOutOfBounds {
                index: end as i64,
                length: list.len(),
            });
        }
        Ok(list[start..end].to_vec())
    }

    // =========================================================================
    // MAP OPERATIONS (Associative Arrays)
    // =========================================================================

    /// Get map keys as list (awọn_kokoro)
    pub fn awon_kokoro(&self, map: &IfaValue) -> IfaResult<Vec<String>> {
        match map {
            IfaValue::Map(m) => Ok(m.keys().map(|k| k.to_string()).collect()),
            _ => Err(IfaError::TypeError {
                expected: "Map or Object".into(),
                got: map.type_name().into(),
            }),
        }
    }

    /// Get map values as list (awọn_iye)
    pub fn awon_iye(&self, map: &IfaValue) -> IfaResult<Vec<IfaValue>> {
        match map {
            IfaValue::Map(m) => Ok(m.values().cloned().collect()),
            _ => Err(IfaError::TypeError {
                expected: "Map or Object".into(),
                got: map.type_name().into(),
            }),
        }
    }

    /// Get map items as list of [key, value] pairs (awọn_nkan)
    pub fn awon_nkan(&self, map: &IfaValue) -> IfaResult<Vec<Vec<IfaValue>>> {
        match map {
            IfaValue::Map(m) => Ok(m
                .iter()
                .map(|(k, v)| vec![IfaValue::Str(k.to_string().into()), v.clone()])
                .collect()),
            _ => Err(IfaError::TypeError {
                expected: "Map or Object".into(),
                got: map.type_name().into(),
            }),
        }
    }

    /// Remove key from map and return value (yọ)
    pub fn yo(&self, map: &mut IfaValue, key: &str) -> IfaResult<IfaValue> {
        match map {
            IfaValue::Map(map_arc) => {
                // Clone-on-Write for Map mutation
                let map = std::sync::Arc::make_mut(map_arc);
                // Need to handle key type (Arc<str>)
                // This is tricky if we don't have the exact key instance.
                // We iterate to find matching key string? Efficient? No.
                // But HashMap keys are Strings/Arc<str>.
                // We can simply remove by strict equality if the map key is `String` or `Arc<str>`.
                // IfaValue::Map uses `HashMap<Arc<str>, IfaValue>`.
                // `remove` takes `&Q` where `Arc<str>: Borrow<Q>`.
                // `str` works.
                Ok(map.remove(key).unwrap_or(IfaValue::Null))
            }
            _ => Err(IfaError::TypeError {
                expected: "Map or Object".into(),
                got: map.type_name().into(),
            }),
        }
    }

    /// English Aliases
    pub fn keys(&self, map: &IfaValue) -> IfaResult<Vec<String>> {
        self.awon_kokoro(map)
    }

    pub fn values(&self, map: &IfaValue) -> IfaResult<Vec<IfaValue>> {
        self.awon_iye(map)
    }

    pub fn items(&self, map: &IfaValue) -> IfaResult<Vec<Vec<IfaValue>>> {
        self.awon_nkan(map)
    }

    pub fn remove(&self, map: &mut IfaValue, key: &str) -> IfaResult<IfaValue> {
        self.yo(map, key)
    }
    // =========================================================================

    /// Dangerous shell metacharacters that could enable injection
    const SHELL_METACHARACTERS: &'static [char] = &[
        '|', '&', ';', '$', '`', '\n', '\r', '(', ')', '{', '}', '<', '>',
    ];

    /// dangerous shell interpreters that should not be spawned directly
    const BLOCKED_SHELLS: &'static [&'static str] = &[
        "cmd",
        "cmd.exe",
        "sh",
        "bash",
        "zsh",
        "powershell",
        "pwsh",
        "dash",
        "ksh",
        "csh",
        "tcsh",
    ];

    /// Validate command is safe (no path traversal, no shell builtins for injection)
    fn validate_command(command: &str) -> IfaResult<()> {
        // Block empty commands
        if command.is_empty() {
            return Err(IfaError::Custom("Empty command".to_string()));
        }

        // Block shell interpreters (use specific ifa capabilities for shell access)
        let cmd_lower = command.to_lowercase();
        // Check exact match or ends_with (for /bin/sh)
        if Self::BLOCKED_SHELLS.iter().any(|&s| {
            cmd_lower == s
                || cmd_lower.ends_with(&format!("/{}", s))
                || cmd_lower.ends_with(&format!("\\{}", s))
        }) {
            return Err(IfaError::Custom(format!(
                "Direct shell execution is blocked for security: {}. Use dedicated shell capabilities if needed.",
                command
            )));
        }

        // Block shell metacharacters in command name
        if command
            .chars()
            .any(|c| Self::SHELL_METACHARACTERS.contains(&c))
        {
            return Err(IfaError::Custom(format!(
                "Command contains dangerous characters: {}",
                command
            )));
        }

        // Block path traversal
        if command.contains("..") {
            return Err(IfaError::Custom(
                "Path traversal not allowed in command".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate arguments are safe
    fn validate_args(args: &[&str]) -> IfaResult<()> {
        for arg in args {
            // Block shell metacharacters that could enable injection
            if arg.chars().any(|c| Self::SHELL_METACHARACTERS.contains(&c)) {
                return Err(IfaError::Custom(format!(
                    "Argument contains dangerous shell metacharacters: {}",
                    arg
                )));
            }
        }
        Ok(())
    }

    /// Run command and get output (ṣiṣẹ́)
    ///
    /// # Security
    /// Commands and arguments are validated to prevent shell injection.
    pub fn sise(&self, command: &str, args: &[&str]) -> IfaResult<Output> {
        Self::validate_command(command)?;
        Self::validate_args(args)?;

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
        Self::validate_command(command)?;
        Self::validate_args(args)?;

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
    #[test]
    fn test_map_ops() {
        use std::collections::HashMap;
        let ogunda = Ogunda;
        let mut map = HashMap::new();
        map.insert("a".into(), IfaValue::Int(1));
        map.insert("b".into(), IfaValue::Int(2));
        let mut val = IfaValue::Map(map.into());

        // Keys
        let keys = ogunda.keys(&val).unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));

        // Values
        let values = ogunda.values(&val).unwrap();
        assert_eq!(values.len(), 2);

        // Items
        let items = ogunda.items(&val).unwrap();
        assert_eq!(items.len(), 2);

        // Remove
        let removed = ogunda.remove(&mut val, "a").unwrap();
        assert_eq!(removed, IfaValue::Int(1));
        let keys_after = ogunda.keys(&val).unwrap();
        assert_eq!(keys_after.len(), 1);
        assert_eq!(keys_after[0], "b");
    }
}
