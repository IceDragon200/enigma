use crate::atom;
use crate::module;
use crate::numeric::division::{FlooredDiv, OverflowingFlooredDiv};
use crate::numeric::modulo::{Modulo, OverflowingModulo};
use crate::process::{self, RcProcess};
use crate::value::{self, Value};
use crate::vm;
use fnv::FnvHashMap;
use num::bigint::BigInt;
use once_cell::sync::Lazy;
use std::i32;
use std::ops::{Add, Mul, Sub};

type BifResult = Result<Value, String>;
type BifFn = fn(&vm::Machine, &RcProcess, &[Value]) -> BifResult;
type BifTable = FnvHashMap<(usize, usize, usize), Box<BifFn>>;

static BIFS: Lazy<BifTable> = sync_lazy! {
    let mut bifs: BifTable = FnvHashMap::default();
    let erlang = atom::i_from_str("erlang");
    bifs.insert((erlang, atom::i_from_str("+"), 2), Box::new(bif_erlang_add_2));
    bifs.insert((erlang, atom::i_from_str("-"), 2), Box::new(bif_erlang_sub_2));
    bifs.insert((erlang, atom::i_from_str("*"), 2), Box::new(bif_erlang_mult_2));
    bifs.insert((erlang, atom::i_from_str("div"), 2), Box::new(bif_erlang_intdiv_2));
    bifs.insert((erlang, atom::i_from_str("rem"), 2), Box::new(bif_erlang_mod_2));
    bifs.insert((erlang, atom::i_from_str("spawn"), 3), Box::new(bif_erlang_spawn_3));
    bifs.insert((erlang, atom::i_from_str("self"), 0), Box::new(bif_erlang_self_0));
    bifs.insert((erlang, atom::i_from_str("send"), 2), Box::new(bif_erlang_send_2));
    bifs.insert((erlang, atom::i_from_str("is_atom"), 1), Box::new(bif_erlang_is_atom_1));
    bifs.insert((erlang, atom::i_from_str("is_list"), 1), Box::new(bif_erlang_is_list_1));
    bifs.insert((erlang, atom::i_from_str("is_tuple"), 1), Box::new(bif_erlang_is_tuple_1));
    bifs.insert((erlang, atom::i_from_str("is_float"), 1), Box::new(bif_erlang_is_float_1));
    bifs.insert((erlang, atom::i_from_str("is_integer"), 1), Box::new(bif_erlang_is_integer_1));
    bifs.insert((erlang, atom::i_from_str("is_number"), 1), Box::new(bif_erlang_is_number_1));
    bifs.insert((erlang, atom::i_from_str("is_port"), 1), Box::new(bif_erlang_is_port_1));
    bifs.insert((erlang, atom::i_from_str("is_reference"), 1), Box::new(bif_erlang_is_reference_1));
    bifs.insert((erlang, atom::i_from_str("is_function"), 1), Box::new(bif_erlang_is_function_1));
    bifs.insert((erlang, atom::i_from_str("is_boolean"), 1), Box::new(bif_erlang_is_boolean_1));
    // math
    let math = atom::i_from_str("math");
    bifs.insert((math, atom::i_from_str("cos"), 1), Box::new(bif_math_cos_1));
    bifs.insert((math, atom::i_from_str("cosh"), 1), Box::new(bif_math_cosh_1));
    bifs.insert((math, atom::i_from_str("sin"), 1), Box::new(bif_math_sin_1));
    bifs.insert((math, atom::i_from_str("sinh"), 1), Box::new(bif_math_sinh_1));
    bifs.insert((math, atom::i_from_str("tan"), 1), Box::new(bif_math_tan_1));
    bifs.insert((math, atom::i_from_str("tanh"), 1), Box::new(bif_math_tanh_1));
    bifs.insert((math, atom::i_from_str("acos"), 1), Box::new(bif_math_acos_1));
    bifs.insert((math, atom::i_from_str("acosh"), 1), Box::new(bif_math_acosh_1));
    bifs.insert((math, atom::i_from_str("asin"), 1), Box::new(bif_math_asin_1));
    bifs.insert((math, atom::i_from_str("asinh"), 1), Box::new(bif_math_asinh_1));
    bifs.insert((math, atom::i_from_str("atan"), 1), Box::new(bif_math_atan_1));
    bifs.insert((math, atom::i_from_str("atanh"), 1), Box::new(bif_math_atanh_1));
    bifs.insert((math, atom::i_from_str("log"), 1), Box::new(bif_math_log_1));
    bifs.insert((math, atom::i_from_str("log2"), 1), Box::new(bif_math_log2_1));
    bifs.insert((math, atom::i_from_str("log10"), 1), Box::new(bif_math_log10_1));
    bifs.insert((math, atom::i_from_str("sqrt"), 1), Box::new(bif_math_sqrt_1));
    bifs.insert((math, atom::i_from_str("atan2"), 2), Box::new(bif_math_atan2_2));
    // pdict
    bifs.insert((erlang, atom::i_from_str("get"), 0), Box::new(bif_erlang_get_0));
    bifs.insert((erlang, atom::i_from_str("get"), 1), Box::new(bif_erlang_get_1));
    bifs.insert((erlang, atom::i_from_str("get_keys"), 0), Box::new(bif_erlang_get_keys_0));
    bifs.insert((erlang, atom::i_from_str("get_keys"), 1), Box::new(bif_erlang_get_keys_1));
    bifs.insert((erlang, atom::i_from_str("put"), 2), Box::new(bif_erlang_put_2));
    bifs.insert((erlang, atom::i_from_str("erase"), 0), Box::new(bif_erlang_erase_0));
    bifs.insert((erlang, atom::i_from_str("erase"), 1), Box::new(bif_erlang_erase_1));
    // lists
    let lists = atom::i_from_str("lists");
    bifs.insert((lists, atom::i_from_str("member"), 2), Box::new(bif_lists_member_2));
    bifs.insert((lists, atom::i_from_str("reverse"), 2), Box::new(bif_lists_reverse_2));
    bifs.insert((lists, atom::i_from_str("keymember"), 3), Box::new(bif_lists_keymember_3));
    bifs.insert((lists, atom::i_from_str("keysearch"), 3), Box::new(bif_lists_keysearch_3));
    bifs.insert((lists, atom::i_from_str("keyfind"), 3), Box::new(bif_lists_keyfind_3));
    bifs
};

