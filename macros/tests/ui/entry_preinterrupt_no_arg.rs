#![no_main]

use msp430_rt_macros::entry;

fn init() {}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main() -> ! {
    loop {}
}
