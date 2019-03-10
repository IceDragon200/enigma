//! Pattern matching abstract machine (PAM)
use super::*;
mod error;
use error::*;

use crate::value::{self, Variant, Cons, Tuple, Map, TryFrom, TryInto};
use crate::immix::Heap;
use crate::atom;
use crate::bif::{self};

struct Pattern {}

/// Compilation flags
///
/// The dialect is in the 3 least significant bits and are to be interspaced by
/// by at least 2 (decimal), thats why ((Uint) 2) isn't used. This is to be 
/// able to add DBIF_GUARD or DBIF BODY to it to use in the match_spec bif
/// table. The rest of the word is used like ordinary flags, one bit for each 
/// flag. Note that DCOMP_TABLE and DCOMP_TRACE are mutually exclusive.
bitflags! {
    pub struct Flag: u8 {
        /// Ets and dets. The body returns a value, and the parameter to the execution is a tuple.
        const DCOMP_TABLE = 1;
        /// Trace. More functions are allowed, and the parameter to the execution will be an array.
        const DCOMP_TRACE = 4;
        /// To mask out the bits marking dialect
        const DCOMP_DIALECT_MASK = 0x7;

        /// When this is active, no setting of trace control words or seq_trace tokens will be done.
        const DCOMP_FAKE_DESTRUCTIVE = 8;

        /// Allow lock seizing operations on the tracee and 3rd party processes
        const DCOMP_ALLOW_TRACE_OPS = 0x10;
        /// This is call trace
        const DCOMP_CALL_TRACE = 0x20;


        // /*
        // ** Flags for the guard bif's
        // */

        // /* These are offsets from the DCOMP_* value */
        // #define DBIF_GUARD 1
        // #define DBIF_BODY  0

        // /* These are the DBIF flag bits corresponding to the DCOMP_* value.
        //  * If a bit is set, the BIF is allowed in that context. */
        // #define DBIF_TABLE_GUARD (1 << (DCOMP_TABLE + DBIF_GUARD))
        // #define DBIF_TABLE_BODY  (1 << (DCOMP_TABLE + DBIF_BODY))
        // #define DBIF_TRACE_GUARD (1 << (DCOMP_TRACE + DBIF_GUARD))
        // #define DBIF_TRACE_BODY  (1 << (DCOMP_TRACE + DBIF_BODY))
        // #define DBIF_ALL \
        // DBIF_TABLE_GUARD | DBIF_TABLE_BODY | DBIF_TRACE_GUARD | DBIF_TRACE_BODY
    }
}

/// match VM instructions
enum Opcode {
    MatchArray(usize), /* Only when parameter is an array (DCOMP_TRACE) */
    MatchArrayBind(usize), /* ------------- " ------------ */
    MatchTuple(usize),
    MatchPushT(usize),
    MatchPushL(Term),
    MatchPushM(usize),
    MatchPop(),
    MatchSwap(),
    MatchBind(usize),
    MatchCmp(usize),
    MatchEqBin(Term),
    MatchEqFloat(value::Float),
    MatchEqBig(Bignum),
    MatchEqRef(process::Ref),
    MatchEq(Term),
    MatchList(),
    MatchMap(usize),
    MatchKey(Term),
    MatchSkip(),
    MatchPushC(Term), // constant
    MatchConsA(), /* Car is below Cdr */
    MatchConsB(), /* Cdr is below Car (unusual) */
    MatchMkTuple(usize),
    MatchMkFlatMap(usize),
    MatchMkHashMap(usize),
    MatchCall0(bif::Fn),
    MatchCall1(bif::Fn),
    MatchCall2(bif::Fn),
    MatchCall3(bif::Fn),
    MatchPushV(usize),
    MatchPushVResult(usize), // First variable reference in result
    MatchPushExpr(), // Push the whole expression we're matching ('$_')
    MatchPushArrayAsList(), // Only when parameter is an Array and not an erlang term  (DCOMP_TRACE)
    MatchPushArrayAsListU(), // As above but unknown size
    MatchTrue(),
    MatchOr(usize),
    MatchAnd(usize),
    MatchOrElse(usize),
    MatchAndAlso(usize),
    MatchJump(usize),
    MatchSelf(),
    MatchWaste(),
    MatchReturn(),
    MatchProcessDump(),
    MatchDisplay(),
    MatchIsSeqTrace(),
    MatchSetSeqToken(),
    MatchGetSeqToken(),
    MatchSetReturnTrace(),
    MatchSetExceptionTrace(),
    MatchCatch(),
    MatchEnableTrace(),
    MatchDisableTrace(),
    MatchEnableTrace2(),
    MatchDisableTrace2(),
    MatchTryMeElse(usize), // fail_label
    MatchCaller(),
    MatchHalt(),
    MatchSilent(),
    MatchSetSeqTokenFake(),
    MatchTrace2(),
    MatchTrace3(),
}

