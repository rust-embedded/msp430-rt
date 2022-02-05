#![no_main]

use msp430_rt_macros::entry;

#[entry]
extern "C" fn main() -> ! {
    loop {}
}
