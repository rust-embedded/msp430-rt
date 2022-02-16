#![no_main]

use msp430::interrupt::CriticalSection;
use msp430_rt_macros::{entry, interrupt};

static mut CS: Option<CriticalSection> = None;

#[entry]
fn main(cs: CriticalSection) -> ! {
    unsafe {
        CS = Some(cs);
    }
    loop {}
}

#[interrupt]
fn DefaultHandler(cs: CriticalSection) -> ! {
    unsafe {
        CS = Some(cs);
    }
    loop {}
}