#[inline]
pub fn is_bif(mfa: &module::MFA) -> bool {
    BIFS.contains_key(mfa)
}

#[inline]
pub fn apply(
    vm: &vm::Machine,
    process: &RcProcess,
    mfa: &module::MFA,
    args: &[Value],
) -> BifResult {
    (BIFS.get(mfa).unwrap())(vm, process, args)
}

// let val: Vec<_> = module
//     .imports
//     .iter()
//     .map(|mfa| {
//         (
//             atom::from_index(&mfa.0).unwrap(),
//             atom::from_index(&mfa.1).unwrap(),
//             mfa.2,
//         )
//     })
//     .collect();

/// Bif implementations
#[inline]
fn bif_erlang_spawn_3(vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    // parent: TODO: track parent of process
    // arg[0] = atom for module
    // arg[1] = atom for function
    // arg[2] = arguments for func (well-formed list)
    // opts, options for spawn

    if let [Value::Atom(module), Value::Atom(func), arglist] = &args[..] {
        let registry = vm.modules.lock().unwrap();
        let module = registry.lookup(*module).unwrap();
        // TODO: avoid the clone here since we copy later
        return process::spawn(&vm.state, module, *func, arglist.clone());
    }
    Err("Invalid arguments to erlang::spawn/3".to_string())
}

#[inline]
fn bif_erlang_add_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, add, overflowing_add))
}

#[inline]
fn bif_erlang_sub_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, sub, overflowing_sub))
}

fn bif_erlang_mult_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, mul, overflowing_mul))
}

fn bif_erlang_intdiv_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(
        None,
        args,
        floored_division,
        overflowing_floored_division
    ))
}

fn bif_erlang_mod_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    // TODO: should be rem but it's mod
    Ok(integer_overflow_op!(None, args, modulo, overflowing_modulo))
}

