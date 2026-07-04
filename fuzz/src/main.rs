//! Simple fuzzer testing all available `BitVec`, `BitSet` and `BitMatrix` operations

use std::env;
use std::fs;

use bit_set::BitSet;
use bit_vec::{BitBlock, BitVec};
// use smallvec::SmallVec;

// There's no point growing too much, so try not to grow
// over this size.
const CAP_GROWTH: usize = 256;

macro_rules! next_usize {
    ($b:ident) => {
        $b.next().unwrap_or(0) as usize
    };
}

macro_rules! next_usize_u32 {
    ($b:ident) => {
        u32::from_le_bytes([$b.next().unwrap_or(0), $b.next().unwrap_or(0), $b.next().unwrap_or(0), $b.next().unwrap_or(0)]) as usize
    };
}

macro_rules! next_u8 {
    ($b:ident) => {
        $b.next().unwrap_or(0)
    };
}

macro_rules! next_string {
    ($b:ident) => {
        String::from_utf8(
            (0..next_u8!($b))
                .map(|_| $b.next().unwrap_or(0) & 0b_01_11_11_11)
                .collect::<Vec<_>>(),
        )
        .expect("why do we have unicode where we shouldn't?")
    };
}

macro_rules! next_slice {
    ($b:ident) => {
        (0..next_u8!($b))
            .map(|_| $b.next().unwrap_or(0))
            .collect::<Vec<_>>()
    };
}

macro_rules! next_c_string {
    ($b:ident) => {
        {
            let mut result = String::new();
            while let Some(ch) = $b.next() {
                if ch == 0 {
                    break;
                } else {
                    result.push(ch as char);
                }
            }
            result
        }
    };
}

fn black_box_bit_vec<T: BitBlock>(s: &BitVec<T>) {
    // print to work as a black_box
    print!("{}", s);
}

fn black_box_bit_set<T: BitBlock>(s: &BitSet<T>) {
    // print to work as a black_box
    print!("{}", s);
}

fn do_test<T>(
    data: &[u8],
) -> BitVec<T>
    where T: BitBlock + for<'de> serde::Deserialize<'de> + serde::Serialize + miniserde::Deserialize + miniserde::Serialize + borsh::BorshDeserialize
{
    let mut v = BitVec::<T>::new_general();

    let mut bytes = data.iter().copied();

    while let Some(mut op) = bytes.next() {
        loop {
            match op {
                0 => {
                    v = BitVec::new_general();
                }
                1 => {
                    v = BitVec::with_capacity_general(next_usize!(bytes));
                }
                2 => {
                    v = BitVec::from_bytes_general(&v.to_bytes()[..]);
                }
                3 => {}
                4 => {
                    if v.len() < CAP_GROWTH {
                        v.push(next_u8!(bytes) < 128)
                    }
                }
                5 => {
                    v.pop();
                }
                6 => v.grow(next_usize!(bytes) + v.len(), next_u8!(bytes) < 128),
                7 => {
                    if v.len() < CAP_GROWTH {
                        v.reserve(next_usize!(bytes))
                    }
                }
                8 => {
                    if v.len() < CAP_GROWTH {
                        v.reserve_exact(next_usize!(bytes))
                    }
                }
                9 => v.shrink_to_fit(),
                10 => v.truncate(next_usize!(bytes)),
                11 => black_box_bit_vec(&v),
                12 => {
                    if !v.is_empty() {
                        v.remove(next_usize!(bytes) % v.len());
                    }
                }
                13 => {
                    v.fill(false);
                }
                14 => {
                    if !v.is_empty() {
                        v.remove(next_usize!(bytes) % v.len());
                    }
                }
                15 => {
                    let insert_pos = next_usize!(bytes) % (v.len() + 1);
                    v.insert(insert_pos, next_u8!(bytes) < 128);
                }

                16 => {
                    v = BitVec::from_bytes_general(&v.to_bytes()[..]);
                }

                17 => {
                    v = BitVec::from_bytes_general(data);
                }

                18 => {
                    if v.len() < CAP_GROWTH {
                        let mut v2 = BitVec::<T>::from_bytes_general(data);
                        v.append(&mut v2);
                    }
                }

                19 => {
                    if v.len() < CAP_GROWTH {
                        v.reserve(next_usize!(bytes));
                    }
                }

                20 => {
                    if v.len() < CAP_GROWTH {
                        v.reserve_exact(next_usize!(bytes));
                    }
                }
                21 => {
                    let slice = next_slice!(bytes);
                    v = BitVec::<T>::from_bytes_general(&slice[..]);
                }
                22 => {
                    v.fill(true);
                }
                23 => {
                    let json = serde_json::to_string(&v).unwrap();
                    let deserialized = serde_json::from_str(&json[..]).unwrap();
                    assert_eq!(v, deserialized);
                }
                24 => {
                    let input_str = next_string!(bytes);
                    if let Ok(deserialized) = serde_json::from_str(&input_str[..]) {
                        v = deserialized;
                    }
                }
                25 => {
                    let input_str = next_c_string!(bytes);
                    if let Ok(deserialized) = serde_json::from_str(&input_str[..]) {
                        v = deserialized;
                    }
                }
                26 => {
                    let input_str = next_c_string!(bytes);
                    if let Ok(deserialized) = serde_json::from_str(&input_str[..]) {
                        v = deserialized;
                    }
                }
                27 => {
                    let input_str = next_c_string!(bytes);
                    if let Ok(deserialized) = miniserde::json::from_str(&input_str[..]) {
                        v = deserialized;
                    }
                }
                28 => {
                    let input_vec = next_slice!(bytes);
                    if let Ok(deserialized) = borsh::from_slice(&input_vec[..]) {
                        v = deserialized;
                    }
                }
                29 => {
                    if !v.is_empty() {
                        unsafe {
                            let idx = next_usize_u32!(bytes) % v.len();
                            *v.get_unchecked_mut(idx) = !v.get_unchecked(idx);
                        }
                    }
                }
                other => {
                    op = other.saturating_sub(29);
                    continue;
                }
            }
            break;
        }
    }
    v
}

