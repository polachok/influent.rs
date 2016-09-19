use std::collections::BTreeMap;
use std::borrow::Borrow;

#[derive(Debug)]
/// Measurement's field value.
pub enum Value<S: Borrow<str>> {
    /// String.
    String(S),
    /// Floating point number.
    Float(f64),
    /// Integer number.
    Integer(i64),
    /// Boolean value.
    Boolean(bool)
}

/// Measurement model.
#[derive(Debug)]
pub struct Measurement<'a,S: Borrow<str>> {
    /// Key.
    pub key: &'a str,

    /// Timestamp.
    pub timestamp: Option<i64>,

    /// Map of fields.
    pub fields: BTreeMap<&'a str, Value<S>>,
    
    /// Map of tags.
    pub tags: BTreeMap<&'a str, S>
}

impl<'a,S> Measurement<'a,S> where S: Borrow<str> {
    /// Constructs a new `Measurement`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use influent::measurement::Measurement;
    ///
    /// let measurement = Measurement::new("key");
    /// ```
    pub fn new(key: &'a str) -> Self {
        Measurement {
            key: key,
            timestamp: None,
            fields: BTreeMap::new(),
            tags: BTreeMap::new()
        }
    }

    /// Adds field to the measurement.
    ///
    /// # Examples
    ///
    /// ```
    /// use influent::measurement::{Measurement, Value};
    ///
    /// let mut measurement = Measurement::new("key");
    ///
    /// measurement.add_field("field", Value::String("hello"));
    /// ```
    pub fn add_field(&mut self, field: &'a str, value: Value<S>) {
        self.fields.insert(field, value);
    }

    /// Adds tag to the measurement.
    ///
    /// # Examples
    ///
    /// ```
    /// use influent::measurement::{Measurement, Value};
    ///
    /// let mut measurement = Measurement::new("key");
    ///
    /// measurement.add_tag("tag", "value");
    /// ```
    pub fn add_tag(&mut self, tag: &'a str, value: S) {
        self.tags.insert(tag, value);
    }

    /// Sets the timestamp of the measurement. It should be unix timestamp in nanosecond
    ///
    /// # Examples
    ///
    /// ```
    /// use influent::measurement::{Measurement, Value};
    ///
    /// let mut measurement = Measurement::new("key");
    ///
    /// measurement.set_timestamp(1434055562000000000)
    /// ```
    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = Some(timestamp);
    }
}
