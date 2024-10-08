mod addr;
mod parse;

pub use addr::Addr;
use derive_try_from_primitive::TryFromPrimitive;
use enumflags2::BitFlags;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{map, verify},
    error::context,
    number::complete::{le_u16, le_u32},
    sequence::tuple,
    Offset as _,
};
use std::{convert::TryFrom as _, fmt, ops::Range};

#[derive(Debug)]
pub struct File {
    pub typ: Type,
    pub machine: Machine,
    pub entry_point: Addr,
    pub program_headers: Vec<ProgramHeader>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum Type {
    None = 0x0,
    Rel = 0x1,
    Exec = 0x2,
    Dyn = 0x3,
    Core = 0x4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum Machine {
    X86 = 0x03,
    X86_64 = 0x3e,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum SegmentType {
    Null = 0x0,
    Load = 0x1,
    Dynamic = 0x2,
    Interp = 0x3,
    Note = 0x4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitFlags)]
#[repr(u32)]
pub enum SegmentFlag {
    Execute = 0x1,
    Write = 0x2,
    Read = 0x4,
}

pub struct ProgramHeader {
    pub r#type: SegmentType,
    pub flags: BitFlags<SegmentFlag>,
    pub offset: Addr,
    pub vaddr: Addr,
    pub paddr: Addr,
    pub filesz: Addr,
    pub memsz: Addr,
    pub align: Addr,
    pub data: Vec<u8>,
}

impl_parse_for_enum!(Type, le_u16);
impl_parse_for_enum!(Machine, le_u16);
impl_parse_for_enum!(SegmentType, le_u32);
impl_parse_for_enumflags!(SegmentFlag, le_u32);

impl File {
    const MAGIC: &'static [u8] = &[0x7f, 0x45, 0x4c, 0x46];

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let full_input = i;

        let (i, _) = tuple((
            context("Magic", tag(Self::MAGIC)),
            context("Class", tag(&[0x2])),
            context("Endianness", tag(&[0x1])),
            context("Version", tag(&[0x1])),
            context("OS ABI", alt((tag(&[0x0]), tag(&[0x3])))),
            context("Padding", take(8_usize)),
        ))(i)?;
        let (i, (typ, machine)) = tuple((Type::parse, Machine::parse))(i)?;
        let (i, _) = context("Version (bis)", verify(le_u32, |&x| x == 1))(i)?;
        let (i, entry_point) = Addr::parse(i)?;

        let u16_size = map(le_u16, |x| x as usize);
        let (i, (ph_offset, sh_offset)) = tuple((Addr::parse, Addr::parse))(i)?;
        let (i, (flags, hdr_size)) = tuple((le_u32, le_u16))(i)?;
        let (i, (ph_entsize, ph_count)) = tuple((&u16_size, &u16_size))(i)?;
        let (i, (sh_entsize, sh_count, sh_nidx)) = tuple((&u16_size, &u16_size, &u16_size))(i)?;

        let ph_slices = (&full_input[ph_offset.into()..]).chunks(ph_entsize);
        let mut program_headers = vec![];
        for ph_slice in ph_slices.take(ph_count) {
            let (_, ph) = ProgramHeader::parse(full_input, ph_slice)?;
            program_headers.push(ph);
        }

        let res = Self {
            machine,
            typ,
            entry_point,
            program_headers,
        };
        Ok((i, res))
    }

    pub fn parse_or_print_error(i: parse::Input) -> Option<Self> {
        match Self::parse(i) {
            Ok((_, file)) => Some(file),
            Err(nom::Err::Failure(err)) | Err(nom::Err::Error(err)) => {
                eprintln!("Parsing failed:");
                for (input, err) in err.errors {
                    let offset = i.offset(input);
                    eprintln!("{:?} at position {}:", err, offset);
                    eprintln!("{:>08x}: {:?}", offset, HexDump(input));
                }
                None
            }
            _ => panic!("unexpected nom error"),
        }
    }
}

impl ProgramHeader {
    pub fn file_range(&self) -> Range<Addr> {
        self.offset..self.offset + self.filesz
    }

    pub fn mem_range(&self) -> Range<Addr> {
        self.vaddr..self.vaddr + self.memsz
    }

    fn parse<'a>(full_input: parse::Input<'_>, i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, (r#type, flags)) = tuple((SegmentType::parse, SegmentFlag::parse))(i)?;
        let (i, (offset, vaddr, paddr, filesz, memsz, align)) = tuple((
            Addr::parse,
            Addr::parse,
            Addr::parse,
            Addr::parse,
            Addr::parse,
            Addr::parse,
        ))(i)?;

        let res = Self {
            r#type,
            flags,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
            data: full_input[offset.into()..][..filesz.into()].to_vec(),
        };
        Ok((i, res))
    }
}

impl fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "file {:?} | mem {:?} | align {:?} | {} {:?}",
            self.file_range(),
            self.mem_range(),
            self.align,
            &[
                (SegmentFlag::Read, 'R'),
                (SegmentFlag::Write, 'W'),
                (SegmentFlag::Execute, 'X')
            ]
            .iter()
            .map(|&(flag, letter)| {
                if self.flags.contains(flag) {
                    letter
                } else {
                    '.'
                }
            })
            .collect::<String>(),
            self.r#type,
        )
    }
}

// Do we really need a struct? Who not fn hexdump(&[u8])
pub struct HexDump<'a>(&'a [u8]);

impl<'a> fmt::Debug for HexDump<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &x in self.0.iter().take(20) {
            write!(f, "{:02x}", x)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_to_u16() {
        assert_eq!(Type::None as u16, 0x0);
        assert_eq!(Type::Dyn as u16, 0x3);
    }

    #[test]
    fn type_from_u16() {
        assert_eq!(Type::try_from(0x3), Ok(Type::Dyn));
        assert_eq!(Type::try_from(0xf00d), Err(0xf00d));
    }

    #[test]
    fn machine_convert() {
        assert_eq!(Machine::X86_64 as u16, 0x3e);
        assert_eq!(Machine::try_from(0x3e), Ok(Machine::X86_64));
        assert_eq!(Machine::try_from(0xfa), Err(0xfa));
    }

    #[test]
    fn try_bitflag() {
        use super::SegmentFlag;
        use enumflags2::BitFlags;

        let flags_integer: u32 = 6;
        let flags = BitFlags::<SegmentFlag>::from_bits(flags_integer).unwrap();
        assert_eq!(flags, SegmentFlag::Read | SegmentFlag::Write);
        assert_eq!(flags.bits(), flags_integer);
        assert!(BitFlags::<SegmentFlag>::from_bits(1992).is_err());
    }
}
