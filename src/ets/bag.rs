use super::*;
use crate::immix::Heap;
use crate::value::{Cons, Term, TryFrom, TryInto, TryIntoMut, Tuple, Variant};
use error::*;
use parking_lot::RwLock;
use std::collections::HashSet;

pub(crate) struct Bag {
    meta: Metadata,
    hashmap: RwLock<HashMap<Term, HashSet<Term>>>,
    heap: Heap,
}

unsafe impl Sync for Bag {}
unsafe impl Send for Bag {}

impl Bag {
    pub fn new(meta: Metadata, process: &mut Process) -> Self {
        Self {
            meta,
            hashmap: RwLock::new(HashMap::new()),
            heap: Heap::new(),
        }
    }
}

fn get_key(pos: usize, value: Term) -> Term {
    let tuple = Tuple::try_from(&value).unwrap();
    tuple[pos]
}

impl Table for Bag {
    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn first(&self, process: &mut Process) -> Result<Term> {
        unimplemented!()
    }

    fn next(&self, process: &mut Process, key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn last(&self, process: &mut Process) -> Result<Term> {
        unimplemented!()
    }

    fn prev(&self, process: &mut Process, key: Term) -> Result<Term> {
        unimplemented!()
    }

    // put
    fn insert(&self, process: &mut Process, value: Term, key_clash_fail: bool) -> Result<()> {
        let value = value.deep_clone(&self.heap);
        let key = get_key(self.meta().keypos, value);
        self.hashmap
            .write()
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(value);
        Ok(())
    }

    fn get(&self, process: &mut Process, key: Term) -> Result<Term> {
        match self.hashmap.read().get(&key) {
            Some(set) => Ok(set.iter().fold(Term::nil(), |acc, v| {
                cons!(&process.heap, v.deep_clone(&process.heap), acc)
            })),
            None => Ok(Term::nil()),
        }
    }

    fn get_element(&self, process: &mut Process, key: Term, index: usize) -> Result<Term> {
        match self.hashmap.read().get(&key) {
            Some(set) => Ok(set
                .iter()
                .map(|v| {
                    let tup = Tuple::try_from(&*v).unwrap();
                    assert!(tup.len() > index);
                    tup[index]
                })
                .fold(Term::nil(), |acc, v| {
                    cons!(&process.heap, v.deep_clone(&process.heap), acc)
                })),
            None => Ok(Term::nil()),
        }
    }

    // contains_key ? why is result a Term, not bool
    fn member(&self, key: Term) -> bool {
        self.hashmap.read().contains_key(&key)
    }

    fn update_element(&self, process: &mut Process, key: Term, list: Term) -> Result<Term> {
        unimplemented!();
    }

    // erase  (remove_entry in rust)
    fn remove(&mut self, key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn remove_object(&mut self, object: Term) -> Result<Term> {
        unimplemented!()
    }

    fn slot(&self, slot: Term) -> Result<Term> {
        unimplemented!()
    }

    // int (*db_select_chunk)(process: &mut Process,
    // table: &Self, /* [in out] */
    //                        Eterm tid,
    // Eterm pattern,
    // Sint chunk_size,
    // int reverse,
    // Eterm* ret);

    // _continue is for when the main function traps, let's just use generators
    fn select(
        &self,
        vm: &vm::Machine,
        process: &mut Process,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
        reverse: bool,
    ) -> Result<Term> {
        unimplemented!()
    }

    // fn select_continue(&mut self, process: &mut Process, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_delete(
        &self,
        vm: &vm::Machine,
        process: &mut Process,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
    ) -> Result<Term> {
        unimplemented!()
    }

    // fn select_delete_continue(&mut self, process: &mut Process, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_count(&self, process: &mut Process, tid: Term, pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    // fn select_count_continue(&self, process: &mut Process, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_replace(&mut self, process: &mut Process, tid: Term, pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    // fn select_replace_continue(&mut self, process: &mut Process, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn take(&mut self, process: &mut Process, key: Term) -> Result<Term> {
        unimplemented!()
    }

    /// takes reds, then returns new reds (equal to delete_all)
    fn clear(&mut self, process: &mut Process, reds: usize) -> Result<usize> {
        unimplemented!()
    }
}
