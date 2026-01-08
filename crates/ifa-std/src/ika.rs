//! # Ìká Domain (0100)
//! 
//! The Controller - String Operations
//! 
//! Efficient string manipulation using ropey and regex.

use ropey::Rope;
use regex::Regex;
use ifa_core::error::{IfaError, IfaResult};
use crate::impl_odu_domain;

/// Ìká - The Controller (Strings)
pub struct Ika;

impl_odu_domain!(Ika, "Ìká", "0100", "The Controller - Strings");

impl Ika {
    /// Concatenate strings (sọ̀pọ̀)
    pub fn so(&self, parts: &[&str]) -> String {
        parts.join("")
    }
    
    /// Get string length in characters (gígùn)
    pub fn gigun(&self, s: &str) -> usize {
        s.chars().count()
    }
    
    /// To uppercase (nlá)
    pub fn nla(&self, s: &str) -> String {
        s.to_uppercase()
    }
    
    /// To lowercase (kékeré)
    pub fn kekere(&self, s: &str) -> String {
        s.to_lowercase()
    }
    
    /// Find substring position (wá)
    pub fn wa(&self, haystack: &str, needle: &str) -> Option<usize> {
        haystack.find(needle)
    }
    
    /// Check if contains (ní)
    pub fn ni(&self, haystack: &str, needle: &str) -> bool {
        haystack.contains(needle)
    }
    
    /// Split string (pín)
    pub fn pin(&self, s: &str, delimiter: &str) -> Vec<String> {
        s.split(delimiter).map(String::from).collect()
    }
    
    /// Join strings (dàpọ̀)
    pub fn dapo(&self, parts: &[&str], separator: &str) -> String {
        parts.join(separator)
    }
    
    /// Replace occurrences (yí padà)
    pub fn yi_pada(&self, s: &str, from: &str, to: &str) -> String {
        s.replace(from, to)
    }
    
    /// Trim whitespace (gé)
    pub fn ge(&self, s: &str) -> String {
        s.trim().to_string()
    }
    
    /// Reverse string (padà)
    pub fn pada(&self, s: &str) -> String {
        s.chars().rev().collect()
    }
    
    /// Substring slice (gé_lára)
    pub fn ge_lara(&self, s: &str, start: usize, end: usize) -> String {
        s.chars().skip(start).take(end - start).collect()
    }
    
    /// Repeat string (tún)
    pub fn tun(&self, s: &str, n: usize) -> String {
        s.repeat(n)
    }
    
    /// Check if starts with (bẹ̀rẹ̀)
    pub fn bere(&self, s: &str, prefix: &str) -> bool {
        s.starts_with(prefix)
    }
    
    /// Check if ends with (parí)
    pub fn pari(&self, s: &str, suffix: &str) -> bool {
        s.ends_with(suffix)
    }
    
    // =========================================================================
    // REGEX OPERATIONS
    // =========================================================================
    
    /// Regex match (bá mu)
    pub fn ba_mu(&self, pattern: &str, text: &str) -> IfaResult<bool> {
        match Regex::new(pattern) {
            Ok(re) => Ok(re.is_match(text)),
            Err(e) => Err(IfaError::Custom(format!("Invalid regex: {}", e))),
        }
    }
    
    /// Regex find first match (wá_àkọ́kọ́)
    pub fn wa_akoko(&self, pattern: &str, text: &str) -> IfaResult<Option<String>> {
        match Regex::new(pattern) {
            Ok(re) => Ok(re.find(text).map(|m| m.as_str().to_string())),
            Err(e) => Err(IfaError::Custom(format!("Invalid regex: {}", e))),
        }
    }
    
    /// Regex find all matches (wá_gbogbo)
    pub fn wa_gbogbo(&self, pattern: &str, text: &str) -> IfaResult<Vec<String>> {
        match Regex::new(pattern) {
            Ok(re) => Ok(re.find_iter(text).map(|m| m.as_str().to_string()).collect()),
            Err(e) => Err(IfaError::Custom(format!("Invalid regex: {}", e))),
        }
    }
    
    /// Regex replace (rọ́pò)
    pub fn ropo(&self, pattern: &str, text: &str, replacement: &str) -> IfaResult<String> {
        match Regex::new(pattern) {
            Ok(re) => Ok(re.replace_all(text, replacement).to_string()),
            Err(e) => Err(IfaError::Custom(format!("Invalid regex: {}", e))),
        }
    }
    
    // =========================================================================
    // ROPE OPERATIONS (for large text editing)
    // =========================================================================
    
    /// Create rope from string
    pub fn rope_new(&self, s: &str) -> Rope {
        Rope::from_str(s)
    }
    
    /// Insert into rope at character index
    pub fn rope_insert(&self, rope: &mut Rope, idx: usize, text: &str) {
        rope.insert(idx, text);
    }
    
    /// Delete from rope (character range)
    pub fn rope_delete(&self, rope: &mut Rope, start: usize, end: usize) {
        rope.remove(start..end);
    }
    
    /// Get rope slice
    pub fn rope_slice(&self, rope: &Rope, start: usize, end: usize) -> String {
        rope.slice(start..end).to_string()
    }
    
    /// Get rope length in characters
    pub fn rope_len(&self, rope: &Rope) -> usize {
        rope.len_chars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_ops() {
        let ika = Ika;
        assert_eq!(ika.so(&["Hello", " ", "World"]), "Hello World");
        assert_eq!(ika.gigun("Ifá"), 3);
        assert_eq!(ika.nla("hello"), "HELLO");
        assert_eq!(ika.pada("abc"), "cba");
    }
    
    #[test]
    fn test_regex() {
        let ika = Ika;
        assert!(ika.ba_mu(r"\d+", "abc123").unwrap());
        assert_eq!(ika.wa_akoko(r"\d+", "abc123xyz").unwrap(), Some("123".to_string()));
    }
}
