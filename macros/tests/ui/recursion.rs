#![no_main]

use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main(cs: CriticalSection) -> ! {
    main(cs)
}

#[interrupt]
fn DefaultHandler(cs: CriticalSection) -> ! {
    DefaultHandler(cs)
}
