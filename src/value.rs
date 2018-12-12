use std::rc::Rc;
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil(), // also known as nil
    Integer(u64),
    Character(i64),
    Atom(usize),
    Catch(),
    // external vals? except Pid can also be internal
    Pid(),
    Port(),
    Ref(),
    // continuation pointer?
    Cons {
        head: Rc<Value>,
        tail: Rc<Value>,
    }, // two values TODO: Rc<[Value; 2]>
    /// Boxed values
    Tuple(Rc<Vec<Value>>), // TODO: allocate on custom heap
    Float(f64),
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    //Binary(ArcWithoutWeak<ImmutableString>),

    /// An interned string is a string allocated on the permanent space. For
    /// every unique interned string there is only one object allocated.
    //InternedBinary(ArcWithoutWeak<ImmutableString>),
    // BigInt(Rc<BigInt>),
    // Closure(),
    // Import(), Export(),
    /// Special values (invalid in runtime)
    Literal(u64),
    X(u64),
    Y(u64),
    Label(u64),
    List(Box<Vec<Value>>),
    FloatReg(u64),
    AllocList(u64),
    ExtendedLiteral(usize), // TODO; replace at load time
}

// TODO: maybe box binaries further:
// // contains size, followed in memory by the data bytes
// ProcBin { nbytes: Word } ,
// // contains reference to heapbin
// RefBin,
// // stores data on a separate heap somewhere else with refcount
// HeapBin { nbytes: Word, refc: Word },

impl Value {
    pub fn is_atom(&self) -> bool {
        match *self {
            Value::Atom(_) => true,
            _ => false,
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
