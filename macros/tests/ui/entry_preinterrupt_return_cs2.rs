#![no_main]

use msp430::interrupt::CriticalSection;
use msp430_rt_macros::entry;

fn init(cs: CriticalSection) -> CriticalSection {
    cs
}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main(_cs: CriticalSection) -> ! {
    loop {}
}
