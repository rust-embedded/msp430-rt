#![no_main]

use msp430::interrupt::CriticalSection;
use msp430_rt_macros::entry;

fn init(cs: &'static CriticalSection<'static>) -> &'static CriticalSection<'static> {
    cs
}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main(_cs: &CriticalSection) -> ! {
    loop {}
}
