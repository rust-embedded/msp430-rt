#![no_main]

use msp430::interrupt::CriticalSection;
use msp430_rt_macros::entry;

fn init<'a>(cs: CriticalSection<'a>) -> CriticalSection<'a> {
    cs
}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main(_cs: CriticalSection) -> ! {
    loop {}
}
