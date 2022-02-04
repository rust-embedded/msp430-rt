#![no_main]

use msp430_rt_macros::{entry, pre_init};

#[entry]
fn bar() -> ! {
    loop {}
}

#[pre_init]
unsafe fn foo() -> u32 {
    3
}
