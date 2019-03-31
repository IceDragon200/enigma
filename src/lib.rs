// The nightly features that are commonly needed with async / await
#![feature(await_macro, async_await, futures_api)]
#![feature(arbitrary_self_types)]
#![recursion_limit = "128"]

// This pulls in the `tokio-async-await` crate. While Rust 2018 doesn't require
// `extern crate`, we need to pull in the macros.
#[macro_use]
extern crate tokio;

#[macro_use]
mod macros;
#[macro_use]
pub mod exception;
#[macro_use]
pub mod vm;
#[macro_use]
pub mod nanbox;
pub mod atom;
mod bif;
pub mod bitstring;
pub mod chashmap;
pub mod etf;
pub mod ets;
pub mod exports_table;
mod immix;
mod instr_ptr;
pub mod loader;
pub mod mailbox;
pub mod module;
pub mod module_registry;
mod numeric;
pub mod opcodes;
pub mod port;
pub mod process;
pub mod servo_arc;
pub mod signal_queue;
pub mod value;

#[macro_use]
extern crate once_cell;

#[macro_use]
extern crate bitflags;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

// extracted from itertools
trait Itertools: Iterator {
    #[inline]
    fn fold_results<A, E, B, F>(&mut self, mut start: B, mut f: F) -> Result<B, E>
    where
        Self: Iterator<Item = Result<A, E>>,
        F: FnMut(B, A) -> B,
    {
        for elt in self {
            match elt {
                Ok(v) => start = f(start, v),
                Err(u) => return Err(u),
            }
        }
        Ok(start)
    }
}

impl<T: ?Sized> Itertools for T where T: Iterator {}