fn bif_erlang_self_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    Ok(Value::Pid(process.pid))
}

fn bif_erlang_send_2(vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // args: dest <pid>, msg <term>
    let pid = &args[0];
    let msg = &args[1];
    let res = process::send_message(&vm.state, process, pid, msg)
        .unwrap()
        .clone();
    Ok(res)
}

fn bif_erlang_is_atom_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_atom()))
}

fn bif_erlang_is_list_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_list()))
}

fn bif_erlang_is_tuple_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_tuple()))
}

fn bif_erlang_is_float_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_float()))
}

fn bif_erlang_is_integer_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_integer()))
}

fn bif_erlang_is_number_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_number()))
}

fn bif_erlang_is_port_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_port()))
}

fn bif_erlang_is_reference_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_ref()))
}

// TODO: is_binary, is_function, is_record

fn bif_erlang_is_function_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_function()))
}

fn bif_erlang_is_boolean_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_boolean()))
}

macro_rules! trig_func {
    (
    $arg:expr,
    $op:ident
) => {{
        let res = match $arg {
            Value::Integer(i) => i as f64, // TODO: potentially unsafe
            Value::Float(value::Float(f)) => f,
            Value::BigInt(..) => panic!("Unimplemented math function for BigInt"),
            _ => return Err("argument error".to_string()),
        };
        Ok(Value::Float(value::Float(res.$op())))
    }};
}

fn bif_math_cos_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], cos)
}

fn bif_math_cosh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], cosh)
}

fn bif_math_sin_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], sin)
}

fn bif_math_sinh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], sinh)
}

fn bif_math_tan_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], tan)
}

fn bif_math_tanh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], tanh)
}
fn bif_math_acos_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], acos)
}

fn bif_math_acosh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], acosh)
}

fn bif_math_asin_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], asin)
}

fn bif_math_asinh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], asinh)
}

fn bif_math_atan_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], atan)
}

fn bif_math_atanh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], atanh)
}

fn bif_math_log_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], ln)
}

fn bif_math_log2_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], log2)
}

fn bif_math_log10_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], log10)
}

fn bif_math_sqrt_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], sqrt)
}

fn bif_math_atan2_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let res = match args[0] {
        Value::Integer(i) => i as f64, // TODO: potentially unsafe
        Value::Float(value::Float(f)) => f,
        Value::BigInt(..) => panic!("Unimplemented math function for BigInt"),
        _ => return Err("argument error".to_string()),
    };
    let arg = match args[1] {
        Value::Integer(i) => i as f64, // TODO: potentially unsafe
        Value::Float(value::Float(f)) => f,
        Value::BigInt(..) => panic!("Unimplemented math function for BigInt"),
        _ => return Err("argument error".to_string()),
    };
    Ok(Value::Float(value::Float(res.atan2(arg))))
}

// Process dictionary

/// Get the whole pdict.
fn bif_erlang_get_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Value = pdict.iter().fold(Value::Nil(), |res, (key, val)| {
        // make tuple
        let tuple = value::tuple(heap, 2);
        tuple[0] = key.clone();
        tuple[1] = val.clone();

        // make cons
        value::cons(heap, Value::Tuple(tuple), res)
    });
    Ok(result)
}

/// Get the value for key in pdict.
fn bif_erlang_get_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    Ok(pdict
        .get(&(args[0]))
        .cloned() // TODO: try to avoid the clone if possible
        .unwrap_or_else(|| Value::Atom(atom::UNDEFINED)))
}

/// Get all the keys in pdict.
fn bif_erlang_get_keys_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Value = pdict
        .keys()
        .fold(Value::Nil(), |res, key| value::cons(heap, key.clone(), res));
    Ok(result)
}

/// Return all the keys that have val
fn bif_erlang_get_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Value = pdict.iter().fold(Value::Nil(), |res, (key, val)| {
        if args[1] == *val {
            value::cons(heap, key.clone(), res)
        } else {
            res
        }
    });
    Ok(result)
}

