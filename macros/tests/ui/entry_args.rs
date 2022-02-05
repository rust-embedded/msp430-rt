#![no_main]

use msp430_rt_macros::entry;

#[entry(arg)]
fn main() -> ! {
    loop {}
}
