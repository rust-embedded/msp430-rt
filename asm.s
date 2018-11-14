  .section .ResetTrampoline, "ax"
  .global ResetTrampoline
  .type ResetTrampoline,%function
ResetTrampoline:
  mov #_stack_start,r1
  br Reset
