#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m;
use bsp::hal;
use trinket_m0 as bsp;
use ws2812_nop_samd21 as ws2812;
use apa102_spi as apa102;

use bsp::clock::GenericClockController;
use bsp::delay::Delay;
use bsp::pac::{CorePeripherals, Peripherals};
use bsp::prelude::*;
use bsp::timer::TimerCounter;

use crate::apa102::Apa102;
use crate::ws2812::Ws2812;
use smart_leds::SmartLedsWrite;
use smart_leds_trait::{RGBW, RGB8, White};
// use cortex_m_rt::entry;

use bsp::entry;


use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use tpm2_rs::tpm2::{Tpm2Packet, Tpm2Type, TPM2_ACK, TPM2_START};

const NUM_LEDS: usize = 150;



#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = bsp::Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let neopixel_pin = pins.d4.into_push_pull_output(&mut pins.port);

    let bus_allocator = bsp::usb_allocator(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm,
        pins.usb_dp,
    );

    let mut serial = SerialPort::new(&bus_allocator);
    let mut usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

    let mut buff = [0u8; NUM_LEDS*3 + 5];
    let mut data = [0u8; NUM_LEDS*3 + 5];

    let mut ws = Ws2812::new_sk6812w(neopixel_pin);

    let mut colors: [RGBW<u8>; NUM_LEDS] = [RGBW {
        r: 100,
        g: 0,
        b: 0,
        a: White(0)
    }; NUM_LEDS];
    cortex_m::interrupt::free(|_| {
        ws.write(colors.iter().cloned()).ok();
    });
    let mut pkt_count: usize = 0;
    let mut offset: usize = 0;
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }
        if let Ok(count) = serial.read(&mut buff) {
            if pkt_count == 0 {
                offset = buff.iter().position(|&r| r == TPM2_START).unwrap_or(count);
            }
            for (i, x) in (pkt_count..pkt_count + count - offset).zip(offset..count) {
                if i < data.len() && x < buff.len() {
                    data[i] = buff[x];
                }
            }
            pkt_count += count;
        } else {
            continue;
        }

        if pkt_count >= buff.len() {
            pkt_count = 0;
            match Tpm2Packet::new(&data) {
                Ok(pkt) => {
                    match pkt.pkt_type {
                        Tpm2Type::Data => {
                            pkt.data.chunks(3).into_iter().zip(colors.iter_mut()).map(| (x, o) | {
                                if x.iter().all(|&item| item == x[0]) {
                                    *o = RGBW { r: 0, g: 0, b: 0, a: White(x[0]) };
                                } else {
                                    *o = RGBW{ r: x[0], g: x[1], b: x[2], a: White(0) };
                                }
                            }).count();
                            cortex_m::interrupt::free(|_| {
                                ws.write(colors.iter().cloned()).ok();
                            });
                        },
                        Tpm2Type::Cmd => {

                        },
                        Tpm2Type::Resp => {
                            serial.write(&NUM_LEDS.to_be_bytes()).ok();
                        }
                    }
                },
                Err(code) => {
                    serial.write(&data).ok();
                    serial.write(&[0xFF, 0xFF, code, 0xFF, 0xFF]).ok();
                    serial.flush().ok();
                    continue;
                }
            }
            serial.write(&[TPM2_ACK]).ok();
        }
    };
}
