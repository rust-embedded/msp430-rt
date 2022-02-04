#![no_main]

extern crate msp430_rt_macros;
use msp430_rt_macros::entry;

#[entry]
fn main() -> ! {
    loop {}
}
