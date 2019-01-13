use crate::atom;
use crate::immix::Heap;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Value};
use crate::vm;
use hamt_rs::HamtMap;

pub fn bif_maps_find_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_get_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        let target = &args[1];
        match hamt_map.find(target) {
            Some(value) => {
                return Ok((*value).clone());
            },
            _ => {
                let heap = &Heap::new();
                let tuple = tup2!(heap, Value::Atom(atom::from_str("badkey")), (*target).clone());
                return Err(Exception::with_value(Reason::EXC_BADARG, tuple));
            }
        };
    }
    let heap = &Heap::new();
    let tuple = tup2!(heap, Value::Atom(atom::from_str("badmap")), (*map).clone());
    Err(Exception::with_value(Reason::EXC_BADARG, tuple))
}

pub fn bif_maps_from_list_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_is_key_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        let target = &args[1];
        match hamt_map.find(target) {
            Some(value) => {
                return Ok(Value::boolean(true));
            },
            _ => {
                return Ok(Value::boolean(false));
            }
        };

    }
    let heap = &Heap::new();
    let tuple = tup2!(heap, Value::Atom(atom::from_str("badmap")), (*map).clone());
    Err(Exception::with_value(Reason::EXC_BADARG, tuple))
}

pub fn bif_maps_keys_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_merge_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_put_3(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_remove_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_update_3(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_values_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom;
    use crate::process::{self};
    use crate::module;
    use crate::servo_arc::Arc;

    #[test]
    fn test_maps_find_2() {
        unimplemented!();
    }

    #[test]
    fn test_maps_get_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let empty_map: HamtMap<Value, Value> = HamtMap::new();
        let (map, _any) = empty_map.insert(Value::Atom(atom::from_str("test")), Value::Integer(3));
        let args = vec![Value::Map(value::Map(Arc::new(map))), Value::Atom(atom::from_str("test"))];

        let res = bif_maps_get_2(&vm, &process, &args);

        assert_eq!(res, Ok(Value::Integer(3)));
    }

    #[test]
    fn test_maps_get_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let wrong_map = &Value::Integer(3);
        let args = vec![(*wrong_map).clone(), Value::Atom(atom::from_str("test"))];

        if let Err(exception) = bif_maps_get_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
            if let Value::Tuple(tuple) = exception.value {
                unsafe {
                    assert_eq!((*tuple).len, 2);
                    let slice: &[Value] = &(**tuple);
                    let mut iter = slice.iter().peekable();
                    if let Some(val) = iter.next() {
                        assert_eq!(val, &Value::Atom(atom::from_str("badmap")));
                    } else {
                        panic!();
                    }
                    if let Some(val) = iter.next() {
                        assert_eq!(val, wrong_map);
                    } else {
                        panic!();
                    }
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_get_2_bad_key() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let empty_map: HamtMap<Value, Value> = HamtMap::new();
        let (map, _any) = empty_map.insert(Value::Atom(atom::from_str("test")), Value::Integer(3));
        let args = vec![Value::Map(value::Map(Arc::new(map))), Value::Atom(atom::from_str("fail"))];

        if let Err(exception) = bif_maps_get_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
            if let Value::Tuple(tuple) = exception.value {
                unsafe {
                    assert_eq!((*tuple).len, 2);
                    let slice: &[Value] = &(**tuple);
                    let mut iter = slice.iter().peekable();
                    if let Some(val) = iter.next() {
                        assert_eq!(val, &Value::Atom(atom::from_str("badkey")));
                    } else {
                        panic!();
                    }
                    if let Some(val) = iter.next() {
                        assert_eq!(val, &Value::Atom(atom::from_str("fail")));
                    } else {
                        panic!();
                    }
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_from_list_1() {
        unimplemented!();
    }

    #[test]
    fn test_maps_is_key_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let empty_map: HamtMap<Value, Value> = HamtMap::new();
        let (map, _any) = empty_map.insert(Value::Atom(atom::from_str("test")), Value::Integer(3));
        let args = vec![Value::Map(value::Map(Arc::new(map))), Value::Atom(atom::from_str("test"))];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Value::boolean(true)));
    }

    #[test]
    fn test_maps_is_key_2_false() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let empty_map: HamtMap<Value, Value> = HamtMap::new();
        let (map, _any) = empty_map.insert(Value::Atom(atom::from_str("test")), Value::Integer(3));
        let args = vec![Value::Map(value::Map(Arc::new(map))), Value::Atom(atom::from_str("no_key"))];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Value::boolean(false)));
    }

    #[test]
    fn test_maps_is_key_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let wrong_map = &Value::Integer(3);
        let args = vec![(*wrong_map).clone(), Value::Atom(atom::from_str("test"))];

        if let Err(exception) = bif_maps_is_key_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
            if let Value::Tuple(tuple) = exception.value {
                unsafe {
                    assert_eq!((*tuple).len, 2);
                    let slice: &[Value] = &(**tuple);
                    let mut iter = slice.iter().peekable();
                    if let Some(val) = iter.next() {
                        assert_eq!(val, &Value::Atom(atom::from_str("badmap")));
                    } else {
                        panic!();
                    }
                    if let Some(val) = iter.next() {
                        assert_eq!(val, wrong_map);
                    } else {
                        panic!();
                    }
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_keys_1() {
        unimplemented!();
    }

    #[test]
    fn test_maps_merge_2() {
        unimplemented!();
    }

    #[test]
    fn test_maps_put_3() {
        unimplemented!();
    }

    #[test]
    fn test_maps_remove_2() {
        unimplemented!();
    }

    #[test]
    fn test_maps_update_3() {
        unimplemented!();
    }

    #[test]
    fn test_maps_values_1() {
        unimplemented!();
    }
}
