//! # Ìwòrì Domain (0110)
//! 
//! The Viewer - Time and Iteration
//! 
//! Date/time operations using chrono with zero-cost iterators.

use chrono::{DateTime, Local, Utc, Duration, Datelike, Timelike, NaiveDate};
use crate::impl_odu_domain;

/// Ìwòrì - The Viewer (Time/Iteration)
pub struct Iwori;

impl_odu_domain!(Iwori, "Ìwòrì", "0110", "The Viewer - Time/Iteration");

impl Iwori {
    /// Get current local time (ìsisìnyí)
    pub fn isisinyi(&self) -> DateTime<Local> {
        Local::now()
    }
    
    /// Get current UTC time
    pub fn utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
    
    /// Get Unix timestamp in seconds (àkókò)
    pub fn akoko(&self) -> i64 {
        Utc::now().timestamp()
    }
    
    /// Get Unix timestamp in milliseconds
    pub fn akoko_ms(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
    
    /// Format date/time (ọjọ́)
    pub fn ojo(&self, format: &str) -> String {
        Local::now().format(format).to_string()
    }
    
    /// Parse date string
    pub fn ka_ojo(&self, date_str: &str, format: &str) -> Option<DateTime<Local>> {
        DateTime::parse_from_str(date_str, format)
            .ok()
            .map(|dt| dt.with_timezone(&Local))
    }
    
    /// Get current year
    pub fn odun(&self) -> i32 {
        Local::now().year()
    }
    
    /// Get current month (1-12)
    pub fn osu(&self) -> u32 {
        Local::now().month()
    }
    
    /// Get current day of month
    pub fn ojo_osu(&self) -> u32 {
        Local::now().day()
    }
    
    /// Get current hour (0-23)
    pub fn wakati(&self) -> u32 {
        Local::now().hour()
    }
    
    /// Get current minute
    pub fn iseju(&self) -> u32 {
        Local::now().minute()
    }
    
    /// Get current second
    pub fn aaya(&self) -> u32 {
        Local::now().second()
    }
    
    /// Get day of week (0=Sunday, 6=Saturday)
    pub fn ojo_ose(&self) -> u32 {
        Local::now().weekday().num_days_from_sunday()
    }
    
    /// Check if year is leap year
    pub fn odun_abule(&self, year: i32) -> bool {
        NaiveDate::from_ymd_opt(year, 2, 29).is_some()
    }
    
    /// Add duration to current time
    pub fn fikun(&self, days: i64, hours: i64, minutes: i64, seconds: i64) -> DateTime<Local> {
        Local::now()
            + Duration::days(days)
            + Duration::hours(hours)
            + Duration::minutes(minutes)
            + Duration::seconds(seconds)
    }
    
    /// Days between two dates
    pub fn iye_ojo(&self, from: DateTime<Local>, to: DateTime<Local>) -> i64 {
        (to - from).num_days()
    }
}

// =============================================================================
// ZERO-COST ITERATORS
// =============================================================================

/// Range iterator (inclusive)
pub fn range(start: i64, end: i64) -> impl Iterator<Item = i64> {
    start..=end
}

/// Step range iterator
pub fn range_step(start: i64, end: i64, step: i64) -> impl Iterator<Item = i64> {
    (0..).map(move |i| start + i * step).take_while(move |&x| {
        if step > 0 { x <= end } else { x >= end }
    })
}

/// Repeat value iterator
pub fn repeat<T: Clone>(value: T, count: usize) -> impl Iterator<Item = T> {
    std::iter::repeat(value).take(count)
}

/// Enumerate iterator
pub fn enumerate<I: Iterator>(iter: I) -> impl Iterator<Item = (usize, I::Item)> {
    iter.enumerate()
}

/// Zip two iterators
pub fn zip<A, B>(a: A, b: B) -> impl Iterator<Item = (A::Item, B::Item)>
where
    A: Iterator,
    B: Iterator,
{
    a.zip(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time() {
        let iwori = Iwori;
        let now = iwori.akoko();
        assert!(now > 0);
        
        let year = iwori.odun();
        assert!(year >= 2024);
    }
    
    #[test]
    fn test_range() {
        let items: Vec<i64> = range(1, 5).collect();
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn test_range_step() {
        let items: Vec<i64> = range_step(0, 10, 2).collect();
        assert_eq!(items, vec![0, 2, 4, 6, 8, 10]);
    }
    
    #[test]
    fn test_leap_year() {
        let iwori = Iwori;
        assert!(iwori.odun_abule(2024));
        assert!(!iwori.odun_abule(2023));
    }
}
