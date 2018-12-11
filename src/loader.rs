use crate::atom::ATOMS;
use crate::etf;
use crate::module::{ErlFun, Lambda, Module};
use crate::opcodes::*;
use crate::value::Value;
use crate::vm::Machine;
use compress::zlib;
use nom::*;
use num_bigint::{BigInt, Sign};
use std::collections::HashMap;
use std::io::{Cursor, Read};

#[derive(Debug)]
pub struct Loader<'a> {
    pub vm: &'a Machine,
    atoms: Vec<&'a str>,
    imports: Vec<ErlFun>,
    exports: Vec<ErlFun>,
    literals: Vec<Value>,
    lambdas: Vec<Lambda>,
    atom_map: HashMap<usize, usize>, // TODO: remove this; local id -> global id
    funs: HashMap<(usize, usize), usize>, // (fun name as atom, arity) -> offset
    labels: HashMap<usize, usize>,   // label -> offset
    code: &'a [u8],
    instructions: Vec<Instruction>,
}

impl<'a> Loader<'a> {
    pub fn new(vm: &Machine) -> Loader {
        Loader {
            vm,
            atoms: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            literals: Vec::new(),
            lambdas: Vec::new(),
            atom_map: HashMap::new(),
            labels: HashMap::new(),
            funs: HashMap::new(),
            code: &[],
            instructions: Vec::new(),
        }
    }

    pub fn load_file(mut self, bytes: &'a [u8]) -> Result<Module, nom::Err<&[u8]>> {
        let (_, res) = scan_beam(bytes).unwrap();

        // parse all the chunks
        for chunk in res {
            match chunk.name.as_ref() {
                "AtU8" => self.load_atoms(chunk),
                "LocT" => self.load_local_fun_table(chunk), // can probably be ignored
                "ImpT" => self.load_imports_table(chunk),
                "ExpT" => self.load_exports_table(chunk),
                //"StrT" => self.load_strings_table(chunk),
                "LitT" => self.load_literals_table(chunk),
                "FunT" => self.load_funs_table(chunk),
                "Attr" => self.load_attributes(chunk),
                "Code" => self.load_code(chunk),
                name => println!("Unhandled chunk: {}", name),
            }
        }

        // parse the instructions, swapping for global vals
        // - swap load atoms with global atoms
        // - patch jump instructions to labels (store patches if label wasn't seen yet)
        // - make imports work via pointers..
        self.prepare();

        Ok(Module {
            atoms: self.atom_map,
            imports: self.imports,
            exports: self.exports,
            literals: self.literals,
            lambdas: self.lambdas,
            funs: self.funs,
            labels: self.labels,
            instructions: self.instructions,
        })
    }

    fn load_code(&mut self, chunk: Chunk<'a>) {
        let (_, data) = code_chunk(chunk.data).unwrap();
        self.code = data.code;
    }

    fn load_atoms(&mut self, chunk: Chunk<'a>) {
        let (_, atoms) = atom_chunk(chunk.data).unwrap();
        self.atoms = atoms;
    }

    fn load_attributes(&mut self, chunk: Chunk<'a>) {
        // Contains two parts: a proplist of module attributes, encoded as External Term Format,
        // and a compiler info (options and version) encoded similarly.
        let (rest, val) = etf::decode(chunk.data).unwrap();
        println!("attrs val: {:?}; rest: {:?}", val, rest);
    }

    // fn load_strings_table(&mut self, chunk: Chunk<'a>) {
    //     // println!("StrT {:?}", data);
    // }

