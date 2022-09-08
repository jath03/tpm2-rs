use nom::{
    bytes, character,
    error::{Error, ErrorKind},
    Err, IResult, Needed,
};
use tpm2_rs::*;

const data: [u8; 11] = [
    0x00u8, 0x05, 0x34, 0xC9, 0xDA, 0x00, 0x03, 0xFF, 0xFF, 0xFF, 0x36,
];

const header: TPM2Header = TPM2Header {
    pkt_type: TPM2Type::DATA,
    size: 3,
};

#[test]
fn parse_header_test() {
    assert_eq!(
        parse_tpm2_header(&data),
        Ok((&[0xFF, 0xFF, 0xFF, 0x36][..], header))
    );
}

#[test]
fn parse_data_test() {
    assert_eq!(
        parse_tpm2_data(&data[7..]),
        Ok((&[0x36][..], &[0xFF, 0xFF, 0xFF][..]))
    );
}

#[test]
fn parse_incomplete_header_test() {
    assert_eq!(
        parse_tpm2_header(&data[..5]),
        Err(Err::Incomplete(Needed::new(2)))
    );
}
