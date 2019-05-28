use bytes::Buf;
use hashing::hash;
use int_to_bytes::{int_to_bytes1, int_to_bytes4};
use std::cmp::max;
use std::io::Cursor;

/// Return `p(index)` in a pseudorandom permutation `p` of `0...list_size-1` with ``seed`` as entropy.
///
/// Utilizes 'swap or not' shuffling found in
/// https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf
/// See the 'generalized domain' algorithm on page 3.
///
/// Note: this function is significantly slower than the `shuffle_list` function in this crate.
/// Using `get_permutated_list` to shuffle an entire list, index by index, has been observed to be
/// 250x slower than `shuffle_list`. Therefore, this function is only useful when shuffling a small
/// portion of a much larger list.
///
/// Returns `None` under any of the following conditions:
///  - `list_size == 0`
///  - `index >= list_size`
///  - `list_size > 2**24`
///  - `list_size > usize::max_value() / 2`
pub fn get_permutated_index(
    index: usize,
    list_size: usize,
    seed: &[u8],
    shuffle_round_count: u8,
) -> Option<usize> {
    if list_size == 0
        || index >= list_size
        || list_size > usize::max_value() / 2
        || list_size > 2_usize.pow(24)
    {
        return None;
    }

    let mut index = index;
    for round in 0..shuffle_round_count {
        let pivot = bytes_to_int64(&hash_with_round(seed, round)[..]) as usize % list_size;
        index = do_round(seed, index, pivot, round, list_size)?;
    }
    Some(index)
}

fn do_round(seed: &[u8], index: usize, pivot: usize, round: u8, list_size: usize) -> Option<usize> {
    let flip = (pivot + list_size - index) % list_size;
    let position = max(index, flip);
    let source = hash_with_round_and_position(seed, round, position)?;
    let byte = source[(position % 256) / 8];
    let bit = (byte >> (position % 8)) % 2;
    Some(if bit == 1 { flip } else { index })
}

fn hash_with_round_and_position(seed: &[u8], round: u8, position: usize) -> Option<Vec<u8>> {
    let mut seed = seed.to_vec();
    seed.append(&mut int_to_bytes1(round));
    /*
     * Note: the specification has an implicit assertion in `int_to_bytes4` that `position / 256 <
     * 2**24`. For efficiency, we do not check for that here as it is checked in `get_permutated_index`.
     */
    seed.append(&mut int_to_bytes4((position / 256) as u32));
    Some(hash(&seed[..]))
}

fn hash_with_round(seed: &[u8], round: u8) -> Vec<u8> {
    let mut seed = seed.to_vec();
    seed.append(&mut int_to_bytes1(round));
    hash(&seed[..])
}

fn bytes_to_int64(bytes: &[u8]) -> u64 {
    let mut cursor = Cursor::new(bytes);
    cursor.get_u64_le()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::H256 as Hash256;
    use hex;
    use std::{fs::File, io::prelude::*, path::PathBuf};
    use yaml_rust::yaml;

    #[test]
    #[ignore]
    fn fuzz_test() {
        let max_list_size = 2_usize.pow(24);
        let test_runs = 1000;

        // Test at max list_size with the end index.
        for _ in 0..test_runs {
            let index = max_list_size - 1;
            let list_size = max_list_size;
            let seed = Hash256::random();
            let shuffle_rounds = 90;

            assert!(get_permutated_index(index, list_size, &seed[..], shuffle_rounds).is_some());
        }

        // Test at max list_size low indices.
        for i in 0..test_runs {
            let index = i;
            let list_size = max_list_size;
            let seed = Hash256::random();
            let shuffle_rounds = 90;

            assert!(get_permutated_index(index, list_size, &seed[..], shuffle_rounds).is_some());
        }

        // Test at max list_size high indices.
        for i in 0..test_runs {
            let index = max_list_size - 1 - i;
            let list_size = max_list_size;
            let seed = Hash256::random();
            let shuffle_rounds = 90;

            assert!(get_permutated_index(index, list_size, &seed[..], shuffle_rounds).is_some());
        }
    }

    #[test]
    fn returns_none_for_zero_length_list() {
        assert_eq!(None, get_permutated_index(100, 0, &[42, 42], 90));
    }

    #[test]
    fn returns_none_for_out_of_bounds_index() {
        assert_eq!(None, get_permutated_index(100, 100, &[42, 42], 90));
    }

    #[test]
    fn returns_none_for_too_large_list() {
        assert_eq!(
            None,
            get_permutated_index(100, usize::max_value() / 2, &[42, 42], 90)
        );
    }

    #[test]
    fn test_vectors() {
        /*
         * Test vectors are generated here:
         *
         * https://github.com/ethereum/eth2.0-test-generators
         */
        let mut file = {
            let mut file_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            file_path_buf.push("src/specs/test_vector_permutated_index.yml");

            File::open(file_path_buf).unwrap()
        };

        let mut yaml_str = String::new();

        file.read_to_string(&mut yaml_str).unwrap();

        let docs = yaml::YamlLoader::load_from_str(&yaml_str).unwrap();
        let doc = &docs[0];
        let test_cases = doc["test_cases"].as_vec().unwrap();

        for (i, test_case) in test_cases.iter().enumerate() {
            let index = test_case["index"].as_i64().unwrap() as usize;
            let list_size = test_case["list_size"].as_i64().unwrap() as usize;
            let permutated_index = test_case["permutated_index"].as_i64().unwrap() as usize;
            let shuffle_round_count = test_case["shuffle_round_count"].as_i64().unwrap();
            let seed_string = test_case["seed"].clone().into_string().unwrap();
            let seed = hex::decode(seed_string.replace("0x", "")).unwrap();

            let shuffle_round_count = if shuffle_round_count < (u8::max_value() as i64) {
                shuffle_round_count as u8
            } else {
                panic!("shuffle_round_count must be a u8")
            };

            assert_eq!(
                Some(permutated_index),
                get_permutated_index(index, list_size, &seed[..], shuffle_round_count),
                "Failure on case #{} index: {}, list_size: {}, round_count: {}, seed: {}",
                i,
                index,
                list_size,
                shuffle_round_count,
                seed_string,
            );
        }
    }
}
