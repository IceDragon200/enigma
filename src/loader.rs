use crate::etf;
use crate::opcodes::*;
use crate::value::Value;
use crate::vm::Machine;
use compress::zlib;
use nom::*;
use num_bigint::{BigInt, Sign};
use std::io::{Cursor, Read};

#[derive(Debug)]
pub struct Loader<'a> {
    pub vm: &'a Machine,
    atoms: Vec<&'a str>,
    imports: Vec<ErlFun>,
    exports: Vec<ErlFun>,
    literals: Vec<Value>,
}

impl<'a> Loader<'a> {
    pub fn new(vm: &Machine) -> Loader {
        Loader {
            vm,
            atoms: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            literals: Vec::new(),
        }
    }

    pub fn load_file(&mut self, bytes: &'a [u8]) -> Result<String, nom::Err<&[u8]>> {
        let (_, res) = scan_beam(bytes).unwrap();

        // let names: Vec<_> = res.iter().map(|chunk| chunk.name).collect();
        // println!("{:?}", names);

        for chunk in res {
            match chunk.name.as_ref() {
                "AtU8" => self.load_atoms(chunk),
                "LocT" => self.load_local_fun_table(chunk),
                "ImpT" => self.load_imports_table(chunk),
                "ExpT" => self.load_exports_table(chunk),
                "LitT" => self.load_literals_table(chunk),
                name => println!("Unhandled chunk: {}", name),
                // let chunk = res.iter().find(|chunk| chunk.name == "Code").unwrap();
                // let code = code_chunk(chunk.data)?;
                // println!("{:?}", code.1);
            }
        }

        println!("{:?}", self);

        self.prepare();
        // parse all the chunks
        // load all the atoms, lambda funcs and literals into the VM and store the vm vals

        // parse the instructions, swapping for global vals
        // - swap load atoms with global atoms
        // - skip line
        // - store labels as offsets
        // - patch jump instructions to labels (store patches if label wasn't seen yet)
        // - make imports work via pointers..

        return Ok(String::from("OK"));
    }

    fn load_atoms(&mut self, chunk: Chunk<'a>) {
        let (_, data) = atom_chunk(chunk.data).unwrap();
        println!("{:?}", data);
        self.atoms = data.atoms;
    }

    fn load_local_fun_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = loct_chunk(chunk.data).unwrap();
        println!("LocT {:?}", data);
    }

    fn load_imports_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = loct_chunk(chunk.data).unwrap();
        println!("ImpT {:?}", data);
        self.imports = data.entries;
    }

    fn load_exports_table(&mut self, chunk: Chunk<'a>) {
        let (_, data) = loct_chunk(chunk.data).unwrap();
        println!("ExpT {:?}", data);
        self.exports = data.entries;
    }

    fn load_literals_table(&mut self, chunk: Chunk<'a>) {
        let (rest, size) = be_u32(chunk.data).unwrap();
        let mut data = Vec::with_capacity(size as usize);

        // Decompress deflated literal table
        let iocursor = Cursor::new(rest);
        zlib::Decoder::new(iocursor).read_to_end(&mut data).unwrap();
        let buf = &data[..];

        println!("{:?}", data);

        assert_eq!(data.len(), size as usize, "LitT inflate failed");

        // self.literals.reserve(count as usize);

        // Decode literals into literal heap
        // pass in an allocator that allocates to a permanent non GC heap
        // TODO: probably GC'd when module is deallocated?
        // &self.literal_allocator
        let (_, literals) = decode_literals(buf).unwrap();
        self.literals = literals;
        println!("{:?}", self.literals);

        println!("LitT {:?}", data);
    }

    // TODO: return a Module
    fn prepare(&mut self) {
        self.register_atoms();
    }

    fn register_atoms(&self) {
        self.vm.atom_table.reserve(self.atoms.len());
        for a in &self.atoms {
            self.vm.atom_table.register_atom(a);
        }
        // keep a mapping of these to patch the instrs

        // Create a new version number for this module and fill self.mod_id
        // self.set_mod_id(code_server)
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
        //take!(sub_size) >>
        code: rest >>
        (CodeChunk { sub_size, version, opcode_max, labels, functions, code })
    )
);

