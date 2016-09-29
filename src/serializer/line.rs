#![feature(test)]

extern crate test;
use ::measurement::{Measurement, Value};
use ::serializer::Serializer;
use std::io::{self,Cursor,Write};
use std::borrow::Borrow;
use std::fmt;

pub struct LineSerializer;

/// Line spec `Measurement` serializer.
impl LineSerializer {
    /// Constructs new `LineSerializer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use influent::serializer::Serializer;
    /// use influent::serializer::line::LineSerializer;
    /// use influent::measurement::{Measurement, Value};
    ///
    /// let serializer = LineSerializer::new();
    /// let mut measurement = Measurement::new("key");
    ///
    /// measurement.add_field("field", Value::String("value"));
    /// measurement.add_tag("tag", "value");
    ///
    /// assert_eq!("key,tag=value field=\"value\"", serializer.serialize(&measurement));
    /// ```
    pub fn new() -> LineSerializer {
        LineSerializer
    }

    // Measurement names must escape commas and spaces.
    fn write_escaped_key(w: &mut Write, key: &str) -> io::Result<usize> {
        let mut written = 0;
        for byte in key.as_bytes() {
            written += match *byte {
                b',' => try!(w.write(b"\\,")),
                b' ' => try!(w.write(b"\\ ")),
                _ => try!(w.write(&[*byte])),
            }
        }
        Ok(written)
    }

    // Tag keys and tag values must escape commas, spaces, and equal signs. 
    fn write_escaped_tag(w: &mut Write, tag: &str) -> io::Result<usize> {
        let mut written = 0;
        for byte in tag.as_bytes() {
            written += match *byte {
                b',' => try!(w.write(b"\\,")),
                b' ' => try!(w.write(b"\\ ")),
                b'=' => try!(w.write(b"\\ ")),
                _ => try!(w.write(&[*byte])),
            }
        }
        Ok(written)
    }

    fn write_escaped_value<S: Borrow<str>>(w: &mut Write, value: &Value<S>) -> io::Result<usize> {
        let mut written = 0;
        match value {
            // Strings are text values. All string values must be
            // surrounded in double-quotes ".
            // If the string contains a double-quote,
            // it must be escaped with a backslash, e.g. \". 
            &Value::String(ref s)  => {
                written += try!(w.write(&[b'"']));
                for byte in s.borrow().as_bytes() {
                    if *byte == b'"' {
                        written += try!(w.write(b"\\\""));
                    } else {
                        written += try!(w.write(&[*byte]));
                    }
                }
                try!(w.write(b"\""));
            },
            // Integers are numeric values that do not include a decimal
            // and are followed by a trailing i when inserted
            &Value::Integer(ref i) => {
                written += try!(w.write(i.to_string().as_bytes()));
                written += try!(w.write(b"i"));
            },
            &Value::Float(ref f) => {
                written += try!(w.write(f.to_string().as_bytes()));
            }, 
            &Value::Boolean(ref b) => {
                written += try!(w.write(if *b { b"t" } else { b"f" }));
            },
        };
        Ok(written)
    }

    fn serialize_buf<S: Borrow<str>>(&self, measurement: &Measurement<S>) -> Vec<u8> {
        use std::io::Cursor;
        let mut buf = Vec::new();
        {
            let mut cur = Cursor::new(buf);
            Self::write_escaped_key(&mut cur, measurement.key.borrow());
            for (tag, value) in measurement.tags.iter() {
                cur.write(b",");
                Self::write_escaped_tag(&mut cur, tag);
                cur.write(b"=");
                Self::write_escaped_tag(&mut cur, value.borrow());
            }
           
            let mut first = true;
            for (field, value) in measurement.fields.iter() {
                if first { first = false; cur.write(b" ") } else { cur.write(b",") };
                Self::write_escaped_tag(&mut cur, field.borrow());
                cur.write(b"=");
                Self::write_escaped_value(&mut cur, value);
            }

            if let Some(ts) = measurement.timestamp {
                cur.write(b" ");
                cur.write(ts.to_string().as_bytes());
            }

            cur.into_inner()
        }
    }
}

fn escape(s: &str) -> String {
    s
        .replace(" ", "\\ ")
        .replace(",", "\\,")
}

fn as_string(s: &str) -> String {
    format!("\"{}\"", s.replace("\"", "\\\""))
}

fn as_integer(i: &i64) -> String {
    format!("{}i", i)
}

fn as_float(f: &f64) -> String {
    f.to_string()
}

fn as_boolean(b: &bool) -> String {
    if *b { "t".to_string() } else { "f".to_string() }
}

