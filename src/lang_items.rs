/// Default panic handler
#[cfg(feature = "abort-on-panic")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    _args: ::core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
    _col: u32,
) -> ! {
    // Disable interrupts to prevent further damage.
    ::msp430::interrupt::disable();
    loop {
        // Prevent optimizations that can remove this loop.
        ::msp430::asm::barrier();
    }
}

/// Lang item required to make the normal `main` work in applications
// This is how the `start` lang item works:
// When `rustc` compiles a binary crate, it creates a `main` function that looks
// like this:
//
// ```
// #[export_name = "main"]
// pub extern "C" fn rustc_main(argc: isize, argv: *const *const u8) -> isize {
//     start(main)
// }
// ```
//
// Where `start` is this function and `main` is the binary crate's `main`
// function.
//
// The final piece is that the entry point of our program, the reset handler,
// has to call `rustc_main`. That's covered by the `reset_handler` function in
// root of this crate.
#[lang = "start"]
extern "C" fn start(
    main: fn(),
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    main();

    0
}
