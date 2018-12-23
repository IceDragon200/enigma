use crate::atom;
use crate::immix::Heap;
use crate::value::{self, Value};
use nom::*;
use num::traits::ToPrimitive;
use num_bigint::{BigInt, Sign};
use std::sync::Arc;

/// External Term Format parser

#[allow(dead_code)]
#[derive(Debug)]
enum Tag {
    NewFloat = 70,
    BitBinary = 77,
    AtomCacheRef_ = 82,
    SmallInteger = 97,
    Integer = 98,
    Float = 99,
    Atom = 100, // deprecated latin-1 ? check orig source
    Reference = 101,
    Port = 102,
    Pid = 103,
    SmallTuple = 104,
    LargeTuple = 105,
    Nil = 106,
    String = 107,
    List = 108,
    Binary = 109,
    SmallBig = 110,
    LargeBig = 111,
    NewFun = 112,
    Export = 113,
    NewReference = 114,
    SmallAtom = 115, // deprecated latin-1
    Map = 116,
    Fun = 117,
    AtomU8 = 118,
    SmallAtomU8 = 119,
}

pub fn decode<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    // starts with  be_u8 that's 131
    let (rest, ver) = be_u8(rest)?;
    assert_eq!(ver, 131, "Expected ETF version number to be 131!");
    decode_value(rest, heap)
}

pub fn decode_value<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    // next be_u8 specifies the type tag
    let (rest, tag) = be_u8(rest)?;
    let tag: Tag = unsafe { ::std::mem::transmute(tag) };

    match tag {
        // TODO:
        // NewFloat
        // BitBinary
        // AtomCacheRef_
        Tag::SmallInteger => {
            let (rest, int) = be_u8(rest)?;
            // TODO store inside the pointer once we no longer copy
            Ok((rest, Value::Integer(u64::from(int))))
        }
        // Integer
        // Float
        // Reference
        // Port
        // Pid
        Tag::String => decode_string(rest, heap),
        // Binary
        // NewFun
        // Export
        // NewReference
        // SmallAtom
        // Map
        // Fun
        // AtomU8
        // SmallAtomU8
        Tag::List => decode_list(rest, heap),
        Tag::Atom => decode_atom(rest),
        Tag::Nil => Ok((rest, Value::Nil())),
        Tag::SmallTuple => {
            let (rest, size) = be_u8(rest)?;
            decode_tuple(rest, size as usize, heap)
        }
        Tag::LargeTuple => {
            let (rest, size) = be_u32(rest)?;
            decode_tuple(rest, size as usize, heap)
        }
        Tag::SmallBig => {
            let (rest, size) = be_u8(rest)?;
            decode_bignum(rest, size as usize)
        }
        Tag::LargeBig => {
            let (rest, size) = be_u32(rest)?;
            decode_bignum(rest, size as usize)
        }

        _ => panic!("Tag is {:?}", tag),
    }
}

pub fn decode_atom(rest: &[u8]) -> IResult<&[u8], Value> {
    let (rest, len) = be_u16(rest)?;
    let (rest, string) = take_str!(rest, len)?;

    // TODO: create atom &string
    Ok((rest, atom::from_str(string)))
}

pub fn decode_tuple<'a>(rest: &'a [u8], len: usize, heap: &Heap) -> IResult<&'a [u8], Value> {
    let mut els: Vec<Value> = Vec::with_capacity(len);

    let rest = (0..len).fold(rest, |rest, _i| {
        let (rest, el) = decode_value(rest, heap).unwrap();
        els.push(el);
        rest
    });

    Ok((rest, Value::Tuple(Arc::new(els))))
}

pub fn decode_list<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    let (rest, len) = be_u32(rest)?;

    unsafe {
        let start = heap.alloc(value::Cons {
            head: Value::Nil(),
            tail: Value::Nil(),
        });

        let (tail, rest) = (0..len).fold((start as *mut value::Cons, rest), |(cons, rest), _i| {
            // TODO: probably doing something wrong here
            let value::Cons {
                ref mut head,
                ref mut tail,
            } = *cons;
            let (rest, val) = decode_value(rest, heap).unwrap();
            let new_cons = heap.alloc(value::Cons {
                head: Value::Nil(),
                tail: Value::Nil(),
            });
            std::mem::replace(&mut *head, val);
            std::mem::replace(&mut *tail, Value::Cons(new_cons as *const value::Cons));
            (new_cons as *mut value::Cons, rest)
        });

        // set the tail
        let (rest, val) = decode_value(rest, heap).unwrap();
        (*tail).tail = val;
        println!("val: {}", Value::Cons(start));
        Ok((rest, Value::Cons(start)))
    }
}

/// A string of bytes encoded as tag 107 (String) with 16-bit length.
/// This is basically a list, but it's optimized to decode to char.
pub fn decode_string<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    let (rest, len) = be_u16(rest)?;
    if len == 0 {
        return Ok((rest, Value::Nil()));
    }

    unsafe {
        let start = heap.alloc(value::Cons {
            head: Value::Nil(),
            tail: Value::Nil(),
        });

        let (tail, rest) =
            (0..len - 1).fold((start as *mut value::Cons, rest), |(cons, rest), _i| {
                // TODO: probably doing something wrong here
                let value::Cons {
                    ref mut head,
                    ref mut tail,
                } = *cons;
                let (rest, elem) = be_u8(rest).unwrap();
                let new_cons = heap.alloc(value::Cons {
                    head: Value::Nil(),
                    tail: Value::Nil(),
                });
                std::mem::replace(&mut *head, Value::Character(elem));
                std::mem::replace(&mut *tail, Value::Cons(new_cons as *const value::Cons));
                (new_cons as *mut value::Cons, rest)
            });

        // set the tail
        let (rest, val) = be_u8(rest).unwrap();
        (*tail).head = Value::Character(val);
        println!("{:?}", rest);

        println!("val: {}", Value::Cons(start));
        Ok((rest, Value::Cons(start)))
    }
}

#[cfg(target_pointer_width = "32")]
pub const WORD_BITS: usize = 32;

#[cfg(target_pointer_width = "64")]
pub const WORD_BITS: usize = 64;

pub fn decode_bignum(rest: &[u8], size: usize) -> IResult<&[u8], Value> {
    let (rest, sign) = be_u8(rest)?;

    let sign = if sign == 0 { Sign::Plus } else { Sign::Minus };

    let (rest, digits) = take!(rest, size)?;
    let big = BigInt::from_bytes_le(sign, digits);

    // Assert that the number fits into small
    if big.bits() < WORD_BITS - 4 {
        let b_signed = big.to_isize().unwrap();
        return Ok((rest, Value::Integer(b_signed as u64)));
    }

    // Determine storage size in words
    //unsafe { Ok(tb.create_bignum(big)?) }
    Ok((rest, Value::Integer(123)))
    //Ok((rest, Value::BigNum(b_signed));
}
