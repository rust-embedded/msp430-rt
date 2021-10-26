//! Startup code and minimal runtime for MSP430 microcontrollers
//!
//! This crate is based on [cortex-m-rt](https://docs.rs/cortex-m-rt)
//! crate by Jorge Aparicio (@japaric).
//!
//! This crate contains all the required parts to build a `no_std` application (binary crate) that
//! targets a MSP430 microcontroller.
//!
//! # Features
//!
//! This crates takes care of:
//!
//! - The memory layout of the program. In particular, it populates the vector table so the device
//! can boot correctly, and properly dispatch interrupts.
//!
//! - Initializing `static` variables before the program entry point.
//!
//! This crate also provides the following attributes:
//!
//! - `#[entry]` to declare the entry point of the program
//! - `#[pre_init]` to run code *before* `static` variables are initialized
//!
//! This crate also implements a related attribute called `#[interrupt]`, which allows you
//! to define interrupt handlers. However, since which interrupts are available depends on the
//! microcontroller in use, this attribute should be re-exported and used from a PAC crate.
//!
//! The documentation for these attributes can be found in the [Attribute Macros](#attributes)
//! section.
//!
//! # Requirements
//!
//! ## `memory.x`
//!
//! This crate expects the user, or some other crate, to provide the memory layout of the target
//! device via a linker script named `memory.x`. This section covers the contents of `memory.x`
//!
//! ### `MEMORY`
//!
//! The linker script must specify the memory available in the device as, at least, three `MEMORY`
//! regions: one named `ROM`, one named `RAM`, and one named `VECTORS`. The `.text` and `.rodata`
//! sections of the program will be placed in the `ROM` region, whereas the `.bss` and `.data`
//! sections, as well as the heap, will be placed in the `RAM` region. The `.vector_table` section,
//! which including the interrupt vectors and reset address, will be placed in the `VECTORS`
//! region at the end of flash. The `ROM` region should end at the address the `VECTORS` region
//! begins.
//!
//! A `VECTORS` region is required because between (_and within_) msp430 device families:
//! * Devices do not have a constant single vector table size.
//! * Devices do not have a constant vector table start address.
//! Consult your Family User's Guide (e.g. MSP430x5xx Family User's Guide, slau208),
//! particularly the Memory Map section, and your device's datasheet (e.g. msp430g2553) for
//! information on vector table layout and size. _You may be able to get more program space if
//! your device's datasheet explicitly marks a contiguous set of vectors as unused!_
//!
//!
//! ``` text
//! /* Linker script for the MSP430G2553 */
//! MEMORY
//! {
//!   RAM : ORIGIN = 0x0200, LENGTH = 0x0200
//!   ROM : ORIGIN = 0xC000, LENGTH = 0x3FE0
//!   VECTORS : ORIGIN = 0xFFE0, LENGTH = 0x20
//! }
//! ```
//!
//! # An example
//!
//! This section presents a minimal application built on top of `msp430-rt`.
//!
//! ``` ignore
//! // IMPORTANT the standard `main` interface is not used because it requires nightly
//! #![no_main]
//! #![no_std]
//!
//! extern crate msp430_rt;
//! // Simple panic handler that infinitely loops.
//! extern crate panic_msp430;
//!
//! use msp430_rt::entry;
//!
//! // use `main` as the entry point of this application
//! // `main` is not allowed to return
//! #[entry]
//! fn main() -> ! {
//!     // initialization
//!
//!     loop {
//!         // application logic
//!     }
//! }
//!
//! ```
//!
//! To actually build this program you need to place a `memory.x` linker script somewhere the linker
//! can find it, e.g. in the current directory; and then link the program using `msp430-rt`'s
//! linker script: `link.x`. The required steps are shown below:
//!
//! ``` text
//! $ cat > memory.x <<EOF
//! /* Memory layout of the MSP430G2553 */
//! MEMORY
//! {
//!   RAM : ORIGIN = 0x0200, LENGTH = 0x0200
//!   ROM : ORIGIN = 0xC000, LENGTH = 0x3FE0
//!   VECTORS : ORIGIN = 0xFFE0, LENGTH = 0x20
//! }
//! EOF
//!
//! $ xargo rustc --target msp430-none-elf -- \
//!       -C link-arg=-nostartfiles -C link-arg=-Tlink.x
//!
//! $ file target/msp430-none-elf/debug/app
//! app: ELF 32-bit LSB executable, TI msp430, version 1 (embedded), statically linked, not stripped
//! ```
//!
//! # Optional features
//!
//! ## `device`
//!
//! If this feature is disabled then this crate populates the whole vector table. All the interrupts
//! in the vector table, even the ones unused by the target device, will be bound to the default
//! interrupt handler. This makes the final application device agnostic: you will be able to run it
//! on any MSP430 device -- provided that you correctly specified its memory layout in `memory.x`
//! -- without hitting undefined behavior.
//!
//! If this feature is enabled then the interrupts section of the vector table is left unpopulated
//! and some other crate, or the user, will have to populate it. This mode is meant to be used in
//! conjunction with PAC crates generated using `svd2rust`. Those *PAC crates* will populate the
//! missing part of the vector table when their `"rt"` feature is enabled.
//!
//! # Inspection
//!
//! This section covers how to inspect a binary that builds on top of `msp430-rt`.
//!
//! ## Sections (`size`)
//!
//! `msp430-rt` uses standard sections like `.text`, `.rodata`, `.bss` and `.data` as one would
//! expect. `msp430-rt` separates the vector table in its own section, named `.vector_table`. This
//! lets you distinguish how much space is taking the vector table in Flash vs how much is being
//! used by actual instructions (`.text`) and constants (`.rodata`).
//!
//! ``` text
//! $ size -Ax target/msp430-none-elf/examples/app
//! section              size     addr
//! .vector_table        0x20   0xffe0
//! .text                0x44   0xc000
//! .rodata               0x0   0xc044
//! .bss                  0x0    0x200
//! .data                 0x0    0x200
//! .MSP430.attributes   0x17      0x0
//! Total                0x7b
//! ```
//!
//! Without the `-A` argument `size` reports the sum of the sizes of `.text`, `.rodata` and
//! `.vector_table` under "text".
//!
//! ``` text
//! $ size target/msp430-none-elf/examples/app
//!    text    data     bss     dec     hex filename
//!     100       0       0     100      64 target/msp430-none-elf/release/app
//! ```
//!
//! ## Symbols (`objdump`, `nm`)
//!
//! One will always find the following (unmangled) symbols in `msp430-rt` applications:
//!
//! - `ResetTrampoline`. This is the reset handler. The microcontroller will executed this function
//! upon booting. This trampoline simply initializes the stack pointer and the jumps to `Reset`.
//!
//! - `Reset`. This function will call the user program entry point (See `#[entry]`) using the
//! `main` symbol so you may also find that symbol in your program; if you do, `main` will contain
//! your application code. Some other times `main` gets inlined into `Reset` and you won't find it.
//!
//! - `DefaultHandler`. This is the default interrupt handler. If not overridden using `#[interrupt]
//! fn DefaultHandler(..` this will be an infinite loop.
//!
//! - `__RESET_VECTOR`. This is the reset vector, a pointer into `ResetTrampoline`. This vector is
//! located at the end of the `.vector_table` section.
//!
//! - `__INTERRUPTS`. This is the device specific interrupt portion of the vector table. This array
//! is located right before `__RESET_VECTOR` in the `.vector_table` section.
//!
//! - `PreInit`. This is a function to be run before RAM is initialized. It defaults to an empty
//! function. The function called can be changed using the `#[pre_init]` attribute. The empty
//! function is not optimized out by default, but if an empty function is marked with the
//! `#[pre_init]` attribute then the function call will be optimized out.
//!
//! If you overrode any interrupt handler you'll find it as an unmangled symbol, e.g. `NMI` or
//! `WDT`, in the output of `objdump`,
//!
//! # Advanced usage
//!
//! ## Setting the program entry point
//!
//! This section describes how `#[entry]` is implemented. This information is useful to developers
//! who want to provide an alternative to `#[entry]` that provides extra guarantees.
//!
//! The `Reset` handler will call a symbol named `main` (unmangled) *after* initializing `.bss` and
//! `.data`. `#[entry]` provides this symbol in its expansion:
//!
//! ``` ignore
//! #[entry]
//! fn main() -> ! {
//!     /* user code */
//! }
//!
//! // expands into
//!
//! #[export_name = "main"]
//! extern "C" fn randomly_generated_string() -> ! {
//!     /* user code */
//! }
//! ```
//!
//! The unmangled `main` symbol must have signature `extern "C" fn() -> !` or its invocation from
//! `Reset`  will result in undefined behavior.
//!
//! ## Incorporating device specific interrupts
//!
//! This section covers how an external crate can insert device specific interrupt handlers into the
//! vector table. Most users don't need to concern themselves with these details, but if you are
//! interested in how device crates generated using `svd2rust` integrate with `msp430-rt` read on.
//!
//! The information in this section applies when the `"device"` feature has been enabled.
//!
//! ### `__INTERRUPTS`
//!
//! The external crate must provide the interrupts portion of the vector table via a `static`
//! variable named`__INTERRUPTS` (unmangled) that must be placed in the `.vector_table.interrupts`
//! section of its object file.
//!
//! This `static` variable will be placed at `ORIGIN(VECTORS)`. This address corresponds to the
//! spot where IRQ0 (IRQ number 0) is located.
//!
//! To conform to the MSP430 ABI `__INTERRUPTS` must be an array of function pointers; some spots
//! in this array may need to be set to 0 if they are marked as *reserved* in the data sheet /
//! reference manual. We recommend using a `union` to set the reserved spots to `0`; `None`
//! (`Option<fn()>`) may also work but it's not guaranteed that the `None` variant will *always* be
//! represented by the value `0`.
//!
//! Let's illustrate with an artificial example where a device only has two interrupt: `Foo`, with
//! IRQ number = 2, and `Bar`, with IRQ number = 4.
//!
//! ``` ignore
//! union Vector {
//!     handler: extern "msp430-interrupt" fn(),
//!     reserved: usize,
//! }
//!
//! extern "msp430-interrupt" {
//!     fn Foo();
//!     fn Bar();
//! }
//!
//! #[link_section = ".vector_table.interrupts"]
//! #[no_mangle]
//! static __INTERRUPTS: [Vector; 15] = [
//!     // 0-1: Reserved
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!
//!     // 2: Foo
//!     Vector { handler: Foo },
//!
//!     // 3: Reserved
//!     Vector { reserved: 0 },
//!
//!     // 4: Bar
//!     Vector { handler: Bar },
//!
//!     // 5-14: Reserved
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//!     Vector { reserved: 0 },
//! ];
//! ```
//!
//! ### `device.x`
//!
//! Linking in `__INTERRUPTS` creates a bunch of undefined references. If the user doesn't set a
//! handler for *all* the device specific interrupts then linking will fail with `"undefined
//! reference"` errors.
//!
//! We want to provide a default handler for all the interrupts while still letting the user
//! individually override each interrupt handler. In C projects, this is usually accomplished using
//! weak aliases declared in external assembly files. In Rust, we could achieve something similar
//! using `global_asm!`, but that's an unstable feature.
//!
//! A solution that doesn't require `global_asm!` or external assembly files is to use the `PROVIDE`
//! command in a linker script to create the weak aliases. This is the approach that `msp430-rt`
//! uses; when the `"device"` feature is enabled `msp430-rt`'s linker script (`link.x`) depends on
//! a linker script named `device.x`. The crate that provides `__INTERRUPTS` must also provide this
//! file.
//!
//! For our running example the `device.x` linker script looks like this:
//!
//! ``` text
//! /* device.x */
//! PROVIDE(Foo = DefaultHandler);
//! PROVIDE(Bar = DefaultHandler);
//! ```
//!
//! This weakly aliases both `Foo` and `Bar`. `DefaultHandler` is the default interrupt handler.
//!
//! Because this linker script is provided by a dependency of the final application the dependency
//! must contain build script that puts `device.x` somewhere the linker can find. An example of such
//! build script is shown below:
//!
//! ``` ignore
//! use std::{env, fs::File, io::Write, path::PathBuf};
//!
//! fn main() {
//!     // Put the linker script somewhere the linker can find it
//!     let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
//!     File::create(out.join("device.x"))
//!         .unwrap()
//!         .write_all(include_bytes!("device.x"))
//!         .unwrap();
//!     println!("cargo:rustc-link-search={}", out.display());
//! }
//! ```
//!
//! [attr-entry]: attr.entry.html
//! [attr-exception]: attr.exception.html
//! [attr-pre_init]: attr.pre_init.html

