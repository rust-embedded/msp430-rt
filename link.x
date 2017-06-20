INCLUDE memory.x

SECTIONS
{
  .vector_table ORIGIN(VECTORS) :
  {
    /* Vector table */
    _svector_table = .;

    KEEP(*(.rodata.interrupts));
    _einterrupts = .;

    KEEP(*(.vector_table.reset_handler));
    _evector_table = .;
  } > VECTORS

  .text ORIGIN(FLASH) :
  {
    /* Put reset handler first in .text section so it ends up as the entry */
    /* point of the program. */
    KEEP(*(.reset_handler));

    *(.text .text.*);
  } > FLASH

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);
  } > FLASH

  .bss : ALIGN(4)
  {
    _sbss = .;
    *(.bss .bss.*);
    _ebss = ALIGN(4);
  } > RAM

  .data : ALIGN(4)
  {
    _sdata = .;
    *(.data .data.*);
    _edata = ALIGN(4);
  } > RAM AT > FLASH

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
  .debug_gdb_scripts 0 (INFO) : {
    KEEP(*(.debug_gdb_scripts))
  }

  .stlog 0 (INFO) : {
    _sstlog_trace = .;
    *(.stlog.trace*);
    _estlog_trace = .;

    _sstlog_debug = .;
    *(.stlog.debug*);
    _estlog_debug = .;

    _sstlog_info = .;
    *(.stlog.info*);
    _estlog_info = .;

    _sstlog_warn = .;
    *(.stlog.warn*);
    _estlog_warn = .;

    _sstlog_error = .;
    *(.stlog.error*);
    _estlog_error = .;
  }

  /DISCARD/ :
  {
    /* Unused unwinding stuff */
    *(.ARM.exidx.*)
    *(.ARM.extab.*)
  }
}

/* Do not exceed this mark in the error messages below                | */
ASSERT(_einterrupts - _svector_table > 0, "
You must specify the interrupt handlers.
Create a non `pub` static variable and place it in the
'.rodata.interrupts' section. (cf. #[link_section]). Apply the
`#[used]` attribute to the variable to help it reach the linker.");

ASSERT(_evector_table == 0x10000, "
Vector table must always end at address 0x10000 (0xFFFE + 2).
Please check the 'VECTORS' memory region or 
the '.rodata.interrupts' section. (cf. #[link_section])");