#[derive(Debug)]
pub struct AtomChunk<'a> {
    pub count: u32,
    pub atoms: Vec<&'a str>,
}

named!(
    atom_chunk<&[u8], AtomChunk>,
    do_parse!(
        count: be_u32 >>
        // TODO figure out if we can prealloc the size
        atoms: count!(map_res!(length_bytes!(be_u8), std::str::from_utf8), count as usize) >>
        (AtomChunk { count, atoms })
    )
);

type ErlFun = (u32, u32, u32);

named!(
    fun_entry<&[u8], ErlFun>,
    do_parse!(
        function: be_u32 >>
        arity: be_u32 >>
        label: be_u32 >>
        (function, arity, label)
    )
);

#[derive(Debug)]
pub struct FunTableChunk {
    pub count: u32,
    pub entries: Vec<ErlFun>,
}

named!(
    loct_chunk<&[u8], FunTableChunk>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (FunTableChunk { count, entries })
    )
);

named!(
    litt_chunk<&[u8], FunTableChunk>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (FunTableChunk { count, entries })
    )
);

// "StrT", "ImpT", "ExpT", "LocT", "Attr", "CInf", "Dbgi", "Line"

// It can be Literal=0, Integer=1, Atom=2, XRegister=3, YRegister=4, Label=5, Character=6, Extended=7.
// If the base tag was Extended=7, then bits 4-5-6-7 PLUS 7 will become the extended tag.
// It can have values Float=8, List=9, FloatReg=10, AllocList=11, Literal=12.

#[derive(Debug)]
pub enum Term {
    Literal(u64),
    Integer(u64),
    Atom(u64),
    X(u64),
    Y(u64),
    Label(u64),
    Character(u64),
    // Extended,
    Float(),
    List(Vec<Term>),
    FloatReg(u64),
    AllocList(u64),
    ExtendedLiteral(u64),
}

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

fn compact_term(i: &[u8]) -> IResult<&[u8], Term> {
    let (rest, b) = be_u8(i)?;
    let tag = b & 0b111;

    if tag < 0b111 {
        let (rest, val) = read_int(b, rest).unwrap();

        return match tag {
            0 => Ok((rest, Term::Literal(val as u64))),
            1 => Ok((rest, Term::Integer(val as u64))),
            2 => Ok((rest, Term::Atom(val))),
            3 => Ok((rest, Term::X(val))),
            4 => Ok((rest, Term::Y(val))),
            5 => Ok((rest, Term::Label(val))),
            6 => Ok((rest, Term::Character(val))),
            _ => panic!("can't happen"),
        };
    }

    parse_extended_term(b, rest)
}

fn parse_extended_term(b: u8, rest: &[u8]) -> IResult<&[u8], Term> {
    match b {
        0b0001_0111 => parse_list(rest),
        0b0010_0111 => parse_float_reg(rest),
        0b0011_0111 => parse_alloc_list(rest),
        0b0100_0111 => parse_extended_literal(rest),
        _ => panic!("can't happen"),
    }
}

fn parse_list(rest: &[u8]) -> IResult<&[u8], Term> {
    // The stream now contains a smallint size, then size/2 pairs of values
    let (mut rest, n) = be_u8(rest)?;
    let mut els = Vec::with_capacity(n as usize);

    for _i in 0..n {
        let (new_rest, term) = compact_term(rest)?;
        els.push(term);
        rest = new_rest;
    }

    Ok((rest, Term::List(els)))
}

fn parse_float_reg(rest: &[u8]) -> IResult<&[u8], Term> {
    Ok((rest, Term::FloatReg(22 as u64)))
}

fn parse_alloc_list(rest: &[u8]) -> IResult<&[u8], Term> {
    Ok((rest, Term::AllocList(22 as u64)))
}

fn parse_extended_literal(rest: &[u8]) -> IResult<&[u8], Term> {
    let (rest, b) = be_u8(rest)?;
    let (rest, val) = read_int(b, rest).unwrap();
    Ok((rest, Term::ExtendedLiteral(val)))
}

// ---------------------------------------------------

#[derive(Debug)]
pub struct Instruction {
    pub op: Opcode,
    pub args: Vec<Term>,
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
