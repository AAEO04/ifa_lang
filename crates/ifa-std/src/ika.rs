//! # Ìká Domain (0100)
//!
//! The Controller - String Operations
//!
//! Efficient string manipulation using ropey and regex.

use crate::impl_odu_domain;
use ifa_core::{
    error::{IfaError, IfaResult},
    value::IfaValue,
};
use regex::Regex;
use ropey::Rope;

/// Ìká - The Controller (Strings & Serialization)
pub struct Ika;

impl_odu_domain!(
    Ika,
    "Ìká",
    "0100",
    "The Controller - Strings & Serialization"
);

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
    // SERIALIZATION (Native Speed)
    // =========================================================================

    /// Serialize to JSON (yi_si_json)
    pub fn yi_si_json(&self, val: &IfaValue) -> IfaResult<String> {
        serde_json::to_string(val).map_err(|e| IfaError::Custom(format!("JSON error: {}", e)))
    }

    /// Deserialize from JSON (yi_padà_json)
    pub fn yi_pada_json(&self, json: &str) -> IfaResult<IfaValue> {
        serde_json::from_str(json).map_err(|e| IfaError::Custom(format!("JSON error: {}", e)))
    }

    /// URL Encode (bo_asiri_url)
    pub fn bo_asiri_url(&self, text: &str) -> String {
        urlencoding::encode(text).to_string()
    }

    /// URL Decode (titu_asiri_url)
    pub fn titu_asiri_url(&self, text: &str) -> IfaResult<String> {
        urlencoding::decode(text)
            .map(|s| s.to_string())
            .map_err(|e| IfaError::Custom(format!("URL decode error: {}", e)))
    }

    /// Serialize Matrix to CSV (yi_si_csv)
    /// Expects List<List<String>> or List<List<Int/Float>>
    pub fn yi_si_csv(&self, rows: &IfaValue) -> IfaResult<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        if let IfaValue::List(list) = rows {
            for row in list {
                if let IfaValue::List(cells) = row {
                    let record: Vec<String> = cells.iter().map(|c| c.to_string()).collect();
                    wtr.write_record(&record)
                        .map_err(|e| IfaError::Custom(format!("CSV error: {}", e)))?;
                }
            }
        }

        let data = wtr
            .into_inner()
            .map_err(|e| IfaError::Custom(format!("CSV flush error: {}", e)))?;
        String::from_utf8(data).map_err(|e| IfaError::Custom(format!("UTF-8 error: {}", e)))
    }

    /// Deserialize CSV to Matrix (yi_pada_csv)
    pub fn yi_pada_csv(&self, data: &str, has_headers: bool) -> IfaResult<IfaValue> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .from_reader(data.as_bytes());

        let mut rows = Vec::new();

        for result in rdr.records() {
            let record = result.map_err(|e| IfaError::Custom(format!("CSV parse error: {}", e)))?;
            let row: Vec<IfaValue> = record
                .iter()
                .map(|s| IfaValue::Str(s.to_string()))
                .collect();
            rows.push(IfaValue::List(row));
        }

        Ok(IfaValue::List(rows))
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

    // --- English Aliases ---

    pub fn len(&self, s: &str) -> usize {
        self.gigun(s)
    }
    pub fn find(&self, haystack: &str, needle: &str) -> Option<usize> {
        self.wa(haystack, needle)
    }
    pub fn has(&self, haystack: &str, needle: &str) -> bool {
        self.ni(haystack, needle)
    }
    pub fn split(&self, s: &str, delimiter: &str) -> Vec<String> {
        self.pin(s, delimiter)
    }
    pub fn join(&self, parts: &[&str], separator: &str) -> String {
        self.dapo(parts, separator)
    }
    pub fn replace(&self, s: &str, from: &str, to: &str) -> String {
        self.yi_pada(s, from, to)
    }
    pub fn matches(&self, pattern: &str, text: &str) -> IfaResult<bool> {
        self.ba_mu(pattern, text)
    }
    pub fn encode(&self, val: &IfaValue) -> IfaResult<String> {
        self.yi_si_json(val)
    }
    pub fn decode(&self, json: &str) -> IfaResult<IfaValue> {
        self.yi_pada_json(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
        assert_eq!(
            ika.wa_akoko(r"\d+", "abc123xyz").unwrap(),
            Some("123".to_string())
        );
    }

    #[test]
    fn test_json_serialization() {
        let ika = Ika;
        let mut map = HashMap::new();
        map.insert("key".to_string(), IfaValue::Str("value".to_string()));
        map.insert("number".to_string(), IfaValue::Int(42));
        let val = IfaValue::Map(map);

        let json = ika.yi_si_json(&val).unwrap();
        // Skip brittle string containment checks (order/spacing varies)
        // Rely on round-trip deserialization below

        let deserialized = ika.yi_pada_json(&json).unwrap();
        if let IfaValue::Map(m) = deserialized {
            assert_eq!(m.get("key"), Some(&IfaValue::Str("value".to_string())));
            assert_eq!(m.get("number"), Some(&IfaValue::Int(42)));
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_csv_serialization() {
        let ika = Ika;
        let rows = IfaValue::List(vec![
            IfaValue::List(vec![
                IfaValue::Str("Name".to_string()),
                IfaValue::Str("Age".to_string()),
            ]),
            IfaValue::List(vec![IfaValue::Str("Alice".to_string()), IfaValue::Int(30)]),
            IfaValue::List(vec![IfaValue::Str("Bob".to_string()), IfaValue::Int(25)]),
        ]);

        let csv = ika.yi_si_csv(&rows).unwrap();
        assert!(csv.contains("Name,Age"));
        assert!(csv.contains("Alice,30"));
        assert!(csv.contains("Bob,25"));

        // Test round trip
        let parsed = ika.yi_pada_csv(&csv, false).unwrap(); // false = treat headers as data
        if let IfaValue::List(l) = parsed {
            assert_eq!(l.len(), 3);
            if let IfaValue::List(r0) = &l[0] {
                assert_eq!(r0[0], IfaValue::Str("Name".to_string()));
            }
        }
    }

    #[test]
    fn test_url_encoding() {
        let ika = Ika;
        let text = "Hello World & Ifá";
        let encoded = ika.bo_asiri_url(text);
        assert_eq!(encoded, "Hello%20World%20%26%20If%C3%A1");

        let decoded = ika.titu_asiri_url(&encoded).unwrap();
        assert_eq!(decoded, text);
    }
}
