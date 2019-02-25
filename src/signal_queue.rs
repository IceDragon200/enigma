use parking_lot::Mutex;
use std::collections::VecDeque;

use crate::exception::Exception;
use crate::process::PID;
use crate::value::Term;

#[derive(Debug, PartialEq)]
pub enum ExitKind {
    ExitLinked = 0,
}

// #[derive(Copy)]
#[derive(Debug)]
pub enum Signal {
    Exit {
        from: PID,
        reason: Exception,
        kind: ExitKind,
    },
    Message {
        from: PID,
        value: Term,
    },
    Link {
        from: PID,
    },
    Unlink {
        from: PID,
    },
}

#[derive(Default, Debug)]
pub struct SignalQueue {
    /// Internal mailbox from which the process is safe to read.
    /// It only holds messages, other signals are processed as we read the external queue.
    internal: VecDeque<Signal>,

    /// External mailbox, to which other processes can write (while holding the lock)
    /// It holds a mixture of different signals and messages.
    external: VecDeque<Signal>,

    /// Used for synchronizing writes to the external part.
    write_lock: Mutex<()>,
}

impl SignalQueue {
    pub fn new() -> Self {
        SignalQueue {
            internal: VecDeque::new(),
            external: VecDeque::new(),
            write_lock: Mutex::new(()),
        }
    }

    pub fn send_external(&mut self, message: Signal) {
        let _lock = self.write_lock.lock();

        self.external.push_back(message);
    }

    // TODO: I'm not sure if skipping external is allowed since it'll break ordering
    pub fn send_internal(&mut self, message: Signal) {
        self.internal.push_back(message);
    }

    pub fn receive(&mut self) -> Option<Signal> {
        if self.internal.is_empty() {
            let _lock = self.write_lock.lock();

            self.internal.append(&mut self.external);
        }

        self.internal.pop_front()
    }

    // pub fn remove(&mut self) {
    //     self.internal.remove(self.save);
    // }

    pub fn has_messages(&self) -> bool {
        if !self.internal.is_empty() {
            return true;
        }

        let _lock = self.write_lock.lock();

        !self.external.is_empty()
    }
}
