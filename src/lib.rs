//! Minimal startup / runtime for MSP430 microcontrollers
//!
//! This crate is based on [cortex-m-rt](https://docs.rs/cortex-m-rt)
//! crate by Jorge Aparicio (@japaric).
//!
//! # Features
//!
//! This crate provides
//!
//! - Before main initialization of the `.bss` and `.data` sections.
//!
//! - A `panic_fmt` implementation that just calls abort that you can opt into
//!   through the "abort-on-panic" Cargo feature. If you don't use this feature
//!   you'll have to provide the `panic_fmt` lang item yourself. Documentation
//!   [here](https://doc.rust-lang.org/unstable-book/language-features/lang-items.html)
//!
//! - A minimal `start` lang item to support the standard `fn main()`
//!   interface. (NOTE: The processor goes into infinite loop after
//!   returning from `main`)
//!
//! - A linker script that encodes the memory layout of a generic MSP430
//!   microcontroller. This linker script is missing some information that must
//!   be supplied through a `memory.x` file (see example below).
//!
//! - A default exception handler that can be overridden using the
//!   [`default_handler!`](macro.default_handler.html) macro.
//!
//! - A `_sheap` symbol at whose address you can locate a heap.
//!
//! # Example
//!
//! Creating a new bare metal project. (I recommend you use the
//! [`msp430-quickstart`][qs] template as it takes of all the boilerplate
//! shown here)
//!
//! [qs]: https://github.com/japaric/msp430-quickstart/
//!
//! ``` text
//! $ cargo new --bin app && cd $_
//!
//! $ # add this crate as a dependency
//! $ edit Cargo.toml && cat $_
//! [dependencies.msp430-rt]
//! features = ["abort-on-panic"]
//! version = "0.1.0"
//!
//! $ # tell Xargo which standard crates to build
//! $ edit Xargo.toml && cat $_
//! [dependencies.core]
//! stage = 0
//!
//! [dependencies.compiler_builtins]
//! features = ["mem"]
//! stage = 1
//!
//! $ # memory layout of the device
//! $ edit memory.x && cat $_
//! MEMORY
//! {
//!   RAM              : ORIGIN = 0x0200, LENGTH = 0x0200
//!   ROM              : ORIGIN = 0xC000, LENGTH = 0x3FDE
//!   VECTORS          : ORIGIN = 0xFFE0, LENGTH = 0x0020
//! }
//!
//! $ edit src/main.rs && cat $_
//! ```
//!
//! ``` ignore,no_run
//! #![feature(used)]
//! #![no_std]
//!
//! extern crate msp430_rt;
//!
//! fn main() {
//!     // do something here
//! }
//!
//! // As we are not using interrupts, we just register a dummy catch all
//! // handler
//! #[link_section = ".vector_table.interrupts"]
//! #[used]
//! static INTERRUPTS: [extern "msp430-interrupt" fn(); 15] =
//!     [default_handler; 15];
//!
//! extern "msp430-interrupt" fn default_handler() {
//!     loop {
//!     }
//! }
//! ```
//!
//! ``` text
//! $ cargo install xargo
//!
//! $ xargo rustc --target msp430-none-elf --release -- \
//!       -C link-arg=-Tlink.x \
//!       -C link-arg=-mmcu=msp430g2553 -C link-arg=-nostartfiles \
//!       -C linker=msp430-elf-gcc -Z linker-flavor=gcc
//!
//! $ msp430-elf-objdump -Cd $(find target -name app) | head
//!
//! Disassembly of section .text:
//!
//! 0000c000 <msp430_rt::reset_handler::h77ef04785a7efdda>:
//!     c000:	31 40 00 04 	mov	#1024,	r1	;#0x0400
//!     c004:	30 40 28 c0 	br	#0xc028		;
//! ```
//!
//! # Symbol interfaces
//!
//! This crate makes heavy use of symbols, linker sections and linker scripts to
//! provide most of its functionality. Below are described the main symbol
//! interfaces.
//!
//! ## `DEFAULT_HANDLER`
//!
//! This weak symbol can be overridden to override the default exception handler
//! that this crate provides. It's recommended that you use the
//! `default_handler!` to do the override, but below is shown how to manually
//! override the symbol:
//!
//! ``` ignore,no_run
//! #[no_mangle]
//! pub extern "msp430-interrupt" fn DEFAULT_HANDLER() {
//!     // do something here
//! }
//! ```
//!
//! ## `.vector_table.interrupts`
//!
//! This linker section is used to register interrupt handlers in the vector
//! table. The recommended way to use this section is to populate it, once, with
//! an array of *weak* functions that just call the `DEFAULT_HANDLER` symbol.
//! Then the user can override them by name.
//!
//! ### Example
//!
//! Populating the vector table
//!
//! ``` ignore,no_run
//! // Number of interrupts the device has
//! const N: usize = 15;
//!
//! // Default interrupt handler that just calls the `DEFAULT_HANDLER`
//! #[linkage = "weak"]
//! #[naked]
//! #[no_mangle]
//! extern "msp430-interrupt" fn WWDG() {
//!     unsafe {
//!         asm!("b DEFAULT_HANDLER" :::: "volatile");
//!         core::intrinsics::unreachable();
//!     }
//! }
//!
//! // You need one function per interrupt handler
//! #[linkage = "weak"]
//! #[naked]
//! #[no_mangle]
//! extern "msp430-interrupt" fn WWDG() {
//!     unsafe {
//!         asm!("b DEFAULT_HANDLER" :::: "volatile");
//!         core::intrinsics::unreachable();
//!     }
//! }
//!
//! // ..
//!
//! // Use `None` for reserved spots in the vector table
//! #[link_section = ".vector_table.interrupts"]
//! #[no_mangle]
//! #[used]
//! static INTERRUPTS: [Option<extern "msp430-interrupt" fn()>; N] = [
//!     Some(WWDG),
//!     Some(PVD),
//!     // ..
//! ];
//! ```
//!
//! Overriding an interrupt (this can be in a different crate)
//!
//! ``` ignore,no_run
//! // the name must match the name of one of the weak functions used to
//! // populate the vector table.
//! #[no_mangle]
//! pub extern "msp430-interrupt" fn WWDG() {
//!     // do something here
//! }
//! ```
//!
//! ## `memory.x`
//!
//! This file supplies the information about the device to the linker.
//!
//! ### `MEMORY`
//!
//! The main information that this file must provide is the memory layout of
//! the device in the form of the `MEMORY` command. The command is documented
//! [here](https://sourceware.org/binutils/docs/ld/MEMORY.html), but at a minimum you'll want to
//! create two memory regions: one for Flash memory and another for RAM.
//!
//! The program instructions (the `.text` section) will be stored in the memory
//! region named ROM, and the program `static` variables (the sections `.bss`
//! and `.data`) will be allocated in the memory region named RAM.
//!
//! ### `_stack_start`
//!
//! This symbol provides the address at which the call stack will be allocated.
//! The call stack grows downwards so this address is usually set to the highest
//! valid RAM address plus one (this *is* an invalid address but the processor
//! will decrement the stack pointer *before* using its value as an address).
//!
//! If omitted this symbol value will default to `ORIGIN(RAM) + LENGTH(RAM)`.
//!
//! #### Example
//!
//! Allocating the call stack on a different RAM region.
//!
//! ```,ignore
//! MEMORY
//! {
//!   /* call stack will go here */
//!   CCRAM : ORIGIN = 0x10000000, LENGTH = 8K
//!   FLASH : ORIGIN = 0x08000000, LENGTH = 256K
//!   /* static variables will go here */
//!   RAM : ORIGIN = 0x20000000, LENGTH = 40K
//! }
//!
//! _stack_start = ORIGIN(CCRAM) + LENGTH(CCRAM);
//! ```
//!
//! ### `_stext`
//!
//! This symbol indicates where the `.text` section will be located. If not
//! specified in the `memory.x` file it will default to right after the vector
//! table -- the vector table is always located at the start of the FLASH
//! region.
//!
//! The main use of this symbol is leaving some space between the vector table
//! and the `.text` section unused. This is required on some microcontrollers
//! that store some configuration information right after the vector table.
//!
//! #### Example
//!
//! Locate the `.text` section 1024 bytes after the start of the FLASH region.
//!
//! ```,ignore
//! _stext = ORIGIN(FLASH) + 0x400;
//! ```
//!
//! ### `_sheap`
//!
//! This symbol is located in RAM right after the `.bss` and `.data` sections.
//! You can use the address of this symbol as the start address of a heap
//! region. This symbol is 4 byte aligned so that address will be a multiple of 4.
//!
//! #### Example
//!
//! ```,ignore
//! extern crate some_allocator;
//!
//! // Size of the heap in bytes
//! const SIZE: usize = 1024;
//!
//! extern "C" {
//!     static mut _sheap: u8;
//! }
//!
//! fn main() {
//!     unsafe {
//!         let start_address = &mut _sheap as *mut u8;
//!         some_allocator::initialize(start_address, SIZE);
//!     }
//! }
//! ```

