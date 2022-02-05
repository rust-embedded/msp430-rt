#![no_main]

use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main() -> ! {
    loop {}
}

#[interrupt(arg)]
fn TIM2() -> ! {
    loop {}
}
