INCLUDE memory.x

/* Create an undefined reference to the INTERRUPTS symbol. This is required to
force the linker to *not* drop the INTERRUPTS symbol if it comes from an
object file that's passed to the linker *before* this crate */
EXTERN(INTERRUPTS);

PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));

SECTIONS
{
  .vector_table ORIGIN(VECTORS) :
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
  } > ROM

  .bss : ALIGN(2)
  {
    _sbss = .;
    *(.bss .bss.*);
    _ebss = ALIGN(2);
  } > RAM

  .data : ALIGN(2)
  {
    _sdata = .;
    *(.data .data.*);
    _edata = ALIGN(2);
  } > RAM AT > ROM

  _sidata = LOADADDR(.data);

  /* The heap starts right after the .bss + .data section ends */
  _sheap = _edata;

  /* Due to an unfortunate combination of legacy concerns,
     toolchain drawbacks, and insufficient attention to detail,
     rustc has no choice but to mark .debug_gdb_scripts as allocatable.
     We really do not want to upload it to our target, so we
     remove the allocatable bit. Unfortunately, it appears
     that the only way to do this in a linker script is
     the extremely obscure "INFO" output section type specifier. */
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
