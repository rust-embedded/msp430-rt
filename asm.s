  .section .ResetTrampoline, "ax"
  .global ResetTrampoline
  .type ResetTrampoline,%function
ResetTrampoline:
  mov #_stack_start,r1
  br #Reset ; XXX "br Reset" should also work, but doesn't on G2553,
            ; and I don't know why.
