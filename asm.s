  .section .Reset, "ax"
  .global Reset
  .type Reset,%function
Reset:
  mov #_stack_start, r1
  call #PreInit

; .bss init
  clr r4
.more_bss:
  cmp bss_size, r4
  jhs .done_bss ; r4 >= bss_size
  clr.b _sbss(r4) ; Zero out RAM.
  inc r4
  jmp .more_bss

; .data init
.done_bss:
  clr r4
.more_data:
  cmp data_size, r4
  jhs .done_data ; r4 >= data_size
  mov.b _sidata(r4), _sdata(r4) ; Copy from ROM to RAM.
  inc r4
  jmp .more_data

.done_data:
  br #main

.section .rodata, "a"
.align 2 ; MSP430 will not behave properly w/ unaligned reads, from my testing.
bss_size: .word _ebss - _sbss
data_size: .word _edata - _sdata
