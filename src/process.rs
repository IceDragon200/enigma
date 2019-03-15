pub use self::table::PID;
use crate::atom;
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::loader::LValue;
use crate::mailbox::Mailbox;
use crate::module::{Module, MFA};
use crate::vm::Machine;
// use crate::servo_arc::Arc; can't do receiver self
use crate::signal_queue::SignalQueue;
pub use crate::signal_queue::{ExitKind, Signal};
use crate::value::{self, Term, TryInto};
use crate::vm::RcState;
use hashbrown::{HashMap, HashSet};
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub mod registry;
pub mod table;

/// Heavily inspired by inko

pub type RcProcess = Pin<Arc<Process>>;

pub type Ref = usize;

// TODO: max registers should be a MAX_REG constant for (x and freg), OTP uses 1024
// regs should be growable and shrink on live
// also, only store "live" regs in the execution context and swap them into VM/scheduler
// ---> sched should have it's own ExecutionContext
// also this way, regs could be a &mut [] slice with no clone?

pub const MAX_REG: usize = 16;

bitflags! {
    pub struct Flag: u8 {
        const INITIAL = 0;
        const TRAP_EXIT = (1 << 0);
    }
}

#[derive(Debug)]
pub struct ExecutionContext {
    /// X registers.
    pub x: [Term; MAX_REG],
    /// Floating point registers.
    pub f: [f64; MAX_REG],
    /// Stack (accessible through Y registers).
    pub stack: Vec<Term>,
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
    /// Reductions left
    pub reds: usize,
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

impl ExecutionContext {
    pub fn new(module: *const Module) -> ExecutionContext {
        ExecutionContext {
            x: [Term::nil(); 16],
            f: [0.0f64; 16],
            stack: Vec::new(),
            catches: 0,
            ip: InstrPtr { ptr: 0, module },
            cp: None,
            live: 0,

            exc: None,

            current: MFA(0, 0, 0),

            // register: Register::new(block.code.registers as usize),
            // binding: Binding::with_rc(block.locals(), block.receiver),
            // line: block.code.line,

            // TODO: not great
            bs: unsafe { std::mem::uninitialized() },
            reds: 0,
        }
    }
}

bitflags! {
    pub struct StateFlag: u8 {
        const INITIAL = 0;

        const PRQ_OFFSET = 0;
        const PRQ_BITS = 3;

        const PRQ_MAX = 0b100;
        const PRQ_HIGH = 0b11;
        const PRQ_MEDIUM = 0b10;
        const PRQ_LOW = 0b01;

        const PRQ_MASK = (1 << Self::PRQ_BITS.bits) - 1;
    }
}

#[derive(Debug)]
pub struct LocalData {
    pub state: StateFlag,

    parent: PID,

    pub group_leader: PID,

    // name (atom)
    pub name: Option<u32>,

    /// error handler, defaults to error_handler
    pub error_handler: u32,

    // links (tree)
    pub links: HashSet<PID>,
    // monitors (tree)
    pub monitors: HashMap<Ref, PID>,
    // lt_monitors (list)
    pub lt_monitors: Vec<(PID, Ref)>,

    // signals are sent on death, and the receiving side cleans up it's link/mon structures
    pub signal_queue: SignalQueue,

    pub mailbox: Mailbox,

    pub flags: Flag,

    /// The ID of the thread this process is pinned to.
    pub thread_id: Option<u8>,

    /// A [process dictionary](https://www.erlang.org/course/advanced#dict)
    pub dictionary: HashMap<Term, Term>,
}

#[derive(Debug)]
pub struct Process {
    /// Data stored in a process that should only be modified by a single thread
    /// at once.
    pub local_data: LocalData,

    pub context: ExecutionContext,

    pub heap: Heap,

    /// The process identifier of this process.
    pub pid: PID,

    /// If the process is waiting for a message.
    pub waiting_for_message: AtomicBool,