impl<S: Borrow<str>> Serializer<S> for LineSerializer {
    fn serialize(&self, measurement: &Measurement<S>) -> String {
        let v = self.serialize_buf(measurement);
        String::from_utf8(v).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{as_boolean, as_string, as_integer, as_float, escape, LineSerializer, test};
    use ::serializer::Serializer;
    use ::measurement::{Measurement, Value};

    #[test]
    fn test_as_boolean() {
        assert_eq!("t", as_boolean(&true));
        assert_eq!("f", as_boolean(&false));
    }

    #[test]
    fn test_as_string() {
        assert_eq!("\"\\\"hello\\\"\"", as_string(&"\"hello\""));
    }

    #[test]
    fn test_as_integer() {
        assert_eq!("1i",    as_integer(&1i64));
        assert_eq!("345i",  as_integer(&345i64));
        assert_eq!("2015i", as_integer(&2015i64));
        assert_eq!("-10i",  as_integer(&-10i64));
    }

    #[test]
    fn test_as_float() {
        assert_eq!("1", as_float(&1f64));
        assert_eq!("1", as_float(&1.0f64));
        assert_eq!("-3.14", as_float(&-3.14f64));
        assert_eq!("10", as_float(&10f64));
    }

    #[test]
    fn test_escape() {
        assert_eq!("\\ ", escape(" "));
        assert_eq!("\\,", escape(","));
        assert_eq!("hello\\,\\ gobwas", escape("hello, gobwas"));
    }


    #[test]
    fn test_line_serializer_2() {
        let serializer = LineSerializer::new();
        let mut measurement = Measurement::new("key");

        measurement.add_field("s", Value::String("string"));
        measurement.add_field("i", Value::Integer(10));
        measurement.add_field("f", Value::Float(10f64));
        measurement.add_field("b", Value::Boolean(false));

        measurement.add_tag("tag", "value");
        
        measurement.add_field("one, two", Value::String("three"));
        measurement.add_tag("one ,two", "three, four");


        measurement.set_timestamp(10);

        let shit = String::from_utf8(serializer.serialize_buf(&measurement)).unwrap();
        assert_eq!("key,one\\ \\,two=three\\,\\ four,tag=value b=f,f=10,i=10i,one\\,\\ two=\"three\",s=\"string\" 10", shit);
    }

    fn do_serialize() {
        let serializer = LineSerializer::new();
        let mut measurement = Measurement::new("key");

        measurement.add_field("s", Value::String("string"));
        measurement.add_field("i", Value::Integer(10));
        measurement.add_field("f", Value::Float(10f64));
        measurement.add_field("b", Value::Boolean(false));

        measurement.add_tag("tag", "value");
        
        measurement.add_field("one, two", Value::String("three"));
        measurement.add_tag("one ,two", "three, four");


        measurement.set_timestamp(10);

        assert_eq!("key,one\\ \\,two=three\\,\\ four,tag=value b=f,f=10,i=10i,one\\,\\ two=\"three\",s=\"string\" 10", serializer.serialize(&measurement));
    }

    fn do_serialize_buf() {
        let serializer = LineSerializer::new();
        let mut measurement = Measurement::new("key");

        measurement.add_field("s", Value::String("string"));
        measurement.add_field("i", Value::Integer(10));
        measurement.add_field("f", Value::Float(10f64));
        measurement.add_field("b", Value::Boolean(false));

        measurement.add_tag("tag", "value");
        
        measurement.add_field("one, two", Value::String("three"));
        measurement.add_tag("one ,two", "three, four");


        measurement.set_timestamp(10);

        let shit = String::from_utf8(serializer.serialize_buf(&measurement)).unwrap();
        assert_eq!("key,one\\ \\,two=three\\,\\ four,tag=value b=f,f=10,i=10i,one\\,\\ two=\"three\",s=\"string\" 10", shit);
    }

    #[bench]
    fn bench_serialize(b: &mut test::Bencher) {
        b.iter(|| do_serialize())
    }

    #[bench]
    fn bench_my_serialize(b: &mut test::Bencher) {
        b.iter(|| do_serialize_buf())
    }

    #[test]
    fn test_line_serializer() {
        let serializer = LineSerializer::new();
        let mut measurement = Measurement::new("key");

        measurement.add_field("s", Value::String("string"));
        measurement.add_field("i", Value::Integer(10));
        measurement.add_field("f", Value::Float(10f64));
        measurement.add_field("b", Value::Boolean(false));

        measurement.add_tag("tag", "value");
        
        measurement.add_field("one, two", Value::String("three"));
        measurement.add_tag("one ,two", "three, four");


        measurement.set_timestamp(10);

        assert_eq!("key,one\\ \\,two=three\\,\\ four,tag=value b=f,f=10,i=10i,one\\,\\ two=\"three\",s=\"string\" 10", serializer.serialize(&measurement));
    }

    #[test]
    fn test_line_serializer_long_timestamp() {
        let serializer = LineSerializer::new();
        let mut measurement = Measurement::new("key");

        measurement.add_field("s", Value::String("string"));

        measurement.set_timestamp(1434055562000000000);

        assert_eq!("key s=\"string\" 1434055562000000000", serializer.serialize(&measurement));
    }
}