/// Set the key to val. Return undefined if a key was inserted, or old val if it was updated.
fn bif_erlang_put_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict
        .insert(args[0].clone(), args[1].clone())
        .unwrap_or_else(|| Value::Atom(atom::UNDEFINED)))
}

/// Remove all pdict entries, returning the pdict.
fn bif_erlang_erase_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    // deletes all the entries, returning the whole dict tuple
    let pdict = &mut process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    // we use drain since it means we do a move instead of a copy
    let result: Value = pdict.drain().fold(Value::Nil(), |res, (key, val)| {
        // make tuple
        let tuple = value::tuple(heap, 2);
        tuple[0] = key;
        tuple[1] = val;

        // make cons
        value::cons(heap, Value::Tuple(tuple), res)
    });
    Ok(result)
}

/// Remove a single entry from the pdict and return it.
fn bif_erlang_erase_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // deletes a single entry, returning the val
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict
        .remove(&(args[0]))
        .unwrap_or_else(|| Value::Atom(atom::UNDEFINED)))
}

fn bif_lists_member_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // need to bump reductions as we go
    let reds_left = 1; // read from process
    let mut max_iter = 16 * reds_left;
    // bool non_immed_key;

    if args[1].is_nil() {
        return Ok(Value::Atom(atom::FALSE));
    } else if !args[1].is_list() {
        return Err("badarg".to_string()); // TODO
    }

    let term = &args[0];
    // non_immed_key = is_not_immed(term);
    let mut list = &args[1];

    while let Value::List(l) = *list {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(BIF_P);
            // BIF_TRAP2(bif_export[BIF_lists_member_2], BIF_P, term, list);
            // TODO: ^ trap schedules the process to continue executing (by storing the temp val
            // and passing it in the bif call)
        }

        unsafe {
            let item = &(*l).head;
            if *item == *term {
                // || (non_immed_key && deep_equals) {
                // BIF_RET2(am_true, reds_left - max_iter/16);
                return Ok(Value::Atom(atom::TRUE));
            }
            list = &(*l).tail;
        }
    }

    if !list.is_list() {
        // BUMP_REDS(BIF_P, reds_left - max_iter/16);
        return Err("badarg".to_string()); // TODO
    }
    Ok(Value::Atom(atom::FALSE)) // , reds_left - max_iter/16
}

// static BIF_RETTYPE lists_reverse_alloc(Process *c_p,
//                                        Eterm list_in,
//                                        Eterm tail_in)
// {
//     static const Uint CELLS_PER_RED = 40;

//     Eterm *alloc_top, *alloc_end;
//     Uint cells_left, max_cells;
//     Eterm list, tail;
//     Eterm lookahead;

//     list = list_in;
//     tail = tail_in;

//     cells_left = max_cells = CELLS_PER_RED * ERTS_BIF_REDS_LEFT(c_p);
//     lookahead = list;

//     while (cells_left != 0 && is_list(lookahead)) {
//         lookahead = CDR(list_val(lookahead));
//         cells_left--;
//     }

//     BUMP_REDS(c_p, (max_cells - cells_left) / CELLS_PER_RED);

//     if (is_not_list(lookahead) && is_not_nil(lookahead)) {
//         BIF_ERROR(c_p, BADARG);
//     }

//     alloc_top = HAlloc(c_p, 2 * (max_cells - cells_left));
//     alloc_end = alloc_top + 2 * (max_cells - cells_left);

//     while (alloc_top < alloc_end) {
//         Eterm *pair = list_val(list);

//         tail = CONS(alloc_top, CAR(pair), tail);
//         list = CDR(pair);

//         ASSERT(is_list(list) || is_nil(list));

//         alloc_top += 2;
//     }

//     if (is_nil(list)) {
//         BIF_RET(tail);
//     }

//     ASSERT(is_list(tail) && cells_left == 0);
//     BIF_TRAP2(bif_export[BIF_lists_reverse_2], c_p, list, tail);
// }

// static BIF_RETTYPE lists_reverse_onheap(Process *c_p,
//                                         Eterm list_in,
//                                         Eterm tail_in)
// {
//     static const Uint CELLS_PER_RED = 60;