fn do_test_set<T: BitBlock>(data: &[u8]) -> BitSet<T> {
    let mut v = BitSet::<T>::new_general();

    let mut bytes = data.iter().copied();

    while let Some(mut op) = bytes.next() {
        loop {
            match op {
                0 => {
                    v = BitSet::new_general();
                }
                1 => {
                    v = BitSet::with_capacity_general(next_usize!(bytes));
                }
                2 => {
                    v = BitSet::from_bytes_general(&v.get_ref().to_bytes()[..]);
                }
                3 => {
                    if v.get_ref().len() < CAP_GROWTH {
                        v.reserve_len(next_usize!(bytes))
                    }
                }
                4 => {
                    if v.get_ref().len() < CAP_GROWTH {
                        v.reserve_len_exact(next_usize!(bytes))
                    }
                }
                5 => v.shrink_to_fit(),
                6 => v.truncate(next_usize!(bytes)),
                7 => black_box_bit_set(&v),
                8 => {
                    if !v.is_empty() {
                        v.remove(next_usize!(bytes) % v.get_ref().len());
                    }
                }
                9 => {
                    v.reset();
                }
                10 => {
                    let insert_pos = next_usize!(bytes) % (v.get_ref().len() + 1);
                    v.insert(insert_pos);
                }

                11 => {
                    v = BitSet::from_bytes_general(&v.get_ref().to_bytes()[..]);
                }

                12 => {
                    v = BitSet::from_bytes_general(data);
                }

                13 => {
                    if v.get_ref().len() < CAP_GROWTH {
                        v.reserve_len(next_usize!(bytes));
                    }
                }

                14 => {
                    if v.get_ref().len() < CAP_GROWTH {
                        v.reserve_len_exact(next_usize!(bytes));
                    }
                }
                15 => {
                    let slice = next_slice!(bytes);
                    v = BitSet::<T>::from_bytes_general(&slice[..]);
                }
                other => {
                    op = other.saturating_sub(16);
                    continue;
                }

            }
            break;
        }
    }
    v
}

fn do_test_all(data: &[u8]) {
    do_test::<u32>(data);
    do_test::<u8>(data);
    do_test::<u16>(data);
    do_test::<u64>(data);
    // do_test::<u32, SmallVec<[u32; 8]>>(data);
    // do_test::<u16, Vec<u16>>(data);

    do_test_set::<u32>(data);
    do_test_set::<u8>(data);
    do_test_set::<u16>(data);
    do_test_set::<u64>(data);
    // do_test_set::<u32, SmallVec<[str; 8]>>(data);
    // do_test_set::<u16, Vec<u16>>(data);
}

#[cfg(all(not(feature = "afl"), not(feature = "honggfuzz")))]
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    
    let bytes: Vec<u8> = fs::read(filename)
        .unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", filename, err);
            std::process::exit(1);
        });

    do_test_all(&bytes[..]);
}

#[cfg(feature = "afl")]
fn main() {
    afl::fuzz!(|data| {
        // Remove the panic hook so we can actually catch panic
        // See https://github.com/rust-fuzz/afl.rs/issues/150
        std::panic::set_hook(Box::new(|_| {}));
        do_test_all(data);
    });
}

#[cfg(feature = "honggfuzz")]
fn main() {
    loop {
        honggfuzz::fuzz!(|data| {
            // Remove the panic hook so we can actually catch panic
            // See https://github.com/rust-fuzz/afl.rs/issues/150
            std::panic::set_hook(Box::new(|_| {}));
            do_test_all(data);
        });
    }
}

#[cfg(test)]
mod tests {
    fn extend_vec_from_hex(hex: &str, out: &mut Vec<u8>) {
        let mut b = 0;
        for (idx, c) in hex.as_bytes().iter().enumerate() {
            b <<= 4;
            match *c {
                b'A'..=b'F' => b |= c - b'A' + 10,
                b'a'..=b'f' => b |= c - b'a' + 10,
                b'0'..=b'9' => b |= c - b'0',
                b'\n' => {}
                b' ' => {}
                _ => panic!("Bad hex"),
            }
            if (idx & 1) == 1 {
                out.push(b);
                b = 0;
            }
        }
    }

    #[test]
    fn duplicate_crash() {
        let mut a = Vec::new();
        // paste the output of `xxd -p <crash_dump>` here and run `cargo test`
        extend_vec_from_hex(
            r#"
            787c4a1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d4a1d1d1d1d1d1d1d
            1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d27271d1d1d1d1d1d2727fffe
            270a610a
            "#,
            &mut a,
        );
        super::do_test_all(&a);
    }
}