pub fn is_variable(obj: Term) -> Option<usize> {
    // byte *b;
    // int n;
    // int N;
    match obj.into_variant() {
        // TODO original checked for < 2 as error but we use nil, true, false as 0,1,2
        Variant::Atom(i) if i > 2 => {
            crate::atom::to_str(i)
                .ok()
                .map(|v| v.as_bytes())
                .and_then(|name| {
                    if name[0] == '$' as u8 {
                        lexical::try_parse::<usize, _>(&name[1..]).ok()
                    } else { None }
                })
        }
        _ => None
    }
}


/// bool tells us if is_constant
type DMCRet = std::result::Result<bool, Error>;

pub(crate) struct Compiler {
    matchexpr: Vec<Term>,
    guardexpr: Vec<Term>,
    bodyexpr: Vec<Term>,
    text: Vec<Opcode>,
    stack: Vec<Term>,
    vars: HashMap<usize, bool>, // is in body
    constant_heap: Heap,
    cflags: Flag,
    stack_used: usize,
    stack_need: usize,
    num_match: usize,
    current_match: usize,
    special: bool,
    is_guard: bool,
    errors: Vec<Error>,
}

impl Compiler {
    pub(crate) fn new(matchexpr: Vec<Term>, guardexpr: Vec<Term>, bodyexpr: Vec<Term>, num_match: usize, cflags: Flag) -> Self {
        Self {
            text: Vec::new(),
            stack: Vec::new(),
            vars: HashMap::new(),
            constant_heap: Heap::new(),
            stack_need: 0,
            stack_used: 0,
            // save: NULL,
            // copy: NULL,
            num_match,
            matchexpr,
            guardexpr,
            bodyexpr,
            errors: Vec::new(),
            cflags,
            special: false,
            is_guard: false,
            current_match: 0 // TODO can maybe remove
        }
    }

