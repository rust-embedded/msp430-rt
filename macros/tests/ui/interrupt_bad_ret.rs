#![no_main]

use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main() -> ! {
    loop {}
}

#[interrupt]
fn TIM2() -> bool {
    true
}
