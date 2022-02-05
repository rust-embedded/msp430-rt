#![no_main]

use msp430_rt_macros::{entry, pre_init};

#[entry]
fn bar() -> ! {
    loop {}
}

#[pre_init(arg)]
unsafe fn foo() {}
