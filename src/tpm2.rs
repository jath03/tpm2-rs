use usbd_serial::SerialPort;

use usb_device::bus::UsbBus;
use usb_device::prelude::*;

use int_enum::IntEnum;

const TPM2_START: u8 = 0xC9;
const TPM2_END: u8 = 0x36;
const TPM2_ACK: u8 = 0xAC;


#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum Tpm2Type {
    Data = 0xDA,
    Cmd = 0xC0,
    Resp = 0xAA,
}


pub struct Tpm2<'a, B: UsbBus, CB: FnMut(&[u8]) -> Option<&[u8]>> {
    serial: &'a mut SerialPort<'a, B>,
    usb_dev: &'a mut UsbDevice<'a, B>,
    data_callback: Option<CB>,
    command_callback: Option<CB>,
    response_callback: Option<CB>,
    buffer: &'a mut [u8],
}

impl<'a, B: UsbBus, CB: FnMut(&[u8]) -> Option<&[u8]>> Tpm2<'a, B, CB> {
    pub fn new(serial: &'a mut SerialPort<'a, B>, usb_dev: &'a mut UsbDevice<'a, B>, buffer: &'a mut [u8], data_size: usize, data_callback: Option<CB>, command_callback: Option<CB>, response_callback: Option<CB>) -> Tpm2<'a, B, CB> {
        assert_eq!(data_size + 5, buffer.len());
        Tpm2 {
            serial,
            usb_dev,
            data_callback,
            command_callback,
            response_callback,
            buffer,
        }
    }
    pub fn update<'b>(&mut self) -> Result<(), UsbError>{
        if !self.usb_dev.poll(&mut [self.serial]) {
            return Ok(());
        }
        let count = self.serial.read(&mut self.buffer)?;
        if count <= 5 || self.buffer[count - 1] != TPM2_END || self.buffer[0] != TPM2_START {
            return Err(UsbError::WouldBlock);
        }
        let mut out: Option<&[u8]> = None;
        if self.buffer[0] == TPM2_START {
            if let Some(pkt_type) = Tpm2Type::from_int(self.buffer[1]).ok() {
                let data_size: usize = ((self.buffer[2] as u16) << 8 | self.buffer[3] as u16).into();
                if count >= data_size {
                    // out = if let Some(cb) = self.get_callback(&pkt_type) {
                    let f = match pkt_type {
                        Tpm2Type::Data => &mut self.data_callback,
                        Tpm2Type::Cmd => &mut self.command_callback,
                        Tpm2Type::Resp => &mut self.response_callback,
                    };
                    out = if let Some(cb) = f {
                        (cb)(&self.buffer[4..data_size + 4])
                        // Some(&[0u8])
                    } else {
                        None
                    };
                }
            }
        }
        self.serial.write(&[TPM2_ACK]).ok();
        if let Some(output) = out {
            self.serial.write(output).ok();
        }
        Ok(())
    }

    // fn get_callback(&mut self, pkt_type: &Tpm2Type) -> Option<&mut CB> {
    //     match pkt_type {
    //         Tpm2Type::Data => self.data_callback.as_mut(),
    //         Tpm2Type::Cmd => self.command_callback.as_mut(),
    //         Tpm2Type::Resp => self.response_callback.as_mut(),
    //     }
    // }

}