    /// Waker associated with the wait
    pub waker: Option<std::task::Waker>,
}

unsafe impl Sync for LocalData {}
unsafe impl Send for LocalData {}
unsafe impl Sync for ExecutionContext {}
unsafe impl Send for ExecutionContext {}
unsafe impl Sync for Process {}
impl RefUnwindSafe for Process {}

impl Process {
    pub fn with_rc(
        pid: PID,
        parent: PID,
        context: ExecutionContext,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let local_data = LocalData {
            // allocator: LocalAllocator::new(global_allocator.clone(), config),
            flags: Flag::INITIAL,
            state: StateFlag::INITIAL,
            parent,
            group_leader: parent,
            name: None,
            error_handler: atom::ERROR_HANDLER,
            links: HashSet::new(),
            monitors: HashMap::new(),
            lt_monitors: Vec::new(),
            signal_queue: SignalQueue::new(),
            mailbox: Mailbox::new(),
            thread_id: None,
            dictionary: HashMap::new(),
        };

        Arc::pin(Process {
            pid,
            heap: Heap::new(),
            local_data: local_data,
            context,
            waiting_for_message: AtomicBool::new(false),
            waker: None,
        })
    }

    pub fn from_block(
        pid: PID,
        parent: PID,
        module: *const Module,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let context = ExecutionContext::new(module);

        Process::with_rc(pid, parent, context /*global_allocator, config*/)
    }

    // #[allow(clippy::mut_from_ref)]
    // pub fn context_mut(&mut self) -> &mut ExecutionContext {
    //     &mut *self.local_data_mut().context
    // }

    // pub fn context(&self) -> &ExecutionContext {
    //     &*self.local_data_mut().context
    // }

    // TODO: replace in the long run
    #[allow(clippy::mut_from_ref)]
    pub fn local_data_mut(&self) -> &mut LocalData {
        unsafe { &mut *(&self.local_data as *const LocalData as *mut LocalData) }
    }

    // pub fn local_data(&self) -> &LocalData {
    //     unsafe { &*self.local_data.get() }
    // }

    pub fn is_main(&self) -> bool {
        self.pid == 0
    }

    pub fn send_signal(&self, signal: Signal) {
        self.local_data_mut().signal_queue.send_external(signal);
    }

    // TODO: remove
    pub fn send_message(&self, from: PID, message: Term) {
        if from == self.pid {
            // skip the signal_queue completely
            self.local_data_mut().mailbox.send(message);
        } else {
            self.local_data_mut()
                .signal_queue
                .send_external(Signal::Message {
                    value: message,
                    from,
                });
        }
    }

    // awkward result, but it works
    pub fn receive(&mut self) -> Result<Option<Term>, Exception> {
        if !self.local_data.mailbox.has_messages() {
            self.process_incoming()?
        }
        Ok(self.local_data_mut().mailbox.receive())
    }

    pub fn wake_up(&self) {
        // TODO: will require locking
        self.waiting_for_message.store(false, Ordering::Relaxed);
        match &self.waker {
            Some(waker) => waker.wake(),
            None => (),
        }
    }

    pub fn set_waiting_for_message(&self, value: bool) {
        self.waiting_for_message.store(value, Ordering::Relaxed)
    }

    pub fn is_waiting_for_message(&self) -> bool {
        self.waiting_for_message.load(Ordering::Relaxed)
    }

