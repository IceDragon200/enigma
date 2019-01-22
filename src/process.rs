use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::loader::LValue;
use crate::loader::{FuncInfo, LINE_INVALID_LOCATION};
use crate::mailbox::Mailbox;
use crate::module::{Module, MFA};
use crate::pool::Job;
pub use crate::process_table::PID;
use crate::value::{self, Term, TryInto};
use crate::vm::RcState;
use hashbrown::HashMap;
use std::cell::UnsafeCell;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Heavily inspired by inko

pub type RcProcess = Arc<Process>;

// TODO: max registers should be a MAX_REG constant for (x and freg), OTP uses 1024
// regs should be growable and shrink on live
// also, only store "live" regs in the execution context and swap them into VM/scheduler
// ---> sched should have it's own ExecutionContext
// also this way, regs could be a &mut [] slice with no clone?

pub const MAX_REG: usize = 16;

pub struct ExecutionContext {
    /// X registers.
    pub x: [Term; MAX_REG],
    /// Floating point registers.
    pub f: [f64; MAX_REG],
    /// Stack (accessible through Y registers).
    pub stack: Vec<Term>,
    pub heap: Heap,
    /// Number of catches on stack.
    pub catches: usize,
    /// Program pointer, points to the current instruction.
    pub ip: InstrPtr,
    /// Continuation pointer
    pub cp: Option<InstrPtr>,
    /// Current function
    pub current: MFA,
    pub live: usize,
    /// binary construction state
    pub bs: *mut Vec<u8>,
    ///
    pub exc: Option<Exception>,
    pub flags: Flag,
}

bitflags! {
    pub struct Flag: u32 {
        const INITIAL = 0;
        const TRAP_EXIT = (1 << 0);
    }
}

impl ExecutionContext {
    #[inline]
    // TODO: expand_arg should return by value
    pub fn expand_arg(&self, arg: &LValue) -> Term {
        match arg {
            // TODO: optimize away into a reference somehow at load time
            LValue::ExtendedLiteral(i) => unsafe { (*self.ip.module).literals[*i as usize] },
            LValue::X(i) => self.x[*i as usize],
            LValue::Y(i) => self.stack[self.stack.len() - (*i + 2) as usize],
            LValue::Integer(i) => Term::int(*i as i32), // TODO: make LValue i32
            LValue::Atom(i) => Term::atom(*i),
            LValue::Nil => Term::nil(),
            value => unimplemented!("expand unimplemented for {:?}", value),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub struct InstrPtr {
    /// Module containing the instruction set.
    pub module: *const Module,
    /// Offset to the current instruction.
    pub ptr: u32,
}

unsafe impl Send for InstrPtr {}
unsafe impl Sync for InstrPtr {}

impl InstrPtr {
    pub fn new(module: *const Module, ptr: u32) -> Self {
        InstrPtr { module, ptr }
    }

    // typedef struct {
    //     ErtsCodeMFA* mfa;		/* Pointer to: Mod, Name, Arity */
    //     Uint needed;		/* Heap space needed for entire tuple */
    //     Uint32 loc;			/* Location in source code */
    //     Eterm* fname_ptr;		/* Pointer to fname table */
    // } FunctionInfo;

    /// Find a function from the given pc and fill information in
    /// the FunctionInfo struct. If the full_info is non-zero, fill
    /// in all available information (including location in the
    /// source code).
    pub fn lookup_func_info(&self) -> Option<(MFA, Option<FuncInfo>)> {
        let module = unsafe { &(*self.module) };

        let mut vec: Vec<(&(u32, u32), &u32)> = module.funs.iter().collect();
        vec.sort_by(|(_, v1), (_, v2)| v1.cmp(v2));

        let mut low: u32 = 0;
        let mut high = (vec.len() - 1) as u32;

        while low < high {
            let mid = low + (high - low) / 2;
            if self.ptr < *vec[mid as usize].1 {
                high = mid;
            } else if self.ptr < *vec[(mid + 1) as usize].1 {
                let ((f, a), fun_offset) = vec[mid as usize];
                let mfa = (module.name, *f, *a);
                let func_info = self.lookup_loc();
                return Some((mfa, func_info));
            } else {
                low = mid + 1;
            }
        }
        None
    }

    pub fn lookup_loc(&self) -> Option<FuncInfo> {
        // TODO limit search scope in the future by searching between (current func, currentfunc+1);
        let module = unsafe { &(*self.module) };

        let mut low = 0;
        let mut high = module.lines.len() - 1;

        while high > low {
            let mid = low + (high - low) / 2;
            if self.ptr < module.lines[mid].1 {
                high = mid;
            } else if self.ptr < module.lines[mid + 1].1 {
                let res = module.lines[mid];

                if res == LINE_INVALID_LOCATION {
                    return None;
                }

                return Some(res);
            } else {
                low = mid + 1;
            }
        }
        None
    }
}

// TODO: these are kinda messy since Opt<ptr> vs ptr deboxes differently

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<value::Boxed<Option<InstrPtr>>> for Term {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&value::Boxed<Option<InstrPtr>>, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == value::BOXED_CP {
                    return Ok(&*(ptr as *const value::Boxed<Option<InstrPtr>>));
                }
            }
        }
        Err(value::WrongBoxError)
    }
}
// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<value::Boxed<InstrPtr>> for Term {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&value::Boxed<InstrPtr>, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == value::BOXED_CATCH {
                    return Ok(&*(ptr as *const value::Boxed<InstrPtr>));
                }
            }
        }
        Err(value::WrongBoxError)
    }
}

