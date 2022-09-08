#![no_std]

use core::convert::TryFrom;

use nom::{
    bytes::streaming::{tag, take_till},
    combinator::map,
    number::streaming::{be_u16, u8},
    sequence::{pair, preceded, tuple},
    IResult,
};

pub const TPM2_START: u8 = 0xC9;
pub const TPM2_END: u8 = 0x36;
pub const TPM2_ACK: u8 = 0xAC;

#[derive(Debug, PartialEq)]
pub enum TPM2Type {
    DATA = 0xDA,
    CMD = 0xC0,
    RESP = 0xAA,
}

impl TryFrom<u8> for TPM2Type {
    type Error = &'static str;
    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0xDA => Ok(TPM2Type::DATA),
            0xC0 => Ok(TPM2Type::CMD),
            0xAA => Ok(TPM2Type::RESP),
            _ => Err("Invalid TPM2Type"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TPM2Header {
    pub pkt_type: TPM2Type,
    pub size: u16,
}

/// Parses the header of a TPM2 packet, ignoring any data before the start of the packet
pub fn parse_tpm2_header(i: &[u8]) -> IResult<&[u8], TPM2Header> {
    preceded(
        tuple((take_till(|c| c == TPM2_START), tag(&[TPM2_START]))),
        map(pair(u8, be_u16), |(pkt_type, size)| TPM2Header {
            pkt_type: TPM2Type::try_from(pkt_type).expect("Invliad TPM2Type"),
            size,
        }),
    )(i)
}

/// Parses the data of a TPM2 packet, assuming that there is nothing preceding the data
pub fn parse_tpm2_data(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_till(|c| c == TPM2_END)(i)
}
