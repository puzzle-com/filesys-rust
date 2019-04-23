//! CBOR and serialization.
//!
//! # What is CBOR?
//! [CBOR](http://cbor.io) is a way to encode data in a space-efficient and fast binary format.
//! CBORs data-model is a superset of the JSONs.
//!
//! A simple object describing a person in diagnostic notation (it is actually JSON plus some
//! annotations) looks like
//!
//! ```json
//! {
//!     "FirstName": "John",
//!     "LastName": "Doe",
//!     "Age": 43,
//!     "Address": {
//!         "Street": "Downing Street 10",
//!         "City": "London",
//!         "Country": "Great Britain"
//!     },
//!     "PhoneNumbers": [
//!         "+44 1234567",
//!         "+44 2345678"
//!     ]
//! }
//! ```
//!
//! The CBOR encoded object with comments in hexadecimal notation looks like
//!
//! ```cbor
//! a5                                      # map(5)
//!    69                                   # text(9)
//!       46697273744e616d65                # "FirstName"
//!    64                                   # text(4)
//!       4a6f686e                          # "John"
//!    68                                   # text(8)
//!       4c6173744e616d65                  # "LastName"
//!    63                                   # text(3)
//!       446f65                            # "Doe"
//!    63                                   # text(3)
//!       416765                            # "Age"
//!    18 2b                                # unsigned(43)
//!    67                                   # text(7)
//!       41646472657373                    # "Address"
//!    a3                                   # map(3)
//!       66                                # text(6)
//!          537472656574                   # "Street"
//!       71                                # text(17)
//!          446f776e696e6720537472656574203130 # "Downing Street 10"
//!       64                                # text(4)
//!          43697479                       # "City"
//!       66                                # text(6)
//!          4c6f6e646f6e                   # "London"
//!       67                                # text(7)
//!          436f756e747279                 # "Country"
//!       6d                                # text(13)
//!          4772656174204272697461696e     # "Great Britain"
//!    6c                                   # text(12)
//!       50686f6e654e756d62657273          # "PhoneNumbers"
//!    82                                   # array(2)
//!       6b                                # text(11)
//!          2b34342031323334353637         # "+44 1234567"
//!       6b                                # text(11)
//!          2b34342032333435363738         # "+44 2345678"
//! ```
//! While the JSON encoding is 174 bytes long the CBOR representation is only 141 bytes long.
//! This is 19% shorter! Sometimes compression will even better, but never CBOR will be longer
//! than the corresponding JSON. More importantly CBOR supports binary data, custom data tyes,
//! annotations for dates, times and expected encoding and is faster to serialize and deserialize.
//! It can even be used on embedded devices.
//!
//! # Type-based Serialization and Deserialization
//! Serde provides a mechanism for low boilerplate serialization & deserialization of values to and
//! from CBOR via the serialization API. To be able to serialize a piece of data, it must implement
//! the `serde::Serialize` trait. To be able to deserialize a piece of data, it must implement the
//! `serde::Deserialize` trait. Serde provides an annotation to automatically generate the
//! code for these traits: `#[derive(Serialize, Deserialize)]`.
//!
//! The CBOR API also provides an enum `serde_cbor::Value`.
//!
//! # Packed Encoding
//! When serializing structs or enums in CBOR the keys or enum variant names will be serialized
//! as string keys to a map. Especially in embedded environments this can increase the file
//! size too much. In packed encoding the keys and variants will be serialized as variable sized
//! integers. The first 24 entries in any struct consume only a single byte!
//! To serialize a document in packed encoding use `ser::to_(vec|writer)_packed`, deserialization
//! works without any changes.
//!
//! # Self describing documents
//! In some contexts different formats are used but there is no way to declare the format used
//! out of band. For this reason CBOR has a magic number that may be added before any document.
//! The *`_sd` (for *s*elf*d*escribe) append the magic number before documents.
//!
//! # Examples
//! Read a CBOR value that is known to be a map of string keys to string values and print it.
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use serde_cbor::from_slice;
//!
//! let slice = b"\xa5aaaAabaBacaCadaDaeaE";
//! let value: BTreeMap<String, String> = from_slice(slice).unwrap();
//! println!("{:?}", value); // {"e": "E", "d": "D", "a": "A", "c": "C", "b": "B"}
//! ```
//!
//! Read a general CBOR value with an unknown content.
//!
//! ```rust
//! use serde_cbor::{from_slice, Value};
//!
//! let slice = b"\x82\x01\xa1aaab";
//! let value: Value = from_slice(slice).unwrap();
//! println!("{:?}", value); // Array([U64(1), Object({String("a"): String("b")})])
//! ```
//!
//! Serialize an object.
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use serde_cbor::to_vec;
//!
//! let mut programming_languages = BTreeMap::new();
//! programming_languages.insert("rust", vec!["safe", "concurrent", "fast"]);
//! programming_languages.insert("python", vec!["powerful", "friendly", "open"]);
//! programming_languages.insert("js", vec!["lightweight", "interpreted", "object-oriented"]);
//! let encoded = to_vec(&programming_languages);
//! assert_eq!(encoded.unwrap().len(), 103);
//! ```
//!
//! Serializing a `Vec` as a specialized byte string uses about 2x less RAM and
//! 100x less CPU time than serializing it as an array.
//!
//! ```rust
//! # extern crate serde_bytes;
//! # extern crate serde_cbor;
//! use std::collections::BTreeMap;
//! use serde_bytes::ByteBuf;
//! use serde_cbor::to_vec;
//!
//! # fn main() {
//! let data: Vec<u8> = vec![0, 1, 255];
//! let serialized_array = to_vec(&data).unwrap();
//! let byte_buf = ByteBuf::from(data);
//! let serialized_byte_string = to_vec(&byte_buf).unwrap();
//! assert!(serialized_byte_string.len() < serialized_array.len());
//! # }
//! ```
//!
//! Deserializing data in the middle of a slice
//! ```
//! # extern crate serde_cbor;
//! use serde_cbor::Deserializer;
//!
//! # fn main() {
//! let data: Vec<u8> = vec![
//!     0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x66, 0x66, 0x6f, 0x6f, 0x62,
//!     0x61, 0x72,
//! ];
//! let mut deserializer = Deserializer::from_slice(&data);
//! let value: &str = serde::de::Deserialize::deserialize(&mut deserializer)
//!     .unwrap();
//! let rest = &data[deserializer.byte_offset()..];
//! assert_eq!(value, "foobar");
//! assert_eq!(rest, &[0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72]);
//! # }
//! ```
//!
//! # Limitations
//!
//! While Serde CBOR strives to support all features of Serde and CBOR
//! there are a few limitations.
//!
//! * [Tags] are ignored during deserialization and can't be emitted during
//!     serialization. This is because Serde has no concept of tagged
//!     values. See:&nbsp;[#3]
//! * Unknown [simple values] cause an `UnassignedCode` error.
//!     The simple values *False* and *True* are recognized and parsed as bool.
//!     *Null* and *Undefined* are both deserialized as *unit*.
//!     The *unit* type is serialized as *Null*. See:&nbsp;[#86]
//! * [128-bit integers] can't be directly encoded in CBOR. If you need them
//!     store them as a byte string. See:&nbsp;[#77]
//!
//! [Tags]: https://tools.ietf.org/html/rfc7049#section-2.4.4
//! [#3]: https://github.com/pyfisch/cbor/issues/3
//! [simple values]: https://tools.ietf.org/html/rfc7049#section-3.5
//! [#86]: https://github.com/pyfisch/cbor/issues/86
//! [128-bit integers]: https://doc.rust-lang.org/std/primitive.u128.html
//! [#77]: https://github.com/pyfisch/cbor/issues/77

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

// When we are running tests in no_std mode we need to explicitly link std, because `cargo test`
// will not work without it.
#[cfg(all(not(feature = "std"), test))]
extern crate std;

pub mod de;
pub mod error;
mod read;
pub mod ser;
mod write;

#[cfg(feature = "std")]
pub mod value;

#[doc(inline)]
pub use crate::de::{from_mut_slice, from_slice_with_scratch, Deserializer, StreamDeserializer};
#[doc(inline)]
#[cfg(feature = "std")]
pub use crate::de::{from_reader, from_slice};

#[doc(inline)]
#[cfg(feature = "std")]
pub use crate::ser::{to_vec, to_vec_with_options, to_writer};
pub use crate::ser::{Serializer, SerializerOptions};
#[doc(inline)]
#[cfg(feature = "std")]
pub use crate::value::{from_value, to_value, ObjectKey, Value};