    /// The actual compiling of the match expression and the guards.
    pub(crate) fn match_compile(&mut self) -> std::result::Result<Vec<u8>, Error> {
        // MatchProg *ret = NULL;
        // Eterm t;
        // Uint i;
        // Uint num_iters;
        // int structure_checked;
        // DMCRet res;
        let mut current_try_label = -1;
        // Binary *bp = NULL;

        // Compile the match expression.
        for i in 0..self.num_match { // long loop ahead
            self.current_match = i;
            let mut t = self.matchexpr[self.current_match];
            self.stack_used = 0;
            let structure_checked = false;

            if self.current_match < self.num_match - 1 {
                self.text.push(Opcode::MatchTryMeElse(0));
                current_try_label = self.text.len();
            } else {
                current_try_label = -1;
            }

            let clause_start = self.text.len(); // the "special" test needs it
            // TODO, are all these -1 ?
            loop {
                match t.into_variant() {
                    Variant::Pointer(..) => {
                        match t.get_boxed_header().unwrap() {
                            BOXED_MAP => {
                                let map = value::Map::try_from(&t).unwrap().0;
                                let num_iters = map.len();
                                if !structure_checked {
                                    self.text.push(Opcode::MatchMap(num_iters));
                                }
                                structure_checked = false;

                                for (key, value) in map.iter() {
                                    if is_variable(*key).is_some() {
                                        return Err(new_error(ErrorKind::Generic("Variable found in map key.")));
                                    } else if *key == atom!(UNDERSCORE) {
                                        return Err(new_error(ErrorKind::Generic("Underscore found in map key.")));
                                    }
                                    self.text.push(Opcode::MatchKey(key.deep_clone(&self.constant_heap)));
                                    {
                                        self.stack_used += 1;
                                        let old_stack = self.stack_used;
                                        self.one_term(*value).unwrap();
                                        if old_stack != self.stack_used {
                                            assert!(old_stack + 1 == self.stack_used);
                                            self.text.push(Opcode::MatchSwap());
                                        }
                                        if self.stack_used > self.stack_need {
                                            self.stack_need = self.stack_used;
                                        }
                                        self.text.push(Opcode::MatchPop());
                                        self.stack_used -= 1;
                                    }
                                }
                            }
                            BOXED_TUPLE => {
                                let p = Tuple::try_from(&t).unwrap();
                                if !structure_checked { // i.e. we did not pop it
                                    self.text.push(Opcode::MatchTuple(p.len()));
                                }
                                structure_checked = false;
                                for val in p.iter() {
                                    self.one_term(t)?;
                                }
                            }
                            _ => {
                                // goto simple_term;
                                structure_checked = false;
                                self.one_term(t)?;
                            }
                        }
                    }
                    Variant::Cons(..) => {
                        if !structure_checked {
                            self.text.push(Opcode::MatchList());
                        }
                        structure_checked = false; // Whatever it is, we did not pop it
                        let cons = Cons::try_from(&t).unwrap();
                        self.one_term(cons.head)?;
                        t = cons.tail;
                        continue;
                    }
                    _ =>  { // Nil and non proper tail end's or single terms as match expressions.
                        //simple_term:
                        structure_checked = false;
                        self.one_term(t)?;
                    }
                }

                // The *program's* stack just *grows* while we are traversing one composite data
                // structure, we can check the stack usage here

                if self.stack_used > self.stack_need {
                    self.stack_need = self.stack_used;
                }

                // We are at the end of one composite data structure, pop sub structures and emit
                // a matchPop instruction (or break)
                if let Some(val) = self.stack.pop() {
                    t = val;
                    self.text.push(Opcode::MatchPop());
                    structure_checked = true; // Checked with matchPushT or matchPushL
                    self.stack_used -= 1;
                } else {
                    break;
                }
            } // end type loop

            // There is one single top variable in the match expression
            // if the text is two Uint's and the single instruction
            // is 'matchBind' or it is only a skip.
            // self.special =
            //     ((self.text.len() - 1) == 2 + clause_start &&
            //      self.text[clause_start] == Opcode::MatchBind()) ||
            //     ((self.text.len() - 1) == 1 + clause_start &&
            //      self.text[clause_start] == Opcode::MatchSkip());

            // tracing stuff
            // if self.cflags.contains(Flag::DCOMP_TRACE) {
            //     if self.special {
            //         if let Opcode::MatchBind(n) = self.text[clause_start] {
            //             self.text[clause_start] = Opcode::MatchArrayBind(n);
            //         }
            //     } else {
            //         assert!(self.text.len() >= 1);
            //         if self.text[clause_start] != Opcode::MatchTuple() {
            //             // If it isn't "special" and the argument is not a tuple, the expression is not valid when matching an array
            //             return Err(new_error(ErrorKind::Generic("Match head is invalid in this self.")));
            //         }
            //         self.text[clause_start] = Opcode::MatchArray();
            //     }
            // }

            // ... and the guards
            self.is_guard = true;
            self.compile_guard_expr(self.guardexpr[self.current_match])?;
            self.is_guard = false;

            if self.cflags.contains(Flag::DCOMP_TABLE) && !self.bodyexpr[self.current_match].is_list() {
                return Err(new_error(ErrorKind::Generic("Body clause does not return anything.")));
            }

            self.compile_guard_expr(self.bodyexpr[self.current_match])?;

            // The compilation does not bail out when error information is requested, so we need to
            // detect that here...
            if self.err_info != NULL && self.err_info.error_added {
                return Err(());
            }


            // If the matchprogram comes here, the match is successful
            self.text.push(Opcode::MatchHalt());
            // Fill in try-me-else label if there is one.
            if current_try_label >= 0 {
                self.text[current_try_label] = self.text.len();
            }
            
        } /* for (self.current_match = 0 ...) */


        /*
        ** Done compiling
        ** Allocate enough space for the program,
        ** heap size is in 'heap_used', stack size is in 'stack_need'
        ** and text size is simply text.len().
        ** The "program memory" is allocated like this:
        ** text ----> +-------------+
        **            |             |
        **              ..........
        **            +-------------+
        **
        **  The heap-eheap-stack block of a MatchProg is nowadays allocated
        **  when the match program is run (see db_prog_match()).
        **
        ** heap ----> +-------------+
        **              ..........
        ** eheap ---> +             +
        **              ..........
        ** stack ---> +             +
        **              ..........
        **            +-------------+
        ** The stack is expected to grow towards *higher* adresses.
        ** A special case is when the match expression is a single binding
        ** (i.e '$1'), then the field single_variable is set to 1.
        */
        // bp = erts_create_magic_binary(((sizeof(MatchProg) - sizeof(UWord)) +
        //                             (text.len() * sizeof(UWord))),
        //                             erts_db_match_prog_destructor);
        // ret = Binary2MatchProg(bp);
        // ret.saved_program_buf = NULL;
        // ret.saved_program = NIL;
        // ret.term_save = self.save;
        // ret.num_bindings = heap.len();
        // ret.single_variable = self.special;
        // sys_memcpy(ret.text, STACK_DATA(text), text.len() * sizeof(UWord));
        // ret.stack_offset = heap.len()*sizeof(MatchVariable) + FENCE_PATTERN_SIZE;
        // ret.heap_size = ret.stack_offset + self.stack_need * sizeof(Eterm*) + FENCE_PATTERN_SIZE;

    // #ifdef DEBUG
    //     ret.prog_end = ret.text + text.len();
    // #endif

        return bp;
    }

