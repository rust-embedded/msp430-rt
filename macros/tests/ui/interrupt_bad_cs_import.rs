#![no_main]

use msp430::interrupt::CriticalSection as CritSec;
use msp430_rt_macros::{entry, interrupt};

// If we use "msp430::interrupt::CriticalSection", we get unused import warning for CritSec,
// which is correct but misleading b/c of interrupt sig requirements.
fn arg(_cs: msp430::interrupt::CriticalSection) {
    /* initialize hardware */
}

#[entry]
fn main() -> ! {
    unimplemented!()
}

#[interrupt]
fn TIM2(_cs: CritSec) {
    unimplemented!()
}
