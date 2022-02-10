#![no_main]

use msp430::interrupt::CriticalSection;
use msp430_rt_macros::entry;

fn init(cs: CriticalSection) -> u32 {
    32
}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main(_i: bool) -> ! {
    loop {}
}