//     Eterm *alloc_start, *alloc_top, *alloc_end;
//     Uint cells_left, max_cells;
//     Eterm list, tail;

//     list = list_in;
//     tail = tail_in;

//     cells_left = max_cells = CELLS_PER_RED * ERTS_BIF_REDS_LEFT(c_p);

//     ASSERT(HEAP_LIMIT(c_p) >= HEAP_TOP(c_p) + 2);
//     alloc_start = HEAP_TOP(c_p);
//     alloc_end = HEAP_LIMIT(c_p) - 2;
//     alloc_top = alloc_start;

//     /* Don't process more cells than we have reductions for. */
//     alloc_end = MIN(alloc_top + (cells_left * 2), alloc_end);

//     while (alloc_top < alloc_end && is_list(list)) {
//         Eterm *pair = list_val(list);

//         tail = CONS(alloc_top, CAR(pair), tail);
//         list = CDR(pair);

//         alloc_top += 2;
//     }

//     cells_left -= (alloc_top - alloc_start) / 2;
//     HEAP_TOP(c_p) = alloc_top;

//     ASSERT(cells_left >= 0 && cells_left <= max_cells);
//     BUMP_REDS(c_p, (max_cells - cells_left) / CELLS_PER_RED);

//     if (is_nil(list)) {
//         BIF_RET(tail);
//     } else if (is_list(list)) {
//         if (cells_left > CELLS_PER_RED) {
//             return lists_reverse_alloc(c_p, list, tail);
//         }

//         BUMP_ALL_REDS(c_p);
//         BIF_TRAP2(bif_export[BIF_lists_reverse_2], c_p, list, tail);
//     }

//     BIF_ERROR(c_p, BADARG);
// }

fn bif_lists_reverse_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // Handle legal and illegal non-lists quickly.
    if args[0].is_nil() {
        return Ok(args[1].clone());
    } else if !args[1].is_list() {
        return Err("badarg".to_string()); // TODO
    }

    /* We build the reversal on the unused part of the heap if possible to save
     * us the trouble of having to figure out the list size. We fall back to
     * lists_reverse_alloc when we run out of space. */
    // if (HeapWordsLeft(BIF_P) > 8) {
    //     return lists_reverse_onheap(BIF_P, BIF_ARG_1, BIF_ARG_2);
    // }

    // return lists_reverse_alloc(BIF_P, BIF_ARG_1, BIF_ARG_2);

    Err("unimplemented!".to_string())
}

fn bif_lists_keymember_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let res = keyfind(bif_lists_keyfind_3, process, args);

    if let Ok(Value::Tuple(..)) = res {
        return Ok(Value::Atom(atom::TRUE));
    }
    res
}

fn bif_lists_keysearch_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let res = keyfind(bif_lists_keyfind_3, process, args);

    if let Ok(Value::Tuple(t)) = res {
        let heap = &process.context_mut().heap;
        let tuple = value::tuple(heap, 2);
        tuple[0] = Value::Atom(atom::VALUE);
        // TODO: this is awkward
        tuple[1] = Value::Tuple(t);
        // Eterm* hp = HAlloc(BIF_P, 3);
        // return TUPLE2(hp, am_value, res);
        return Ok(Value::Tuple(tuple));
    }
    res
}

fn bif_lists_keyfind_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    keyfind(bif_lists_keyfind_3, process, args)
}

/// Swap process out after this number
const CONTEXT_REDS: usize = 4000;

