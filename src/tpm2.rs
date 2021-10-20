use num_traits::FromPrimitive;

pub const TPM2_START: u8 = 0xC9;
pub const TPM2_END: u8 = 0x36;
pub const TPM2_ACK: u8 = 0xAC;

pub struct Tpm2Packet<'a> {
    pub pkt_type: Tpm2Type,
    pub size: usize,
    pub data: &'a [u8]
}

#[derive(FromPrimitive, ToPrimitive)]
pub enum Tpm2Type {
    Data = 0xDA,
    Cmd = 0xC0,
    Resp = 0xAA,
}

impl<'a> Tpm2Packet<'a> {
    pub fn new(data: &'a [u8]) -> Result<Tpm2Packet, u8>{
        if data[0] != TPM2_START {
            return Err(1);
        }
        if data[data.len() - 1] != TPM2_END { return Err(2); }
        if data.len() <= 4 { return Err(3); }
        let pkt_type: Tpm2Type = match FromPrimitive::from_u8(data[1]) {
            Some(t) => t,
            None => return Err(data[1]),
        };
        let data_size: usize = ((data[2] as u16) << 8 | data[3] as u16).into();
        let pkt_data = &data[4..data_size+4];
        if data_size != data.len() - 5 {
            Err(5)
        } else {
            Ok(Tpm2Packet {
                pkt_type,
                size: data_size,
                data: pkt_data
            })
        }
    }
}