    /// Handle one term in the match expression (not the guard)
    fn one_term(&mut self, c: Term) -> DMCRet {
        // Sint n;
        // Eterm *hp;
        // Uint sz, sz2, sz3;
        // Uint i, j;

        match c.value.tag() as u8 {
            value::TERM_ATOM => {
                let n = is_variable(c);

                if let Some(n) = n { // variable 
                    if self.vars.get(&n).is_some() {
                        self.text.push(Opcode::MatchCmp(n));
                    } else { /* Not bound, bind! */
                        self.text.push(Opcode::MatchBind(n));
                        self.vars[&n] = false; // bind var, set in_guard to false
                    }
                } else if c == atom!(UNDERSCORE) {
                    self.text.push(Opcode::MatchSkip());
                } else {
                    // Any other atom value
                    self.text.push(Opcode::MatchEq(c));
                }
            }
            value::TERM_CONS => {
                self.text.push(Opcode::MatchPushL(c));
                self.stack_used += 1;
            }
            value::TERM_FLOAT => {
                self.text.push(Opcode::MatchEqFloat(c));
            // #ifdef ARCH_64
            //     PUSH(*self.text, 0);
            // #else
            //     PUSH(*self.text, float_val(c)[2] as usize);
            // #endif
            }
            value::TERM_POINTER => {
                match c.get_boxed_header().unwrap() { // inefficient, cast directly
                    value::BOXED_TUPLE => {
                        let n = Tuple::try_from(&c).unwrap().len();
                        self.text.push(Opcode::MatchPushT(n));
                        self.stack_used += 1;
                        self.stack.push(c);
                    }
                    value::BOXED_MAP => {
                        let n = value::Map::try_from(&c).unwrap().0.len();
                        self.text.push(Opcode::MatchPushM(n));
                        self.stack_used += 1;
                        self.stack.push(c);
                    }
                    value::BOXED_REF => {
                        self.text.push(Opcode::MatchEqRef(c));
                    }
                    value::BOXED_BIGINT => {
                        self.text.push(Opcode::MatchEqBig(c));
                    }
                    _ => { /* BINARY, FUN, VECTOR, or EXTERNAL */
                        self.text.push(Opcode::MatchEqBin(c.deep_clone(&self.constant_heap)));
                    }
                }
            }
            _ => {
                // Any immediate value
                self.text.push(Opcode::MatchEq(c));
            }
        }

        Ok(true)
    }

    fn compile_guard_expr(&self, mut l: Term) -> DMCRet {
        // DMCRet ret;
        // int constant;
        // Eterm t;

        if l != Term::nil() {
            if !l.is_list() {
                return Err(new_error(ErrorKind::Generic("Match expression is not a list.")));
            }
            if !self.is_guard {
                self.text.push(Opcode::MatchCatch());
            }
            while let Ok(Cons { head: t, tail }) = l.try_into() {
                let constant = self.expr(*t)?;
                if constant {
                    self.do_emit_constant(*t);
                }
                l = *tail;
                if self.is_guard {
                    self.text.push(Opcode::MatchTrue());
                } else {
                    self.text.push(Opcode::MatchWaste());
                }
                self.stack_used -= 1;
            }
            if !l.is_nil() {
                return Err(new_error(ErrorKind::Generic("Match expression is not a proper list.")));
            }
            if !self.is_guard && self.cflags.contains(Flag::DCOMP_TABLE) {
                assert!(Some(&Opcode::MatchWaste()) == self.text.last());
                self.text.pop();
                self.text.push(Opcode::MatchReturn()); // Same impact on stack as matchWaste
            }
        }
        Ok(())
    }

