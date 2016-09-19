use ::measurement::Measurement;
use std::borrow::Borrow;

pub mod line;

/// `Measurement` serializer.
pub trait Serializer<S: Borrow<str>> {
    /// Serializes measurement to String.
    fn serialize(&self, measurement: &Measurement<S>) -> String;
}