    fn load_local_fun_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = loct_chunk(chunk.data).unwrap();
    }

    fn load_imports_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = loct_chunk(chunk.data).unwrap();
        self.imports = data;
    }

    fn load_exports_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = loct_chunk(chunk.data).unwrap();
        self.exports = data;
    }

    fn load_literals_table(&mut self, chunk: Chunk<'a>) {
        let (rest, size) = be_u32(chunk.data).unwrap();
        let mut data = Vec::with_capacity(size as usize);

        // Decompress deflated literal table
        let iocursor = Cursor::new(rest);
        zlib::Decoder::new(iocursor).read_to_end(&mut data).unwrap();
        let buf = &data[..];

        println!("raw literals: {:?}", data);

        assert_eq!(data.len(), size as usize, "LitT inflate failed");

        // self.literals.reserve(count as usize);

        // Decode literals into literal heap
        // pass in an allocator that allocates to a permanent non GC heap
        // TODO: probably GC'd when module is deallocated?
        // &self.literal_allocator
        let (_, literals) = decode_literals(buf).unwrap();
        self.literals = literals;
    }

    fn load_funs_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = funt_chunk(chunk.data).unwrap();
        self.lambdas = data;
    }

    // TODO: return a Module
    fn prepare(&mut self) {
        self.register_atoms();
        //self.stage2_fill_lambdas();

        self.postprocess_raw_code();
        ////unsafe { disasm::disasm(self.code.as_slice(), None) }
        //self.postprocess_fix_labels()?;
        //self.postprocess_setup_imports()?;
    }

    fn register_atoms(&mut self) {
        ATOMS.reserve(self.atoms.len());
        for (index, a) in self.atoms.iter().enumerate() {
            let g_index = ATOMS.register_atom(a);
            // keep a mapping of these to patch the instrs
            self.atom_map.insert(index, g_index);
        }

        // Create a new version number for this module and fill self.mod_id
        // self.set_mod_id(code_server)
    }

    fn postprocess_raw_code(&mut self) {
        // scan over the Code chunk bits, but we'll need to know bit length of each instruction.
        let (_, code) = scan_instructions(self.code).unwrap();
        for instruction in code {
            match &instruction.op {
                Opcode::Line => continue, // skip for now
                Opcode::Label => {
                    // one operand, Integer
                    if let Value::Literal(i) = instruction.args[0] {
                        // Store weak ptr to function and code offset to this label
                        // let floc = self.code.len(); // current byte length
                        self.labels.insert(i as usize, self.instructions.len());
                    } else {
                        // op_badarg_panic(op, &args, 0);
                        panic!("Bad argument to {:?}", instruction.op)
                    }
                    continue;
                }
                Opcode::FuncInfo => {
                    // record function data M:F/A
                    // don't skip so we can apply tracing during runtime
                    if let [_module, Value::Atom(f), Value::Literal(a)] = &instruction.args[..] {
                        self.funs
                            .insert((*f as usize, *a as usize), self.instructions.len());
                    } else {
                        panic!("Bad argument to {:?}", instruction.op)
                    }
                }
                Opcode::IntCodeEnd => {
                    println!("Finished processing instructions");
                    break;
                }
                _ => {}
            }

            self.instructions.push(instruction);
        }
    }
}

named!(
    decode_literals<&[u8], Vec<Value>>,
    do_parse!(
        _count: be_u32 >>
        literals: many0!(complete!(
            do_parse!(
                _size: be_u32 >>
                literal: call!(etf::decode) >>
                (literal)
            )
        )) >>
        (literals)
    )
);

#[derive(Debug, PartialEq)]
pub struct Chunk<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

named!(
    pub scan_beam<&[u8], Vec<Chunk>>,
    do_parse!(
        tag!("FOR1") >>
        _size: le_u32 >>
        tag!("BEAM") >>

        data: chunks >>
        (data)
    )
);

named!(
    chunks<&[u8], Vec<Chunk>>,
    many0!(complete!(chunk))
);

named!(
    chunk<&[u8], Chunk>,
    do_parse!(
        name: map_res!(take!(4), std::str::from_utf8) >>
        size: be_u32 >>
        bytes: take!(size) >>
        take!(align_bytes(size)) >>
        (Chunk { name, data: bytes })
    )
);

fn align_bytes(size: u32) -> u32 {
    let rem = size % 4;
    if rem == 0 {
        0
    } else {
        4 - rem
    }
}

#[derive(Debug)]
pub struct CodeChunk<'a> {
    pub sub_size: u32,   // prefixed extra field data
    pub version: u32,    // should be 0
    pub opcode_max: u32, // highest opcode used for versioning
    pub labels: u32,     // number of labels
    pub functions: u32,  // number of functions
    pub code: &'a [u8],
}

named!(
    code_chunk<&[u8], CodeChunk>,
    do_parse!(
        sub_size: be_u32 >>
        version: be_u32 >>
        opcode_max: be_u32 >>
        labels: be_u32 >>
        functions: be_u32 >>
        //take!(sub_size) >> offset from the header
        code: rest >>
        (CodeChunk { sub_size, version, opcode_max, labels, functions, code })
    )
);

named!(
    atom_chunk<&[u8], Vec<&str>>,
    do_parse!(
        count: be_u32 >>
        // TODO figure out if we can prealloc the size
        atoms: count!(map_res!(length_bytes!(be_u8), std::str::from_utf8), count as usize) >>
        (atoms)
    )
);

named!(
    fun_entry<&[u8], ErlFun>,
    do_parse!(
        function: be_u32 >>
        arity: be_u32 >>
        label: be_u32 >>
        (function, arity, label)
    )
);

named!(
    loct_chunk<&[u8], Vec<ErlFun>>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (entries)
    )
);

named!(
    litt_chunk<&[u8], Vec<ErlFun>>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (entries)
    )
);

named!(
    funt_chunk<&[u8], Vec<Lambda>>,
    do_parse!(
        count: be_u32 >>
        entries: count!(do_parse!(
            name: be_u32 >>
            arity: be_u32 >>
            offset: be_u32 >>
            index: be_u32 >>
            nfree: be_u32 >>
            ouniq: be_u32 >>
            (Lambda { name, arity, offset, index, nfree, ouniq })
            )
        , count as usize) >>
        (entries)
    )
);