    /*
    ** Match guard compilation
    */

    fn do_emit_constant(&self, t: Term) {
        let tmp = t.deep_clone(&self.constant_heap);
        self.text.push(Opcode::MatchPushC(tmp));
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
    }

    fn list(&mut self, t: Term) -> DMCRet {
        let cons = Cons::try_from(&t).unwrap();
        let c1 = self.expr(cons.head)?;
        let c2 = self.expr(cons.tail)?;

        if c1 && c2 {
            return Ok(true);
        } 
        if !c1 {
            /* The CAR is not a constant, so if the CDR is, we just push it,
            otherwise it is already pushed. */
            if c2 {
                self.do_emit_constant(cons.tail);
            }
            self.text.push(Opcode::MatchConsA());
        } else { /* !c2 && c1 */
            self.do_emit_constant(cons.head);
            self.text.push(Opcode::MatchConsB());
        }
        self.stack_used -= 1; /* Two objects on stack becomes one */
        Ok(false)
    }

    fn rearrange_constants(&mut self, textpos: usize, p: &[Term], nelems: usize) {
        //STACK_TYPE(UWord) instr_save;
        Uint i;

        INIT_STACK(instr_save);
        while self.text.len() > textpos {
            PUSH(instr_save, POP(*text));
        }
        for (i = nelems; i--;) {
            self.do_emit_constant(p[i]);
        }
        while(!EMPTY(instr_save)) {
            PUSH(*text, POP(instr_save));
        }
        FREE(instr_save);
    }

    fn array(&mut self, terms: &[Term]) -> DMCRet {
        let mut all_constant = true;
        let textpos = self.text.len();
        // Uint i;

        // We remember where we started to layout code,
        // assume all is constant and back up and restart if not so.
        // The array should be laid out with the last element first,
        // so we can memcpy it to the eheap.

        // p = terms, nemels = terms.len()

        for (i = nelems; i--;) {
            let res = self.expr(p[i])?;
            if !res && all_constant {
                all_constant = false;
                if i < nelems - 1 {
                    self.rearrange_constants(textpos, p + i + 1, nelems - i - 1);
                }
            } else if res && !all_constant {
                self.do_emit_constant(p[i]);
            }
        }
        Ok(all_constant)
    }

    fn tuple(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let nelems = t.len();

        let all_constant = self.array(&t[..])?;
        if all_constant {
            return Ok(true);
        }
        self.text.push(Opcode::MatchMkTuple(nelems));
        self.stack_used -= nelems - 1;
        Ok(false)
    }

    fn map(&mut self, t: Term) -> DMCRet {
        assert!(t.is_map());

        let map = value::Map::try_from(&t).unwrap().0;
        let mut constant_values = true;
        let nelems = map.len();

        for (_, val) in map.iter() {
            let c = self.expr(*val)?;
            if !c {
                constant_values = false;
            }
        }

        if constant_values {
            return Ok(true);
        }

        // not constant

        for (key, value) in map.iter() {
            // push key
            let c = self.expr(*key)?;
            if c {
                self.do_emit_constant(*key);
            }
            // push value
            let c = self.expr(*value)?;
            if c {
                self.do_emit_constant(*value);
            }
        }
        self.text.push(Opcode::MatchMkHashMap(nelems));
        self.stack_used -= nelems;
        Ok(false)
    }

