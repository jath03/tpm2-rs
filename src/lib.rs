#![no_std]

use usbd_serial::SerialPort;

use usb_device::bus::UsbBus;
use usb_device::prelude::*;

const TPM2_START: u8 = 0xC9;
const TPM2_END: u8 = 0x36;
const TPM2_ACK: u8 = 0xAC;
const TPM2_TYPE_DATA: u8 = 0xDA;
const TPM2_TYPE_CMD: u8 = 0xC0;
const TPM2_TYPE_RESP: u8 = 0xAA;


pub struct TPM2<'a, B: UsbBus, CB: FnMut(&[u8])> {
    serial: &'a mut SerialPort<'a, B>,
    usb_dev: &'a mut UsbDevice<'a, B>,
    data_callback: CB,
    buffer: &'a mut [u8],
}

impl<'a, B: UsbBus, CB: FnMut(&[u8])> TPM2<'a, B, CB> {
    pub fn new(serial: &'a mut SerialPort<'a, B>, usb_dev: &'a mut UsbDevice<'a, B>, data_callback: CB, buffer: &'a mut [u8], data_size: usize) -> TPM2<'a, B, CB> {
        assert_eq!(data_size + 5, buffer.len());
        TPM2 {
            serial,
            usb_dev,
            data_callback,
            buffer,
        }
    }
    pub fn update(&mut self) -> Result<(), UsbError>{
        if !self.usb_dev.poll(&mut [self.serial]) {
            return Ok(());
        }
        let count = self.serial.read(&mut self.buffer)?;
        if count <= 5 || self.buffer[count - 1] != TPM2_END {
            return Err(UsbError::WouldBlock);
        }

        if self.buffer[0] == TPM2_START {
            match self.buffer[1] {
                TPM2_TYPE_DATA => {
                    let data_size: usize = ((self.buffer[2] as u16) << 8 | self.buffer[3] as u16).into();
                    if count > data_size {
                        (self.data_callback)(&self.buffer[4..data_size + 4]);
                    }
                },
                TPM2_TYPE_CMD => {},
                TPM2_TYPE_RESP => {},
                _ => {}
            }
        }
        self.serial.write(&[TPM2_ACK]).ok();
        Ok(())
    }

}
