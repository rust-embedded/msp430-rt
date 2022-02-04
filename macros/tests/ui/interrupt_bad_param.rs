#![no_main]

use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main() -> ! {
    loop {}
}

#[interrupt]
fn TIM2(i: u32) -> ! {
    loop {}
}