fn keyfind(_func: BifFn, _process: &RcProcess, args: &[Value]) -> BifResult {
    let mut max_iter: isize = 10 * CONTEXT_REDS as isize;

    let key = &args[0];
    let pos_val = &args[1];
    let mut list = &args[2];

    let pos = pos_val.to_usize();

    // OTP does 3 different loops based on key type (simple, immed, boxed), but luckily in rust we
    // just rely on Eq/PartialEq.

    while let Value::List(ptr) = *list {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(p);
            // BIF_TRAP3(bif_export[Bif], p, key, pos_val, list);
        }

        let term = unsafe { &(*ptr).head };
        list = unsafe { &(*ptr).tail };
        if let Value::Tuple(ptr) = term {
            let tuple = unsafe { &**ptr };
            if pos <= tuple.len {
                if *key == tuple[pos] {
                    return Ok(term.clone());
                }
            }
        }
    }

    if !list.is_nil() {
        // BIF_ERROR(p, BADARG);
    }
    Ok(Value::Atom(atom::FALSE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immix::Heap;

    /// Converts an erlang list to a value vector.
    fn to_vec(value: Value) -> Vec<Value> {
        let mut vec = Vec::new();
        unsafe {
            let mut cons = &value;
            while let Value::List(ptr) = *cons {
                vec.push((*ptr).head.clone());
                cons = &(*ptr).tail;
            }
            // lastly, the tail
            vec.push((*cons).clone());
        }
        vec
    }

    /// Converts a value vector to an erlang list.
    fn from_vec(heap: &Heap, vec: Vec<Value>) -> Value {
        vec.into_iter()
            .rev()
            .fold(Value::Nil(), |res, val| value::cons(heap, val, res))
    }

    macro_rules! tup2 {
        ($heap:expr, $element1:expr, $element2:expr) => {{
            let mut tuple = value::tuple($heap, 2);
            tuple[0] = $element1;
            tuple[1] = $element2;
            Value::Tuple(tuple)
        }};
    }

    macro_rules! atom {
        ($const:ident) => {
            Value::Atom(atom::$const)
        };
    }

    #[test]
    fn test_bif_erlang_add_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let res = bif_erlang_add_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(3)));
    }

    #[test]
    fn test_bif_erlang_sub_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(2), Value::Integer(1)];
        let res = bif_erlang_sub_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_bif_erlang_mult_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(2), Value::Integer(4)];
        let res = bif_erlang_mult_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(8)));
    }

    #[test]
    fn test_bif_erlang_intdiv_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(8), Value::Integer(4)];
        let res = bif_erlang_intdiv_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(2)));
    }

    #[test]
    fn test_bif_erlang_mod_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(4), Value::Integer(3)];
        let res = bif_erlang_mod_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_bif_erlang_self_0() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![];
        let res = bif_erlang_self_0(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Pid(process.pid)));
    }

    // TODO: test send_2

    #[test]
    fn test_bif_erlang_is_atom_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    // TODO: test rest of is_type funcs

    #[test]
    fn test_bif_math_cos_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(1)];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Float(value::Float(1.0_f64.cos()))));

        let args = vec![Value::Float(value::Float(1.0))];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Float(value::Float(1.0_f64.cos()))));
    }

    // TODO: test rest of math funcs

    #[test]
    fn test_bif_pdict() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Atom(1), Value::Integer(2)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Value::Atom(1), Value::Integer(3)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(2)));

        let args = vec![Value::Atom(2), Value::Integer(1)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Value::Atom(2)];
        let res = bif_erlang_get_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));

        // TODO: add a assert helper for lists
        let args = vec![];
        let res = bif_erlang_get_0(&vm, &process, &args);
        // assert_eq!(res, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_bif_lists_member_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Value::Atom(1);
        let list = from_vec(heap, vec![Value::Atom(3), Value::Atom(2)]);
        let res = bif_lists_member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Value::Atom(1);
        let list = from_vec(heap, vec![Value::Atom(3), Value::Atom(2), Value::Atom(1)]);
        let res = bif_lists_member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(TRUE)));
    }

    #[test]
    fn test_bif_lists_keyfind_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Value::Atom(1);
        let pos = Value::Integer(5);
        let list = from_vec(heap, vec![]);
        let res = bif_lists_keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Value::Atom(3);
        let pos = Value::Integer(0);
        let list = from_vec(
            heap,
            vec![
                tup2!(heap, Value::Atom(1), Value::Integer(4)),
                tup2!(heap, Value::Atom(2), Value::Integer(3)),
                tup2!(heap, Value::Atom(3), Value::Integer(2)),
                tup2!(heap, Value::Atom(4), Value::Integer(1)),
            ],
        );
        let res = bif_lists_keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(atom!(FALSE)));
    }
}
