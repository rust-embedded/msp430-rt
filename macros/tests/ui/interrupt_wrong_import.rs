#![no_main]

// Incorrect w/ bad error message- use of undeclared crate or module `interrupt`.
use msp430_rt_macros::{entry, interrupt};

#[entry]
fn main() -> ! {
    unimplemented!()
}

#[interrupt]
fn TIM2() {
    unimplemented!()
}
