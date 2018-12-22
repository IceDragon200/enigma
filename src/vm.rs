use crate::atom;
use crate::bif;
use crate::module;
use crate::module_registry::{ModuleRegistry, RcModuleRegistry};
use crate::opcodes::Opcode;
use crate::pool::{Job, JoinGuard as PoolJoinGuard, Pool, Worker};
use crate::process::{self, ExecutionContext, RcProcess};
use crate::process_table::ProcessTable;
use crate::value::Value;
use std::panic;
use std::sync::Arc;
use std::sync::Mutex;
use std::time;

/// A reference counted State.
pub type RcState = Arc<State>;
/// Reference counted ModuleRegistry.

pub struct State {
    /// Table containing all processes.
    pub process_table: Mutex<ProcessTable<RcProcess>>,
    /// Use priorities later on
    pub process_pool: Pool<RcProcess>,

    /// The start time of the VM (more or less).
    pub start_time: time::Instant,
}

#[derive(Clone)]
pub struct Machine {
    pub state: RcState,

    // env config, arguments, panic handler

    // atom table is accessible globally as ATOMS
    // export table
    // module table
    pub modules: RcModuleRegistry,
}

macro_rules! set_register {
    ($context:expr, $register:expr, $value:expr) => {{
        match $register {
            Value::X(reg) => {
                $context.x[*reg] = $value;
            }
            Value::Y(reg) => {
                let len = $context.stack.len();
                $context.stack[len - (*reg + 2)] = $value;
            }
            reg => panic!("Unhandled register type! {:?}", reg),
        }
    }};
}

macro_rules! op_return {
    ($context:expr) => {{
        if $context.cp == -1 {
            println!("Process exited with normal, x0: {:?}", $context.x[0]);
            break;
        }
        op_jump!($context, $context.cp as usize);
        $context.cp = -1;
    }};
}

macro_rules! op_jump {
    ($context:expr, $label:expr) => {{
        $context.ip = $label;
    }};
}

macro_rules! op_is_type {
    ($vm:expr, $context:expr, $args:expr, $op:ident) => {{
        assert_eq!($args.len(), 2);

        // TODO: patch the labels to point to exact offsets to avoid labels lookup
        let l = $vm.expand_arg($context, &$args[0]).to_usize();
        let fail = unsafe { (*$context.module).labels[&l] };

        let val = $vm.expand_arg($context, &$args[1]);

        if !val.$op() {
            op_jump!($context, fail);
        }
    }};
}

macro_rules! safepoint_and_reduce {
    ($vm:expr, $process:expr, $reductions:expr) => {{
        // if $vm.gc_safepoint(&$process) {
        //     return Ok(());
        // }

        // Reduce once we've exhausted all the instructions in a
        // context.
        if $reductions > 0 {
            $reductions -= 1;
        } else {
            $vm.state
                .process_pool
                .schedule(Job::normal($process.clone()));
            return Ok(());
        }
    }};
}

impl Machine {
    pub fn new() -> Machine {
        let primary_threads = 8;
        let process_pool = Pool::new(primary_threads, Some("primary".to_string()));

        let state = State {
            process_table: Mutex::new(ProcessTable::new()),
            process_pool,
            start_time: time::Instant::now(),
        };

        Machine {
            state: Arc::new(state),
            modules: ModuleRegistry::with_rc(),
        }
    }

    /// Starts the VM
    ///
    /// This method will block the calling thread until it returns.
    ///
    /// This method returns true if the VM terminated successfully, false
    /// otherwise.
    pub fn start(&self, file: &str) {
        //self.configure_rayon();

        let primary_guard = self.start_primary_threads();

        self.start_main_process(file);

        // Joining the pools only fails in case of a panic. In this case we
        // don't want to re-panic as this clutters the error output.
        if primary_guard.join().is_err() {
            println!("Primary guard error!")
            //self.set_exit_status(1);
        }
    }

    fn terminate(&self) {
        self.state.process_pool.terminate();
    }

    fn start_primary_threads(&self) -> PoolJoinGuard<()> {
        let machine = self.clone();
        let pool = &self.state.process_pool;

        pool.run(move |worker, process| machine.run_with_error_handling(worker, &process))
    }

    /// Starts the main process
    pub fn start_main_process(&self, path: &str) {
        let process = {
            let module = module::load_module(&self.modules, path).unwrap();

            process::allocate(&self.state, module).unwrap()
        };

        /* TEMP */
        let context = process.context_mut();

        // let fun = atom::i_from_str("fib");
        //let arity = 1;
        // context.x[0] = Value::Integer(23);
        let fun = atom::i_from_str("start");
        let arity = 0;
        unsafe { op_jump!(context, (*context.module).funs[&(fun, arity)]) }
        unsafe { println!("ins: {:?}", (*context.module).instructions) };
        /* TEMP */

        let process = Job::normal(process);
        self.state.process_pool.schedule(process);
    }