impl ExecutionContext {
    pub fn new(module: *const Module) -> ExecutionContext {
            ExecutionContext {
                x: [Term::nil(); 16],
                f: [0.0f64; 16],
                stack: Vec::new(),
                heap: Heap::new(),
                catches: 0,
                ip: InstrPtr { ptr: 0, module },
                cp: None,
                live: 0,

                exc: None,

                current: (0, 0, 0),

                // register: Register::new(block.code.registers as usize),
                // binding: Binding::with_rc(block.locals(), block.receiver),
                // line: block.code.line,

                // TODO: not great
                bs: unsafe { std::mem::uninitialized() },

                flags: Flag::INITIAL,
            }
    }
}

pub struct LocalData {
    // allocator, panic handler
    context: Box<ExecutionContext>,

    pub mailbox: Mailbox,

    /// The ID of the thread this process is pinned to.
    pub thread_id: Option<u8>,

    /// A [process dictionary](https://www.erlang.org/course/advanced#dict)
    pub dictionary: HashMap<Term, Term>,
}

pub struct Process {
    /// Data stored in a process that should only be modified by a single thread
    /// at once.
    pub local_data: UnsafeCell<LocalData>,

    /// The process identifier of this process.
    pub pid: PID,

    /// If the process is waiting for a message.
    pub waiting_for_message: AtomicBool,
}

unsafe impl Sync for LocalData {}
unsafe impl Send for LocalData {}
unsafe impl Sync for Process {}
impl RefUnwindSafe for Process {}

impl Process {
    pub fn with_rc(
        pid: PID,
        context: ExecutionContext,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let local_data = LocalData {
            // allocator: LocalAllocator::new(global_allocator.clone(), config),
            context: Box::new(context),
            mailbox: Mailbox::new(),
            thread_id: None,
            dictionary: HashMap::new(),
        };

        Arc::new(Process {
            pid,
            local_data: UnsafeCell::new(local_data),
            waiting_for_message: AtomicBool::new(false),
        })
    }

    pub fn from_block(
        pid: PID,
        module: *const Module,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let context = ExecutionContext::new(module);

        Process::with_rc(pid, context /*global_allocator, config*/)
    }

