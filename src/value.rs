// use crate::arc_without_weak::ArcWithoutWeak;
use crate::process;
use num::bigint::BigInt;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    // Immediate values
    Nil(), // also known as nil
    Integer(u64),
    Character(u8),
    Atom(usize),
    Catch(),
    Pid(process::PID),
    Port(),
    Ref(),
    Float(f64),
    // Extended values (on heap)
    Cons {
        head: Box<Value>,
        tail: Box<Value>,
    }, // two values TODO: ArcWithoutWeak<[Value; 2]>
    Tuple(Arc<Vec<Value>>), // TODO: allocate on custom heap
    /// Boxed values
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    //Binary(ArcWithoutWeak<ImmutableString>),

    /// An interned string is a string allocated on the permanent space. For
    /// every unique interned string there is only one object allocated.
    //InternedBinary(ArcWithoutWeak<ImmutableString>),
    BigInt(Arc<BigInt>), // ArcWithoutWeak<BigInt>
    // Closure(),
    /// Special values (invalid in runtime)
    // Import(), Export(),
    Literal(usize),
    X(usize),
    Y(usize),
    Label(usize),
    List(Vec<Value>),
    FloatReg(usize),
    AllocList(u64),
    ExtendedLiteral(usize), // TODO; replace at load time
    CP(isize),              // continuation pointer
}

unsafe impl Sync for Value {}

// TODO: maybe box binaries further:
// // contains size, followed in memory by the data bytes
// ProcBin { nbytes: Word } ,
// // contains reference to heapbin
// RefBin,
// // stores data on a separate heap somewhere else with refcount
// HeapBin { nbytes: Word, refc: Word },

impl Value {
    pub fn is_integer(&self) -> bool {
        match *self {
            Value::BigInt(..) => true,
            Value::Integer(..) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match *self {
            Value::Float(..) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Value::Float(..) => true,
            Value::BigInt(..) => true,
            Value::Integer(..) => true,
            _ => false,
        }
    }

    pub fn is_atom(&self) -> bool {
        match *self {
            Value::Atom(..) => true,
            _ => false,
        }
    }

    pub fn is_pid(&self) -> bool {
        match *self {
            Value::Pid(..) => true,
            _ => false,
        }
    }

    pub fn is_ref(&self) -> bool {
        match *self {
            Value::Ref(..) => true,
            _ => false,
        }
    }

    pub fn is_port(&self) -> bool {
        match *self {
            Value::Port(..) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match *self {
            Value::Nil(..) => true,
            _ => false,
        }
    }

    // TODO: is_binary

    pub fn is_list(&self) -> bool {
        match *self {
            Value::Cons { .. } => true,
            Value::Nil(..) => true, // apparently also valid
            _ => false,
        }
    }

    pub fn is_non_empty_list(&self) -> bool {
        match *self {
            Value::Cons { ref tail, .. } => !tail.is_nil(),
            _ => false,
        }
    }

    pub fn is_tuple(&self) -> bool {
        match *self {
            Value::Tuple(..) => true,
            _ => false,
        }
    }

    pub fn to_usize(&self) -> usize {
        match *self {
            Value::Atom(i) => i,
            Value::Label(i) => i,
            _ => panic!("Unimplemented to_integer for {:?}", self),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Nil() => write!(f, "nil"),
            _ => write!(f, "(val)"),
        }
    }
}

// /// A pointer to a value managed by the GC.
// #[derive(Clone, Copy)]
// pub struct ValuePointer {
//     pub raw: TaggedPointer<Value>,
// }

// unsafe impl Send for ValuePointer {}
// unsafe impl Sync for ValuePointer {}
