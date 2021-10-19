#![no_std]
#![no_main]
#![feature(more_qualified_paths)]

use panic_halt as _;

use bsp::hal;
use trinket_m0 as bsp;
use ws2812_spi as ws2812;
// use apa102_spi as apa102;

use bsp::clock::GenericClockController;
use bsp::delay::Delay;
use bsp::pac::{CorePeripherals, Peripherals};
use bsp::prelude::*;
use bsp::timer::TimerCounter;

// use crate::apa102::Apa102;
use crate::ws2812::Ws2812;
use crate::ws2812::devices;
use smart_leds::SmartLedsWrite;
use smart_leds_trait::{RGBW, RGB8, RGBA, White};
// use cortex_m_rt::entry;

use bsp::entry;


use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use tpm2_rs::tpm2::Tpm2;

const NUM_LEDS: usize = 90;

// fn my_callback<S: SmartLedsWrite>(strip: S) -> impl FnMut(&[u8]) -> &[u8]
//     where
//         <S as SmartLedsWrite>::Color: From<RGBW<u8>>,
//         // S: SmartLedsWrite<Error = Debug>
//
// {
//     | data: &[u8] | {
//         let colors = data.chunks(3).into_iter().map(|x| {
//             <Ws2812<_, devices::Sk6812w> as SmartLedsWrite>::Color{ r: x[0], g: x[1], b: x[2], a: White(0) }
//         });
//         strip.write(colors).ok();
//         // strip.write(data.chunks(3).map(|x| -> RGBA<u8> {
//         //     tmp[..].copy_from_slice(x);
//         //     RGB8::from(tmp).alpha(0)
//         // }).cloned())
//         //   .unwrap();
//             // strip.write([data].iter().cloned()).unwrap();
//         &[] as &[u8]
//         // dotstar.write([RGB8 { r: data[0], g: data[1], b: data[2] }].iter().cloned()).unwrap();
//     }
// }


#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = bsp::Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    // let di = pins.dotstar_di.into_push_pull_output(&mut pins.port);
    // let ci = pins.dotstar_ci.into_push_pull_output(&mut pins.port);
    // let nc = pins.dotstar_nc.into_floating_input(&mut pins.port);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.PM);
    timer.start(5.khz());

    let spi = bsp::spi_master(
        &mut clocks,
        3.mhz(),
        peripherals.SERCOM0,
        &mut peripherals.PM,
        pins.d3,
        pins.d4,
        pins.d2,
        &mut pins.port,
    );

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

    let mut ws = Ws2812::new_sk6812w(spi);

    // let spi = bitbang_hal::spi::SPI::new(apa102_spi::MODE, nc, di, ci, timer);
    //
    // let mut dotstar = Apa102::new(spi);

    let size = buff.len();

    let my_cb = | data: &[u8] | -> Option<&[u8]> {
        let colors = data.chunks(3).into_iter().map(|x| {
            RGBW{ r: x[0], g: x[1], b: x[2], a: White(0) }
        });
        ws.write(colors).ok();
        None
    };

    let mut tpm2 = Tpm2::new(&mut serial, &mut usb_dev, &mut buff, size, Some(my_cb), None, None);

    loop {
        tpm2.update().ok();
    };
}

// static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
// static mut TPM2_DRIVER: Option<TPM2<UsbBus>> = None;
//
// fn poll_usb() {
//     unsafe {
//         if let Some(tpm) = TPM2_DRIVER  {
//             tpm.update();
//         }
//     };
// }
//
// #[interrupt]
// fn USB() {
//     poll_usb();
// }

// fn update_light()