#![deny(missing_docs)]
#![feature(abi_msp430_interrupt)]
#![no_std]

use msp430::asm;
pub use msp430_rt_macros::interrupt;
pub use msp430_rt_macros::{entry, pre_init};

/// Returns a pointer to the start of the heap
///
/// The returned pointer is guaranteed to be 4-byte aligned.
#[inline]
pub fn heap_start() -> *mut u32 {
    extern "C" {
        static mut __sheap: u32;
    }

    unsafe { &mut __sheap }
}

extern "msp430-interrupt" {
    fn Reset() -> !;
}

#[link_section = ".__RESET_VECTOR"]
#[no_mangle]
static __RESET_VECTOR: unsafe extern "msp430-interrupt" fn() -> ! = Reset;

#[no_mangle]
unsafe extern "C" fn PreInit_() {}

#[no_mangle]
extern "msp430-interrupt" fn DefaultHandler_() -> ! {
    // The interrupts are already disabled here.
    loop {
        // Prevent optimizations that can remove this loop.
        asm::barrier();
    }
}

// Interrupts for generic application
#[cfg(not(feature = "device"))]
#[no_mangle]
#[link_section = ".vector_table.interrupts"]
static __INTERRUPTS: [unsafe extern "msp430-interrupt" fn(); 15] = [{
    extern "msp430-interrupt" {
        fn DefaultHandler();
    }

    DefaultHandler
}; 15];