    #[inline]
    fn expand_arg<'a>(&'a self, context: &'a ExecutionContext, arg: &'a Value) -> Value {
        match arg {
            // TODO: optimize away into a reference somehow at load time
            Value::ExtendedLiteral(i) => unsafe { (*context.module).literals[*i].clone() },
            Value::X(i) => context.x[*i].clone(),
            Value::Y(i) => context.stack[context.stack.len() - (*i + 2)].clone(),
            value => value.clone(),
        }
    }

    /// Executes a single process, terminating in the event of an error.
    pub fn run_with_error_handling(&self, worker: &mut Worker, process: &RcProcess) {
        // We are using AssertUnwindSafe here so we can pass a &mut Worker to
        // run()/panic(). This might be risky if values captured are not unwind
        // safe, so take care when capturing new variables.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            if let Err(message) = self.run(process) {
                //self.panic(worker, process, &message);
                panic!(message);
            }
        }));

        if let Err(error) = result {
            if let Ok(message) = error.downcast::<String>() {
                //self.panic(worker, process, &message);
                panic!(message);
            } else {
                panic!("The VM panicked with an unknown error");
                /*
                self.panic(
                    worker,
                    process,
                    &"The VM panicked with an unknown error",
                );*/
            };
        }
    }

    pub fn run(&self, process: &RcProcess) -> Result<(), String> {
        let mut reductions = 2000; // self.state.config.reductions;
        let context = process.context_mut();

        println!(
            "running proc pid {:?}, offset {:?}",
            process.pid, context.ip
        );
        loop {
            let ins = unsafe { &(*context.module).instructions[context.ip] };
            println!("running proc pid {:?}, ins {:?}", process.pid, ins.op);
            context.ip += 1;
            match &ins.op {
                Opcode::FuncInfo => {}//println!("Running a function..."),
                Opcode::Move => {
                    // arg1 can be either a value or a register
                    let val = self.expand_arg(context, &ins.args[0]);
                    set_register!(context, &ins.args[1], val)
                }
                Opcode::Return => {
                    op_return!(context)
                }
                Opcode::Send => {
                    // send x1 to x0, write result to x0
                    let pid = &context.x[0];
                    let msg = &context.x[1];
                    let res = process::send_message(&self.state, process, pid, msg)?;
                    context.x[0] = res.clone();
                }
                Opcode::RemoveMessage => {
                    // Unlink the current message from the message queue. Remove any timeout.
                    process.local_data_mut().mailbox.remove();
                    // TODO: clear timeout
                }
                Opcode::Timeout => {
                    //  Reset the save point of the mailbox and clear the timeout flag.
                    process.local_data_mut().mailbox.reset();
                    // TODO: clear timeout
                }
                Opcode::LoopRec => { // label, source
                    // grab message from queue, put to x0, if no message, jump to fail label
                    if let Some(msg) = process.local_data_mut().mailbox.receive() {
                        // TODO: this is very hacky
                        unsafe { context.x[0] = (**msg).clone(); }
                    } else {
                        let l = self.expand_arg(context, &ins.args[0]).to_usize();
                        let fail = unsafe { (*context.module).labels[&l] };
                        op_jump!(context, fail);
                    }
                }
                Opcode::LoopRecEnd => { // label
                    // Advance the save pointer to the next message and jump back to Label.
                    assert_eq!(ins.args.len(), 1);

                    process.local_data_mut().mailbox.advance();

                    let l = self.expand_arg(context, &ins.args[0]).to_usize();
                    let label = unsafe { (*context.module).labels[&l] };
                    op_jump!(context, label);
                }
                Opcode::Wait => { // label
                    // jump to label, set wait flag on process
                    assert_eq!(ins.args.len(), 1);

                    let l = self.expand_arg(context, &ins.args[0]).to_usize();
                    let label = unsafe { (*context.module).labels[&l] };
                    op_jump!(context, label);

                    // set wait flag
                    process.set_waiting_for_message(true);
                    // TODO: return (suspend process)

                }
                Opcode::WaitTimeout => {
                    // @spec wait_timeout Lable Time
                    // @doc  Sets up a timeout of Time milliseconds and saves the address of the
                    //       following instruction as the entry point if the timeout triggers.

                    // TODO: timeout and jump to label if time expires
                    // set wait flag
                    process.set_waiting_for_message(true);
                    // TODO: return (suspend process)
                }
                // TODO: RecvMark(label)/RecvSet(label) for ref based sends
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [Value::Literal(_a), Value::Label(i)] = &ins.args[..] {
                        context.cp = context.ip as isize;
                        op_jump!(context, *i - 2);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::CallExtOnly => {
                    //literal arity, literal destination (module.imports index)
                        println!("{:?}", &ins.args);
                    if let [Value::Literal(arity), Value::Literal(dest)] = &ins.args[..] {
                        // unsafe { println!("{:?}", (*context.module).imports) };
                        let mfa = unsafe { &(*context.module).imports[*dest] };

                        println!("Is bif: {:?}", bif::is_bif(mfa));
                        // TODO: precompute which exports are bifs
                        // call_ext_only Ar=u Bif=u$is_bif => \
                        // allocate u Ar | call_bif Bif | deallocate_return u
                        if bif::is_bif(mfa) {
                            // make a slice out of arity x registers
                            let args = &context.x[0..*arity];
                            let val = bif::apply(self, process, mfa, args);
                            set_register!(context, &Value::X(0), val); // HAXX
                            op_return!(context);
                        } else {
                            panic!("unhandled non-bif call")
                        }
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::AllocateZero => {
                    // literal stackneed, literal live
                    if let [Value::Literal(need), Value::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*need {
                            context.stack.push(Value::Nil())
                        }
                        context.stack.push(Value::CP(context.cp));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::Deallocate => {
                    // literal nwords
                    if let [Value::Literal(nwords)] = &ins.args[..] {
                        let cp = context.stack.pop().unwrap();
                        context.stack.truncate(context.stack.len() - nwords);
                        if let Value::CP(cp) = cp {
                            context.cp = cp;
                        } else {
                            panic!("Bad CP value! {:?}", cp)
                        }
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::IsLt => {
                    // Checks relation, that arg1 IS LESS than arg2, jump to arg0 otherwise.
                    // Structure: is_lt(on_false:CP, a:src, b:src)
                    // assert_arity(gen_op::OPCODE_IS_LT, 3);
                    // shared_equality_opcode(vm, ctx, curr_p, true, Ordering::Less, false)
                    assert_eq!(ins.args.len(), 3);

                    let l = self.expand_arg(context, &ins.args[0]).to_usize();
                    let fail = unsafe { (*context.module).labels[&l] };

                    let v1 = self.expand_arg(context, &ins.args[1]);
                    let v2 = self.expand_arg(context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsEq => {
                    assert_eq!(ins.args.len(), 3);

                    let l = self.expand_arg(context, &ins.args[0]).to_usize();
                    let fail = unsafe { (*context.module).labels[&l] };

                    let v1 = self.expand_arg(context, &ins.args[1]);
                    let v2 = self.expand_arg(context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsFloat        => { op_is_type!(self, context, ins.args, is_float) }
                Opcode::IsNumber       => { op_is_type!(self, context, ins.args, is_number) }
                Opcode::IsAtom         => { op_is_type!(self, context, ins.args, is_atom) }
                Opcode::IsPid          => { op_is_type!(self, context, ins.args, is_pid) }
                Opcode::IsReference    => { op_is_type!(self, context, ins.args, is_ref) }
                Opcode::IsPort         => { op_is_type!(self, context, ins.args, is_port) }
                Opcode::IsNil          => { op_is_type!(self, context, ins.args, is_nil) }
                // Opcode::IsBinary => { op_is_type!(self, context, ins.args, is_binary) }
                Opcode::IsList         => { op_is_type!(self, context, ins.args, is_list) }
                Opcode::IsNonemptyList => { op_is_type!(self, context, ins.args, is_non_empty_list) }
                Opcode::IsTuple        => { op_is_type!(self, context, ins.args, is_tuple) }
                Opcode::Jump => {
                    assert_eq!(ins.args.len(), 1);
                    let label = self.expand_arg(context, &ins.args[0]).to_usize();
                    op_jump!(context, label)
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        let args = vec![
                            self.expand_arg(context, &ins.args[3]),
                            self.expand_arg(context, &ins.args[4]),
                        ];
                        let val = unsafe { bif::apply(self, process, &(*context.module).imports[*i], &args[..]) };

                        set_register!(context, &ins.args[5], val)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
        }

        // Terminate once the main process has finished execution.
        if process.is_main() {
            self.terminate();
        }

        Ok(())
    }

    pub fn elapsed_time(&self) -> time::Duration {
        self.state.start_time.elapsed()
    }
}