// It can be Literal=0, Integer=1, Atom=2, XRegister=3, YRegister=4, Label=5, Character=6, Extended=7.
// If the base tag was Extended=7, then bits 4-5-6-7 PLUS 7 will become the extended tag.
// It can have values Float=8, List=9, FloatReg=10, AllocList=11, Literal=12.

fn read_int(b: u8, rest: &[u8]) -> IResult<&[u8], u64> {
    // it's not extended
    if 0 == (b & 0b1000) {
        // Bit 3 is 0 marks that 4 following bits contain the value
        return Ok((rest, (b >> 4) as u64));
    }

    // Bit 3 is 1, but...
    if 0 == (b & 0b1_0000) {
        // Bit 4 is 0, marks that the following 3 bits (most significant) and
        // the following byte (least significant) will contain the 11-bit value
        let (rest, r) = be_u8(rest)?;
        Ok((rest, ((b & 0b1110_0000) << 3 | r) as u64))
    } else {
        // Bit 4 is 1 means that bits 5-6-7 contain amount of bytes+2 to store
        // the value
        let mut n_bytes = (b >> 5) + 2;
        if n_bytes == 9 {
            println!("more than 9!")
            //     // bytes=9 means upper 5 bits were set to 1, special case 0b11111xxx
            //     // which means that following nested tagged value encodes size,
            //     // followed by the bytes (Size+9)
            //     let bnext = r.read_u8();
            //     if let Integral::Small(tmp) = read_word(bnext, r) {
            //       n_bytes = tmp as Word + 9;
            //     } else {
            //       panic!("{}read word encountered a wrong byte length", module())
            //     }
        }

        // Read the remaining big endian bytes and convert to int
        let (rest, long_bytes) = take!(rest, n_bytes)?;
        let sign = if long_bytes[0] & 0x80 == 0x80 {
            Sign::Minus
        } else {
            Sign::Plus
        };

        let r = BigInt::from_bytes_be(sign, &long_bytes);
        println!("{}", r);
        //Integral::from_big(r)
        Ok((rest, 23))
    } // if larger than 11 bits
}

fn compact_term(i: &[u8]) -> IResult<&[u8], Value> {
    let (rest, b) = be_u8(i)?;
    let tag = b & 0b111;

    if tag < 0b111 {
        let (rest, val) = read_int(b, rest).unwrap();

        return match tag {
            0 => Ok((rest, Value::Literal(val as u64))),
            1 => Ok((rest, Value::Integer(val as u64))),
            2 => Ok((rest, Value::Atom(val as usize))),
            3 => Ok((rest, Value::X(val))),
            4 => Ok((rest, Value::Y(val))),
            5 => Ok((rest, Value::Label(val))),
            6 => Ok((rest, Value::Character(val as i64))),
            _ => panic!("can't happen"),
        };
    }

    parse_extended_term(b, rest)
}

fn parse_extended_term(b: u8, rest: &[u8]) -> IResult<&[u8], Value> {
    match b {
        0b0001_0111 => parse_list(rest),
        0b0010_0111 => parse_float_reg(rest),
        0b0011_0111 => parse_alloc_list(rest),
        0b0100_0111 => parse_extended_literal(rest),
        _ => panic!("can't happen"),
    }
}

fn parse_list(rest: &[u8]) -> IResult<&[u8], Value> {
    // The stream now contains a smallint size, then size/2 pairs of values
    let (mut rest, n) = be_u8(rest)?;
    let mut els = Vec::with_capacity(n as usize);

    for _i in 0..n {
        let (new_rest, term) = compact_term(rest)?;
        els.push(term);
        rest = new_rest;
    }

    Ok((rest, Value::List(Box::new(els))))
}

fn parse_float_reg(rest: &[u8]) -> IResult<&[u8], Value> {
    Ok((rest, Value::FloatReg(22 as u64)))
}

fn parse_alloc_list(rest: &[u8]) -> IResult<&[u8], Value> {
    Ok((rest, Value::AllocList(22 as u64)))
}

fn parse_extended_literal(rest: &[u8]) -> IResult<&[u8], Value> {
    let (rest, b) = be_u8(rest)?;
    let (rest, val) = read_int(b, rest).unwrap();
    Ok((rest, Value::ExtendedLiteral(val)))
}

// ---------------------------------------------------

#[derive(Debug)]
pub struct Instruction {
    pub op: Opcode,
    pub args: Vec<Value>,
}

named!(
    pub scan_instructions<&[u8], Vec<Instruction>>,
    many0!(complete!(scan_instruction))
);

// apply compact_term, arity times (keep an arity table)
named!(
    scan_instruction<&[u8], Instruction>,
    do_parse!(
        op: be_u8 >>
        args: count!(compact_term, opcode_arity(op)) >>
        (Instruction { op: to_opcode(op), args })
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compact_term() {
        assert_eq!(
            compact_term(&vec![0b10010000u8]),
            Ok((&[] as &[u8], 9 as u8))
        );
        assert_eq!(
            compact_term(&vec![0b11110000u8]),
            Ok((&[] as &[u8], 15 as u8))
        );
    }
}
