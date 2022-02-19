# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## Added
- The `entry` attribute macro for main can now take an `interrupt_enable`
  argument. The `interrupt_enable` argument will enable interrupts before
  calling the function with the `entry` attribute, possibly calling an an
  optional setup function before enabling interrupts.
  - Not using the `interrupt_enable` argument is backwards-compatible with [v0.3.0].
- Use [`trybuild`](https://github.com/dtolnay/trybuild) to test error messages
  from the `pre-init`, `entry`, and `interrupt` macros.
  - Use [GHA](https://github.com/rust-embedded/msp430-rt/actions) for these UI
    tests.

## Fixed
- [docs.rs](https://docs.rs) [metadata](https://docs.rs/about/metadata) added
  to `Cargo.toml` to fix documentation building [failure](https://github.com/rust-embedded/msp430-rt/issues/16).
- Change instances of `msp430_macros` in docs to correct `msp430_rt_macros`
  import.

## [v0.3.0]- 2022-01-25

### Changed
- [breaking-change] In preparation for bumping the version of `bare-metal` in
  `msp430` to `v1.0`, the `CriticalSection` parameters to the `main` and ISR
  functions now have their lifetimes constrained to the body of their
  functions. Due to `no_mangle`, PACs using v0.2.x will not work properly
  with `v0.3.0` of this crate.
- [breaking-change] Bump `msp430` to `v0.3.0`, which uses the new
  `CriticalSection<'a>` parameters generated for the `main` and ISR functions
  (via `interrupt::free` and `interrupt::Mutex`)
- Remaining dependencies were bumped to the most recent versions available on
  Cargo at the time.
  - The RNG used to generate idents was changed from [`Pcg64Mcg`](https://docs.rs/rand/0.6.0/rand/rngs/struct.SmallRng.html)
    (64-bit) and `Pcg32` (32-bit) to [`Xoshiro128PlusPlus`](https://docs.rs/rand/0.8.4/rand/rngs/struct.SmallRng.html)
    (32/64-bit) to avoid the need for the `SmallRng` feature (and consistency
    between 32/64-bit platforms).
  - No other functional changes intended by bumping dependencies.

## [v0.2.5]- 2021-10-27

To replace [`r0`], startup code is now written in assembly, optimized for
size. The startup code provides the non-mangled function called `Reset` which:
  - Initializes the stack pointer.
  - Calls `PreInit`.
  - Initializes static data in the `.bss` and `.data` sections.
  - Calls `main`.

Because `Reset` is no longer written in Rust, a user's `main` function will no
longer be inlined into `Reset`. Additionally, `ResetTrampoline` was removed,
since its sole purpose was to set the stack pointer; `Reset` does this now.

### Removed
- Remove [`r0`] dependency in light of potential [unsoundness](https://github.com/rust-embedded/cortex-m-rt/issues/300).
  This internal change should not affect user-facing applications.

## [v0.2.4]- 2020-03-04

### Fixed
- `msp430-rt` now depends on the correct version of `msp430-rt-macros`.

## [v0.2.3]- 2020-03-04 (YANKED)

This release was yanked because `msp430-rt-macros` was not updated accordingly
before release.

### Added
- Modify `#[entry]` and `#[interrupt]` macros so that `main()` and interrupt
  handlers optionally accept a `CriticalSection` argument.

### Changed
- Proactively update [`r0`] to 1.0.0, the first crate of
  [WG Issue #383](https://github.com/rust-embedded/wg/issues/383).

## [v0.2.2]- 2020-01-07

### Fixed
- Fix entry point in linker script to be `ResetTrampoline` instead of `Reset`.
  This caused subtle breakage during debugging where invoking and exiting `gdb`
  without forcing a `monitor reset` (_which you should be doing anyway_).
  Specifically, `gdb` would reset the program counter to skip the stack
  initialization code in `ResetTrampoline`, which can leak stack memory.

## [v0.2.1]- 2020-01-07

### Fixed
- Correct build.rs script so that msp430-rt is only recompiled if `link.x.in`
  changed, rather than `link.x`. The latter triggers unconditional rebuilds.

## [v0.2.0]- 2020-01-01

- [breaking-change] Interrupts are now implemented using an attribute macro
  called `#[interrupt]`, provided by the [`msp430-rt-macros`](macros) package.
- [breaking-change] Old compilers using the `Termination` trait will no longer
  compile this crate.
- [breaking-change] The `INTERRUPTS` array is now called `__INTERRUPTS` for
  parity with [`cortex-m-rt`],
  and the linker script has been updated to accomodate. `^0.1.0` PACs will not
  work properly with this crate.
- [breaking-change] If the `device` feature is enabled, the linker script
  expects a PAC, such as [`msp430g2553`](https://github.com/pftbest/msp430g2553),
  to provide interrupt vector addresses via the `device.x` file. This should be
  transparent to the user due to `build.rs`.
- [breaking-change] The `default_handler` macro was removed; a default
  interrupt handler is defined defining an function called `DefaultHandler`
  with the `#[interrupt]` attribute.
- [breaking-change] An application's entry point is now defined using the
  `#[entry]` attribute macro, with function signature `fn() -> !`
- Add `#[pre_init]` attribute macro, for parity with [`cortex-m-rt`].
- Removed instances of `asm` macros. Use a separate assembly file called
  `asm.s` and `libmsp430.a` for stable assembly when absolutely necessary (at
  the cost of some inlining). This should be transparent to the user thanks
  to `build.rs`.
- Reset handler name changed from `reset_handler` to `Reset`; this is invisible
  to users.
- All but one required feature have either stabilized (`used`) or are no longer
  used in the crate (`asm`, `lang_items`, `linkage`, `naked_functions`). The
  only remaining unstable feature is [`abi_msp430_interrupt`](https://github.com/rust-lang/rust/issues/38487).

## [v0.1.4] - 2019-11-01

- Removed panic_implementation

## [v0.1.3] - 2018-06-18

- Upgrade to panic_implementation

## [v0.1.2] - 2018-04-08

- Fix version tags

- Fix build with recent nightly-2018-04-08.

## [v0.1.1] - 2018-02-02

- Import `Termination` trait code from [`cortex-m-rt`] to permit compiling with
recent nightlies.

## v0.1.0 - 2017-07-22

Initial release

[`r0`]: https://github.com/rust-embedded/r0
[`cortex-m-rt`]: https://github.com/japaric/cortex-m-rt

[Unreleased]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.3.0...HEAD
[v0.3.0]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.2.5...msp_v0.3.0
[v0.2.5]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.2.4...msp_v0.2.5
[v0.2.4]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.2.3...msp_v0.2.4
[v0.2.3]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.2.2...msp_v0.2.3
[v0.2.2]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.2.1...msp_v0.2.2
[v0.2.1]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.2.0...msp_v0.2.1
[v0.2.0]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.1.4...msp_v0.2.0
[v0.1.4]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.1.3...msp_v0.1.4
[v0.1.3]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.1.2...msp_v0.1.3
[v0.1.2]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.1.1...msp_v0.1.2
[v0.1.1]: https://github.com/rust-embedded/msp430-rt/compare/msp_v0.1.0...msp_v0.1.1