#![cfg_attr(target_arch = "msp430", feature(core_intrinsics))]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(abi_msp430_interrupt)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(used)]
#![no_std]

extern crate msp430;
#[cfg(target_arch = "msp430")]
extern crate r0;

#[cfg(not(test))]
mod lang_items;

#[cfg(target_arch = "msp430")]
extern "C" {
    // NOTE `rustc` forces this signature on us. See `src/lang_items.rs`
    fn main(argc: isize, argv: *const *const u8) -> isize;

    // Boundaries of the .bss section
    static mut _ebss: u16;
    static mut _sbss: u16;

    // Boundaries of the .data section
    static mut _edata: u16;
    static mut _sdata: u16;

    // Initial values of the .data section (stored in ROM)
    static _sidata: u16;
}

/// The reset handler
///
/// This is the entry point of all programs
#[cfg(target_arch = "msp430")]
unsafe extern "C" fn reset_handler() -> ! {
    r0::zero_bss(&mut _sbss, &mut _ebss);
    r0::init_data(&mut _sdata, &mut _edata, &_sidata);

    // Neither `argc` or `argv` make sense in bare metal context so we
    // just stub them
    main(0, ::core::ptr::null());

    // If `main` returns, then we go into "reactive" mode and simply attend
    // interrupts as they occur.
    loop {
        // Prevent optimizations that can remove this loop.
        ::msp430::asm::barrier();
    }

    // This is the real entry point
    #[link_section = ".vector_table.reset_handler"]
    #[naked]
    unsafe extern "msp430-interrupt" fn trampoline() -> ! {
        // "trampoline" to get to the real reset handler.
        asm!("mov #_stack_start, r1
              br $0"
             :
             : "i"(reset_handler as unsafe extern "C" fn() -> !)
             :
             : "volatile"
        );

        ::core::intrinsics::unreachable()
    }

    #[link_section = ".vector_table.reset_vector"]
    #[used]
    static RESET_VECTOR: unsafe extern "msp430-interrupt" fn() -> ! =
        trampoline;
}

#[export_name = "DEFAULT_HANDLER"]
#[linkage = "weak"]
extern "msp430-interrupt" fn default_handler() {
    // The interrupts are already disabled here.
    loop {
        // Prevent optimizations that can remove this loop.
        ::msp430::asm::barrier();
    }
}

// make sure the compiler emits the DEFAULT_HANDLER symbol so the linker can
// find it!
#[used]
static KEEP: extern "msp430-interrupt" fn() = default_handler;

/// This macro lets you override the default exception handler
///
/// The first and only argument to this macro is the path to the function that
/// will be used as the default handler. That function must have signature
/// `fn()`
///
/// # Examples
///
/// ``` ignore
/// default_handler!(foo::bar);
///
/// mod foo {
///     pub fn bar() {
///         loop {}
///     }
/// }
/// ```
#[macro_export]
macro_rules! default_handler {
    ($path:path) => {
        #[allow(non_snake_case)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "msp430-interrupt" fn DEFAULT_HANDLER() {
            // type checking
            let f: fn() = $path;
            f();
        }
    }
}