    #[allow(clippy::mut_from_ref)]
    pub fn context_mut(&self) -> &mut ExecutionContext {
        &mut *self.local_data_mut().context
    }

    #[allow(clippy::mut_from_ref)]
    pub fn local_data_mut(&self) -> &mut LocalData {
        unsafe { &mut *self.local_data.get() }
    }

    pub fn local_data(&self) -> &LocalData {
        unsafe { &*self.local_data.get() }
    }

    pub fn is_main(&self) -> bool {
        self.pid == 0
    }

    pub fn send_message(&self, sender: &RcProcess, message: Term) {
        if sender.pid == self.pid {
            self.local_data_mut().mailbox.send_internal(message);
        } else {
            self.local_data_mut().mailbox.send_external(message);
        }
    }

    pub fn set_waiting_for_message(&self, value: bool) {
        self.waiting_for_message.store(value, Ordering::Relaxed);
    }

    pub fn is_waiting_for_message(&self) -> bool {
        self.waiting_for_message.load(Ordering::Relaxed)
    }
}

pub fn allocate(state: &RcState, module: *const Module) -> Result<RcProcess, Exception> {
    let mut process_table = state.process_table.lock();

    let pid = process_table
        .reserve()
        .ok_or_else(|| Exception::new(Reason::EXC_SYSTEM_LIMIT))?;

    let process = Process::from_block(
        pid, module, /*, state.global_allocator.clone(), &state.config*/
    );

    process_table.map(pid, process.clone());

    Ok(process)
}

pub fn spawn(
    state: &RcState,
    module: *const Module,
    func: u32,
    args: Term,
) -> Result<Term, Exception> {
    println!("Spawning..");
    // let block_obj = block_ptr.block_value()?;
    let new_proc = allocate(state, module)?;
    let new_pid = new_proc.pid;
    // let pid_ptr = new_proc.allocate_usize(new_pid, state.integer_prototype);
    let pid_ptr = Term::pid(new_pid);

    let context = new_proc.context_mut();

    // arglist to process registers,
    // TODO: it also needs to deep clone all the vals (for example lists etc)
    let mut i = 0;
    let mut cons = &args;
    while let Ok(value::Cons { head, tail }) = cons.try_into() {
        context.x[i] = *head;
        i += 1;
        cons = tail;
    }
    // lastly, the tail
    context.x[i] = *cons;

    // TODO: func to ip offset
    let func = unsafe {
        (*module)
            .funs
            .get(&(func, i as u32)) // arglist arity
            .expect("process::spawn could not locate func")
    };

    context.ip.ptr = *func;

    /*
     * Check if this process should be initially linked to its parent.
     */

    // if (so->flags & SPO_LINK) {
    //     ErtsLink *lnk;
    //     ErtsLinkData *ldp = erts_link_create(ERTS_LNK_TYPE_PROC,
    //                                          parent->common.id,
    //                                          p->common.id);
    //     lnk = erts_link_tree_lookup_insert(&ERTS_P_LINKS(parent), &ldp->a);
    //     if (lnk) {
    //         /*
    //          * This should more or less never happen, but could
    //          * potentially happen if pid:s wrap...
    //          */
    //         erts_link_release(lnk);
    //     }
    //     erts_link_tree_insert(&ERTS_P_LINKS(p), &ldp->b);
    // }

    state.process_pool.schedule(Job::normal(new_proc));

    Ok(pid_ptr)
}

pub fn send_message(
    state: &RcState,
    process: &RcProcess,
    // TODO: use pointers for these
    pid: Term,
    msg: Term,
) -> Result<Term, Exception> {
    let pid = pid.to_u32();

    if let Some(receiver) = state.process_table.lock().get(pid) {
        receiver.send_message(process, msg);

        if receiver.is_waiting_for_message() {
            // wake up
            receiver.set_waiting_for_message(false);

            state.process_pool.schedule(Job::normal(receiver));
        }
    }

    Ok(msg)
}
