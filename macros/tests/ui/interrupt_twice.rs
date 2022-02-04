#![no_main]
#![feature(abi_msp430_interrupt)]

use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main() -> ! {
    loop {}
}

#[allow(non_camel_case_types)]
enum interrupt {
    TIM1,
    TIM2,
}

#[interrupt]
fn TIM1() {
    // Different name, so it shouldn't be flagged
    loop {}
}

#[interrupt]
fn TIM2() {
    loop {}
}

#[interrupt]
fn TIM2() {
    loop {}
}
