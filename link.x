INCLUDE memory.x

/* Create an undefined reference to the INTERRUPTS symbol. This is required to
   force the linker to *not* drop the INTERRUPTS symbol if it comes from an
   object file that's passed to the linker *before* this crate */
EXTERN(INTERRUPTS);

PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));

SECTIONS
{
  .vector_table ORIGIN(VECTORS) : ALIGN(2)
  {
    _sinterrupts = .;
    KEEP(*(.vector_table.interrupts));
    _einterrupts = .;

    KEEP(*(.vector_table.reset_vector));
  } > VECTORS

  .text ORIGIN(ROM) :
  {
    /* Put the reset handler first in .text section so it ends up as the entry
       point of the program */
    KEEP(*(.vector_table.reset_handler));

    *(.text .text.*);
  } > ROM

  .rodata : ALIGN(2)
  {
    *(.rodata .rodata.*);
    . = ALIGN(2);
  } > ROM

  .bss : ALIGN(2)
  {
    _sbss = .;
    *(.bss .bss.*);
    . = ALIGN(2);
    _ebss = .;
  } > RAM

  .data : ALIGN(2)
  {
    _sidata = LOADADDR(.data);
    _sdata = .;
    *(.data .data.*);
    . = ALIGN(2);
    _edata = .;
  } > RAM AT > ROM

  /* fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect
     relocatable code in the input files and raise an error if relocatable code
     is found */
  .got :
  {
    _sgot = .;
    KEEP(*(.got .got.*));
    _egot = .;
  } > RAM AT > ROM

  /* The heap starts right after the .bss + .data section ends */
  _sheap = _edata;

  /* Due to an unfortunate combination of legacy concerns,
     toolchain drawbacks, and insufficient attention to detail,
     rustc has no choice but to mark .debug_gdb_scripts as allocatable.
     We really do not want to upload it to our target, so we
     remove the allocatable bit. Unfortunately, it appears
     that the only way to do this in a linker script is
     the extremely obscure "INFO" output section type specifier. */
  /* a rustc hack will force the program to read the first byte of this section,
     so we'll set the (fake) start address of this section to something we're
     sure can be read at runtime: the start of the .text section */
  .debug_gdb_scripts ORIGIN(ROM) (INFO) : {
    KEEP(*(.debug_gdb_scripts))
  }
}

/* Do not exceed this mark in the error messages below                | */
ASSERT(_einterrupts - _sinterrupts > 0, "
The interrupt handlers are missing. If you are not linking to a device
crate then you supply the interrupt handlers yourself. Check the
documentation.");

ASSERT(ORIGIN(VECTORS) + LENGTH(VECTORS) == 0x10000, "
The VECTORS memory region must end at address 0x10000. Check memory.x");

ASSERT(_einterrupts == 0xFFFE, "
The section .vector_table.interrupts appears to be wrong. It should
end at address 0xFFFE");

ASSERT(_sgot == _egot, "
.got section detected in the input files. Dynamic relocations are not
supported. If you are linking to C code compiled using the `gcc` crate
then modify your build script to compile the C code _without_ the
-fPIC flag. See the documentation of the `gcc::Config.fpic` method for
details.");
