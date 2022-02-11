#![no_main]

use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main(cs: CriticalSection) -> ! {
    main()
}

#[interrupt]
fn DefaultHandler(cs: CriticalSection) -> ! {
    DefaultHandler()
}