    fn whole_expression(&mut self, t: Term) -> DMCRet {
        if self.cflags.contains(Flag::DCOMP_TRACE) {
            // Hmmm, convert array to list...
            if self.special {
                self.text.push(Opcode::MatchPushArrayAsListU());
            } else { 
                assert!(self.matchexpr[self.current_match].is_tuple());
                self.text.push(Opcode::MatchPushArrayAsList());
            }
        } else {
            self.text.push(Opcode::MatchPushExpr());
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    /// Figure out which PushV instruction to use.
    fn add_pushv_variant(&mut self, n: usize) {
        let v = &mut self.vars[&n];
        let mut instr = Opcode::MatchPushV(n);

        if !self.is_guard {
            if !*v {
                instr = Opcode::MatchPushVResult(n);
                *v = true;
            }
        }
        self.text.push(instr);
    }

    fn variable(&mut self, n: usize) -> DMCRet {
        // TODO this is already called inside expr(), just pass number in instead
        // optimize this in beam too
        // Uint n = db_is_variable(t);

        if self.vars.get(&n).is_none() {
            return Err(new_error(ErrorKind::Generic(&format!("Variable ${} is unbound", n))));
        }

        self.add_pushv_variant(n);

        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn all_bindings(&mut self, t: Term) -> DMCRet {
        self.text.push(Opcode::MatchPushC(Term::nil()));
        for (n, _) in self.vars.iter() {
            self.add_pushv_variant(*n);
            self.text.push(Opcode::MatchConsB());
        }
        self.stack_used += 1;
        if (self.stack_used + 1) > self.stack_need  {
            self.stack_need = self.stack_used + 1;
        }
        Ok(false)
    }

    fn constant(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "const", value: t, reason: "with more than one argument" }));
        }
        Ok(true)
    }

    fn and(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "and", value: t, reason: "without arguments" }));
        }
        for val in &p[1..] { // skip the :&&
            let c = self.expr(*val)?;
            if c {
                self.do_emit_constant(*val);
            }
        }
        self.text.push(Opcode::MatchAnd(a - 1));
        self.stack_used -= a - 2;
        Ok(false)
    }

    fn or(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "or", value: t, reason: "without arguments" }));
        }
        for val in &p[1..] { // skip the :||
            let c = self.expr(*val)?;
            if c {
                self.do_emit_constant(*val);
            }
        }
        self.text.push(Opcode::MatchOr(a - 1));
        self.stack_used -= a - 2;
        Ok(false)
    }


    fn andalso(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        // int i;
        // int c;
        // Uint lbl;
        // Uint lbl_next;
        // Uint lbl_val;

        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "andalso", value: t, reason: "without arguments" }));
        }
        let mut lbl = 0;
        let mut iter = p.iter();
        let len = iter.len();
        iter.next(); // drop the operator

        for val in iter.take(len - 2) {
            let c = self.expr(*val)?;
            if c {
                self.do_emit_constant(*val);
            }
            self.text.push(Opcode::MatchAndAlso(lbl));
            lbl = self.text.len()-1;
            self.stack_used -= 1;
        }
        // repeat for last operand, but use a jump
        let last = iter.next().unwrap();
        let c = self.expr(*last)?;
        if c {
            self.do_emit_constant(*last);
        }
        self.text.push(Opcode::MatchJump(lbl));
        lbl = self.text.len()-1;
        self.stack_used -= 1;
        // -- end

        self.text.push(Opcode::MatchPushC(atom!(TRUE)));
        let lbl_val = self.text.len();
        while lbl {
            lbl_next = text[lbl];
            text[lbl] = lbl_val-lbl-1;
            lbl = lbl_next;
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
           self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn orelse(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let a = t.len();
        // int i;
        // int c;
        // Uint lbl;
        // Uint lbl_next;
        // Uint lbl_val;
        
        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "orelse", value: t, reason: "without arguments" }));
        }
        let mut lbl = 0;
        let mut iter = p.iter();
        let len = iter.len();
        iter.next(); // drop the operator

        for val in iter.take(len - 2) {
            let c = self.expr(*val)?;
            if c {
                self.do_emit_constant(*val);
            }
            self.text.push(Opcode::MatchAndAlso(lbl));
            lbl = self.text.len()-1;
            self.stack_used -= 1;
        }
        // repeat for last operand, but use a jump
        let last = iter.next().unwrap();
        let c = self.expr(last)?;
        if c {
            self.do_emit_constant(last);
        }
        self.text.push(Opcode::MatchJump(lbl));
        lbl = self.text.len()-1;
        self.stack_used -= 1;
        // -- end

        self.text.push(Opcode::MatchPushC(atom!(FALSE)));
        let lbl_val = self.text.len();
        while lbl {
            lbl_next = text[lbl];
            text[lbl] = lbl_val-lbl-1;
            lbl = lbl_next;
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn message(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "message" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "message" }));
        }

        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "message", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        self.text.push(Opcode::MatchReturn());
        self.text.push(Opcode::MatchPushC(atom!(TRUE)));
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }

    fn selff(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "self", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchSelf());
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn return_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "return_trace" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "return_trace" }));
        }

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "return_trace", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchSetReturnTrace()); /* Pushes 'true' on the stack */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn exception_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "exception_trace" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "exception_trace" }));
        }

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "exception_trace", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchSetExceptionTrace()); /* Pushes 'true' on the stack */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn check_trace(&self, op: &str, need_cflags: Flag, allow_in_guard: bool) -> DMCRet {
        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: op }))
        }
        if (self.cflags & need_cflags) != need_cflags {
            return Err(new_error(ErrorKind::Generic(&format!("Special form '{}' not allowed for this trace event.", op))));
        }
        if self.is_guard && !allow_in_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: op }));
        }
        Ok(true)
    }

    fn is_seq_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        self.check_trace("is_seq_trace", Flag::DCOMP_ALLOW_TRACE_OPS, true)?;

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "is_seq_trace", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchIsSeqTrace()); 
        /* Pushes 'true' or 'false' on the stack */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn set_seq_token(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        self.check_trace("set_seq_trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        if a != 3 {
            return Err(new_error(ErrorKind::Argument { form: "set_seq_token", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[2])?;
        if c { 
            self.do_emit_constant(p[2]);
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        if self.cflags.contains(Flag::DCOMP_FAKE_DESTRUCTIVE) {
            self.text.push(Opcode::MatchSetSeqTokenFake());
        } else {
            self.text.push(Opcode::MatchSetSeqToken());
        }
        self.stack_used -= 1; /* Remove two and add one */
        Ok(false)
    }

    fn get_seq_token(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        self.check_trace("get_seq_token", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "get_seq_token", value: t, reason: "with arguments" }));
        }

        self.text.push(Opcode::MatchGetSeqToken());
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn display(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "display" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "display" }));
        }

        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "display", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        self.text.push(Opcode::MatchDisplay());
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }

    fn process_dump(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        self.check_trace("process_dump", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "process_dump", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchProcessDump()); /* Creates binary */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn enable_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let arity = p.len();
        
        self.check_trace("enable_trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        match arity {
            2 => {
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchEnableTrace());
                /* Push as much as we remove, stack_need is untouched */
            }
            3 => {
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchEnableTrace2());
                self.stack_used -= 1; /* Remove two and add one */
            }
            _ => return Err(new_error(ErrorKind::Argument { form: "enable_trace", value: t, reason: "with wrong number of arguments" }))
        }
        Ok(false)
    }

    fn disable_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let arity = p.len();

        self.check_trace("disable_trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        match arity {
            2 => {
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchDisableTrace());
                /* Push as much as we remove, stack_need is untouched */
            }
            3 => {
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchDisableTrace2());
                self.stack_used -= 1; // Remove two and add one
            }
            _ => return Err(new_error(ErrorKind::Argument { form: "disable_trace", value: t, reason: "with wrong number of arguments" }))
        }
        Ok(false)
    }

    fn trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let arity = p.len();
        
        self.check_trace("trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        match arity {
            3 => {
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchTrace2());
                self.stack_used -= 1; /* Remove two and add one */
            }
            4 => {
                let c = self.expr(p[3])?;
                if c { 
                    self.do_emit_constant(p[3]);
                }
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchTrace3());
                self.stack_used -= 2; /* Remove three and add one */
            }
            _ => return Err(new_error(ErrorKind::Argument { form: "trace", value: t, reason: "with wrong number of arguments" }))
        }
        Ok(false)
    }

    fn caller(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        self.check_trace("caller", Flag::DCOMP_CALL_TRACE | Flag::DCOMP_ALLOW_TRACE_OPS, false)?;
    
        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "caller", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchCaller()); /* Creates binary */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn silent(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
      
        self.check_trace("silent", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;
    
        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "silent", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        self.text.push(Opcode::MatchSilent());
        self.text.push(Opcode::MatchPushC(atom!(TRUE)));
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }
    
    fn fun(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        let arity = a - 1;
        // int i;
        // DMCGuardBif *b;
    
        /* Special forms. */
        let b = match p[0].into_variant() {
            Variant::Atom(atom::CONST) => return self.constant(t),
            Variant::Atom(atom::AND) => return self.and(t),
            Variant::Atom(atom::OR) => return self.or(t),
            Variant::Atom(atom::ANDALSO) => return self.andalso(t),
            Variant::Atom(atom::ANDTHEN) => return self.andalso(t),
            Variant::Atom(atom::ORELSE) => return self.orelse(t),
            Variant::Atom(atom::SELF) => return self.selff(t),
            Variant::Atom(atom::MESSAGE) => return self.message(t),
            Variant::Atom(atom::IS_SEQ_TRACE) => return self.is_seq_trace(t),
            Variant::Atom(atom::SET_SEQ_TOKEN) => return self.set_seq_token(t),
            Variant::Atom(atom::GET_SEQ_TOKEN) => return self.get_seq_token(t),
            Variant::Atom(atom::RETURN_TRACE) => return self.return_trace(t),
            Variant::Atom(atom::EXCEPTION_TRACE) => return self.exception_trace(t),
            Variant::Atom(atom::DISPLAY) => return self.display(t),
            Variant::Atom(atom::PROCESS_DUMP) => return self.process_dump(t),
            Variant::Atom(atom::ENABLE_TRACE) => return self.enable_trace(t),
            Variant::Atom(atom::DISABLE_TRACE) => return self.disable_trace(t),
            Variant::Atom(atom::TRACE) => return self.trace(t),
            Variant::Atom(atom::CALLER) => return self.caller(t),
            Variant::Atom(atom::SILENT) => return self.silent(t),
            Variant::Atom(atom::SET_TCW) => {
                if self.cflags.contains(Flag::DCOMP_FAKE_DESTRUCTIVE) {
                    lookup_bif(atom!(SET_TCW_FAKE), arity);
                } else {
                    lookup_bif(p[0], arity);
                }
            }
            _ => lookup_bif(p[0],  arity),
        };


        if let None = b {
            if self.err_info != NULL {
                return Err(new_error(ErrorKind::Generic(&format!("Function {}/{} does not exist", p[0], arity))));
            } else {
                return Err(());
            }
        } 
        assert!(b.arity == arity);
        if !(b.flags & 
            (1 << 
                ((self.cflags & Flag::DCOMP_DIALECT_MASK) + 
                (if self.is_guard { Flag::DBIF_GUARD } else { Flag::DBIF_BODY })))) {
            /* Body clause used in wrong context. */
            if self.err_info != NULL {
                return Err(new_error(ErrorKind::Generic(&format!("Function {}/{} cannot be called in this context.", p[0], arity))));
            } else {
                return Err(());
            }
        }

        // not constant

        // why are constants emitted backwards
        for val in &p[1..] { // skip the function name
            let c = self.expr(*val)?;
            if c {
                self.do_emit_constant(*val);
            }
        }

        match b.arity {
            0 => self.text.push(Opcode::MatchCall0(b.biff)),
            1 => self.text.push(Opcode::MatchCall1(b.biff)),
            2 => self.text.push(Opcode::MatchCall2(b.biff)),
            3 => self.text.push(Opcode::MatchCall3(b.biff)),
            _ => panic!("ets:match() internal error, guard with more than 3 arguments."),
        }
        self.stack_used -= a - 2;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }
    
    fn expr(&mut self, t: Term) -> DMCRet {
        match t.value.tag() as u8 {
            value::TERM_CONS => self.list(t),
            value::TERM_POINTER => {
                if t.is_map() {
                    return self.map(t);
                }
                if t.is_tuple() {
                    let p = Tuple::try_from(&t).unwrap();
                    // #ifdef HARDDEBUG
                    //                 erts_fprintf(stderr,"%d %d %d %d\n",arityval(*p),is_tuple(tmp = p[1]),
                    //                 is_atom(p[1]),db_is_variable(p[1]));
                    // #endif
                    if p.len() == 1 && p[0].is_tuple() {
                        self.tuple(p[0])
                    } else if p.len() >= 1 && p[0].is_atom() && is_variable(p[0]).is_none() {
                        self.fun(t)
                    } else {
                        RETURN_TERM_ERROR("%T is neither a function call, nor a tuple (tuples are written {{ ... }}).", t);
                    }
                } else {
                    Ok(true)
                }
            }
            value::TERM_ATOM => { // immediate
                let n = is_variable(t);

                if let Some(n) = n {
                    self.variable(n)
                } else if t == atom!(DOLLAR_UNDERSCORE) {
                    self.whole_expression(t)
                } else if t == atom!(DOLLAR_DOLLAR) {
                    self.all_bindings(t)
                } else {
                    Ok(true)
                }  
            }
            // Fall through, immediate
            _ => Ok(true)
        }
    }

}