    // we're in receive(), but ran out of internal messages, process external queue
    /// An Err signals that we're now exiting.
    pub fn process_incoming(&mut self) -> Result<(), Exception> {
        // get internal, if we ran out, start processing external
        while let Some(signal) = self.local_data.signal_queue.receive() {
            match signal {
                Signal::Message { value, .. } => {
                    self.local_data.mailbox.send(value);
                }
                Signal::Exit { .. } => {
                    self.handle_exit_signal(signal)?;
                }
                Signal::Link { from } => {
                    self.local_data.links.insert(from);
                }
                Signal::Unlink { from } => {
                    self.local_data.links.remove(&from);
                }
                Signal::MonitorDown { .. } => {
                    // monitor down: delete from monitors tree, deliver :down message
                    self.handle_monitor_down_signal(signal);
                }
                Signal::Monitor { from, reference } => {
                    self.local_data.lt_monitors.push((from, reference));
                }
                Signal::Demonitor { from, reference } => {
                    if let Some(pos) = self
                        .local_data
                        .lt_monitors
                        .iter()
                        .position(|(x, r)| *x == from && *r == reference)
                    {
                        self.local_data.lt_monitors.remove(pos);
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_monitor_down_signal(&mut self, signal: Signal) {
        // Create a 'DOWN' message and replace the signal with it...
        if let Signal::MonitorDown {
            from,
            reason,
            reference,
        } = signal
        {
            // assert!(is_immed(reason));
            let from = Term::pid(from);
            let reference = Term::reference(&self.heap, reference as usize);
            let reason = reason.value;

            let msg = tup!(
                &self.heap,
                atom!(DOWN),
                reference,
                atom!(PROCESS),
                from,
                reason
            );
            self.local_data.mailbox.send(msg);
        // bump reds by 8?
        } else {
            unreachable!();
        }
    }

    /// Return value is true if the process is now terminating.
    pub fn handle_exit_signal(&mut self, signal: Signal) -> Result<(), Exception> {
        // this is extremely awkward, wish we could enforce a signal variant on the function signature
        // we're also technically matching twice since process_incoming also pattern matches.
        // TODO: inline?
        if let Signal::Exit { kind, from, reason } = signal {
            let mut reason = reason.value;
            let local_data = &mut self.local_data;

            if kind == ExitKind::ExitLinked {
                // delete from link tree
                if local_data.links.take(&from).is_none() {
                    // if it was already deleted, ignore
                    return Ok(());
                }
            }

            if reason != atom!(KILL) && local_data.flags.contains(Flag::TRAP_EXIT) {
                // if reason is immed, create an EXIT message tuple instead and replace
                // (push to internal msg queue as message)
                let msg = tup3!(&self.heap, atom!(EXIT), Term::pid(from), reason);
                // TODO: ensure we do process wakeup
                // erts_proc_notify_new_message(c_p, ERTS_PROC_LOCK_MAIN);
                local_data.mailbox.send(msg);
                Ok(())
            } else if reason == atom!(NORMAL)
            /*&& xsigd.u.normal_kills */
            {
                /* TODO: for exit/2, exit_signal/2 implement normal kills
                 * Preserve the very old and *very strange* behaviour
                 * of erlang:exit/2...
                 *
                 * - terminate ourselves even though exit reason
                 *   is normal (unless we trap exit)
                 * - terminate ourselves before exit/2 return
                 */

                // ignore
                Ok(())
            } else {
                // terminate
                // save = true;
                if
                /*op == ERTS_SIG_Q_OP_EXIT && */
                reason == atom!(KILL) {
                    reason = atom!(KILLED);
                }

                // if save { // something to do with heap fragments I think mainly to remove it from proc
                //     sig->data.attached = ERTS_MSG_COMBINED_HFRAG;
                //     ERL_MESSAGE_TERM(sig) = xsigd->message;
                //     erts_save_message_in_proc(c_p, sig);
                // }

                // Exit process...

                // kill catches
                self.context.catches = 0;

                // return an exception to trigger process exit
                Err(Exception::with_value(Reason::EXT_EXIT, reason))
            }
        // if destroy { cleanup messages up to signal? }
        } else {
            unreachable!()
        }
    }

    // equivalent of erts_continue_exit_process
    pub fn exit(&mut self, state: &RcState, reason: Exception) {
        let local_data = &mut self.local_data;

        // set state to exiting

        // TODO: cancel timers

        // TODO: unregister process name

        // delete links
        for pid in local_data.links.drain() {
            // TODO: reason has to be deep cloned, make a constructor
            println!("pid={} sending exit signal to from={}", self.pid, pid);
            let msg = Signal::Exit {
                reason: reason.clone(),
                from: self.pid,
                kind: ExitKind::ExitLinked,
            };
            self::send_signal(state, pid, msg);
            // erts_proc_sig_send_link_exit(c_p, c_p->common.id, lnk, reason, SEQ_TRACE_TOKEN(c_p));
        }

        // delete monitors
        for (reference, pid) in local_data.monitors.drain() {
            // we're watching someone else
            // send_demonitor(mon)
            let msg = Signal::Demonitor {
                from: self.pid,
                reference,
            };
            self::send_signal(state, pid, msg);
        }

        for (pid, reference) in local_data.lt_monitors.drain(..) {
            // we're being watched
            // send_monitor_down(mon, reason)
            let msg = Signal::MonitorDown {
                reason: reason.clone(),
                from: self.pid,
                reference,
            };
            self::send_signal(state, pid, msg);
        }
    }
}

pub fn allocate(
    state: &RcState,
    parent: PID,
    module: *const Module,
) -> Result<RcProcess, Exception> {
    let mut process_table = state.process_table.lock();

    let pid = process_table
        .reserve()
        .ok_or_else(|| Exception::new(Reason::EXC_SYSTEM_LIMIT))?;

    let process = Process::from_block(
        pid, parent, module, /*, state.global_allocator.clone(), &state.config*/
    );

    process_table.map(pid, process.clone());

    Ok(process)
}

bitflags! {
    pub struct SpawnFlag: u8 {
        const NONE = 0;
        const LINK = 1;
        const MONITOR = 2;
        // const USE_ARGS = 4;
        // const SYSTEM_PROC = 8;
        // const OFF_HEAP_MSGQ = 16;
        // const ON_HEAP_MSGQ = 32;
    }
}

pub fn spawn(
    state: &RcState,
    parent: &mut Process,
    module: *const Module,
    func: u32,
    args: Term,
    flags: SpawnFlag,
) -> Result<Term, Exception> {
    let new_proc = allocate(state, parent.pid, module)?;
    let mut new_proc = unsafe {
        // TODO: this should happen in the alloc
        let ptr = &*new_proc as *const Process as *mut Process;
        Pin::new_unchecked(&mut *ptr)
    };
    let mut ret = Term::pid(new_proc.pid);

    // Set the arglist into process registers.
    // TODO: it also needs to deep clone all the vals (for example lists etc)
    let mut i = 0;
    let mut cons = &args;
    while let Ok(value::Cons { head, tail }) = cons.try_into() {
        new_proc.context.x[i] = *head;
        i += 1;
        cons = tail;
    }
    // lastly, the tail
    new_proc.context.x[i] = *cons;

    println!(
        "Spawning... pid={} mfa={} args={}",
        new_proc.pid,
        MFA(unsafe { (*module).name }, func, i as u32),
        args
    );

    // TODO: func to ip offset
    let func = unsafe {
        (*module)
            .funs
            .get(&(func, i as u32)) // arglist arity
            .expect("process::spawn could not locate func")
    };

    new_proc.context.ip.ptr = *func;

    // Check if this process should be initially linked to its parent.
    if flags.contains(SpawnFlag::LINK) {
        new_proc.local_data_mut().links.insert(parent.pid);

        parent.local_data.links.insert(new_proc.pid);
    }

    if flags.contains(SpawnFlag::MONITOR) {
        let reference = state.next_ref();

        parent.local_data.monitors.insert(reference, new_proc.pid);

        new_proc
            .local_data_mut()
            .lt_monitors
            .push((parent.pid, reference));

        ret = tup2!(&parent.heap, ret, Term::reference(&parent.heap, reference))
    }

    // let new_proc = unsafe {
    //     let ptr = &*new_proc as *const Process as *mut Process;
    //     Pin::new_unchecked(&mut *ptr)
    // };
    use futures::compat::Compat;
    let future = Compat::new(new_proc);
    tokio::spawn(future);

    Ok(ret)
}

pub fn send_message(
    state: &RcState,
    process: &mut Process,
    pid: Term,
    msg: Term,
) -> Result<Term, Exception> {
    let receiver = match pid.into_variant() {
        value::Variant::Atom(name) => {
            if let Some(process) = state.process_registry.lock().whereis(name) {
                Some(process.clone())
            } else {
                println!("registered name {} not found!", pid);
                return Err(Exception::new(Reason::EXC_BADARG));
            }
        }
        value::Variant::Pid(pid) => state.process_table.lock().get(pid),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    if let Some(receiver) = receiver {
        receiver.send_message(process.pid, msg);

        if receiver.is_waiting_for_message() {
            // wake up
            receiver.wake_up();
        }
    } else {
        println!("NOTFOUND");
    }
    // TODO: if err, we return err that's then put in x0?

    Ok(msg)
}

pub fn send_signal(state: &RcState, pid: PID, signal: Signal) -> bool {
    if let Some(receiver) = state.process_table.lock().get(pid) {
        receiver.send_signal(signal);

        if receiver.is_waiting_for_message() {
            // wake up
            receiver.wake_up();
        }
        return true;
    }
    // TODO: if err, we return err ?
    false
}

pub enum State {
    Done,
    Wait,
    Yield,
}

use std::future::Future;
// use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Poll, Waker};

impl Future for Process {
    type Output = Result<(), ()>;

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        self.waker = None;
        Machine::with_current(|vm| match vm.run_with_error_handling(&mut self.get_mut()) {
            State::Done => Poll::Ready(Ok(())),
            State::Wait => {
                self.waker = Some(waker.clone());
                Poll::Pending
            }
            State::Yield => {
                waker.wake();
                Poll::Pending
            }
        })
    }
}
