mod parse;

use derive_try_from_primitive::TryFromPrimitive;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    error::context,
    sequence::tuple,
    Offset,
};
use std::{convert::TryFrom as _, fmt};

#[derive(Debug)]
pub struct File {
    pub typ: Type,
    pub machine: Machine,
}

impl File {
    const MAGIC: &'static [u8] = &[0x7f, 0x45, 0x4c, 0x46];

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, _) = tuple((
            context("Magic", tag(Self::MAGIC)),
            context("Class", tag(&[0x2])),
            context("Endianness", tag(&[0x1])),
            context("Version", tag(&[0x1])),
            context("OS ABI", alt((tag(&[0x0]), tag(&[0x3])))),
            context("Padding", take(8_usize)),
        ))(i)?;
        let (i, (typ, machine)) = tuple((Type::parse, Machine::parse))(i)?;

        let res = Self { machine, typ };
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

impl_parse_for_enum!(Type, le_u16);
impl_parse_for_enum!(Machine, le_u16);

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
}
