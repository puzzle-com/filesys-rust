extern crate serde_cbor;

use std::collections::HashMap;
use serde_cbor::{Value, ObjectKey};

pub trait IPLD {
    /// The type of an IPLD object
    type Object;

    /// Type for keys of an IPLD object
    type ObjectKey;

    /// Type for values of an IPLD object
    type Value;

    /// Representation of an IPLD path, e.g. /my/val
    type Path;

    /// Given any value, and a path resolve the path and return the
    /// value at the end.
    fn cat<'a>(&self, &'a Self::Value, Self::Path) -> &'a Self::Value;
}

pub struct CborIpld;

impl IPLD for CborIpld {
    type Object = HashMap<ObjectKey, Value>;
    type ObjectKey = serde_cbor::ObjectKey;
    type Value = serde_cbor::Value;

    type Path = Vec<ObjectKey>;

    fn cat<'a>(&self, obj: &'a Value, path: Vec<ObjectKey>) -> &'a Value {
        path.iter().fold(obj, |acc, x| {
            match *acc {
                Value::Array(ref vec) => {
                    match *x {
                        ObjectKey::Integer(i) => &vec[i as usize],
                        _ => panic!("Can not access array"),
                    }
                }
                Value::Object(ref map) => map.get(x).unwrap(),
                Value::U64(_)   |
                Value::I64(_)   |
                Value::Bytes(_) |
                Value::String(_)|
                Value::F64(_)   |
                Value::Bool(_)  |
                Value::Null     => acc,
            }
        })
    }
}

pub trait ToIPLD<X: IPLD> {
    fn to_ipld(&self) -> <X as IPLD>::Value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use serde_cbor::{Value, ObjectKey};

    #[derive(Hash, Eq, PartialEq, Debug)]
    struct File {
        data: String,
        size: usize,
    }

    impl ToIPLD<CborIpld> for File {
        fn to_ipld(&self) -> <CborIpld as IPLD>::Value {
            let mut map = HashMap::new();
            map.insert(ObjectKey::String("data".to_string()),
                       Value::String(((*self.data).to_string())));
            map.insert(ObjectKey::String("size".to_string()),
                       Value::U64(self.size as u64));

            Value::Object(map)
        }
    }

    #[test]
    fn test_cat_file() {
        let file = File {
            data: "hello world".to_string(),
            size: 11,
        };

        let cbor_ipld = CborIpld;
        let file_val = file.to_ipld();
        let result = cbor_ipld.cat(&file_val, vec![ObjectKey::String("data".to_string())]);

        let val = match result {
            &Value::String(ref val) => val,
            _ => panic!("Wrong value"),
        };

        assert_eq!(val, "hello world");
    }

    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    struct Chunk {
        link: String,
        size: usize,
    }

    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    struct ChunkedFile {
        size: usize,
        subfiles: Vec<Chunk>,
    }

    impl ToIPLD<CborIpld> for Chunk {
        fn to_ipld(&self) -> <CborIpld as IPLD>::Value {
            let mut map = HashMap::new();
            map.insert(ObjectKey::String("link".to_string()),
                       Value::String(((*self.link).to_string())));
            map.insert(ObjectKey::String("size".to_string()),
                       Value::U64(self.size as u64));

            Value::Object(map)
        }
    }

    impl ToIPLD<CborIpld> for ChunkedFile {
        fn to_ipld(&self) -> <CborIpld as IPLD>::Value {
            let mut map = HashMap::new();
            map.insert(ObjectKey::String("size".to_string()),
                       Value::U64(self.size as u64));
            map.insert(ObjectKey::String("subfiles".to_string()),
                       Value::Array(self.subfiles
                           .iter()
                           .cloned()
                           .map(|x| x.to_ipld())
                           .collect()));

            Value::Object(map)
        }
    }

    #[test]
    fn test_cat_chunked_file() {
        let file = ChunkedFile {
            size: 1424119,
            subfiles: vec![
                Chunk {
                    link: "QmAAA".to_string(),
                    size: 100324
                },
                Chunk {
                    link: "QmBBB".to_string(),
                    size: 120345
                }
            ]
        };

        let cbor_ipld = CborIpld;
        let file_val = file.to_ipld();
        let result = cbor_ipld.cat(&file_val,
                                   vec![ObjectKey::String("subfiles".to_string()),
                                        ObjectKey::Integer(1),
                                        ObjectKey::String("link".to_string())]);

        let val = match result {
            &Value::String(ref val) => val,
            _ => panic!("Wrong value"),
        };

        assert_eq!(val, "QmBBB");
    }
}
