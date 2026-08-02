.intel_syntax noprefix

.global _start
_start:
  # initial stack layout created by kernel is:
  #   [rsp]           = argc
  #   [rsp+8]         = argv[0]
  #   [rsp+8*argc+8]  = NULL (end of argv)
  #   [rsp+8*argc+16] = envp[0]
  #   ...             ...
  #   ...             = NULL (end of envp)
  mov rdi, [rsp]        # argc
  lea rsi, [rsp+8]      # argv
  lea rdx, [rsi+rdi*8+8] # envp = argv + (argc + 1) * 8 (next of NULL termination)

  # x86_64 System V ABI defines that rsp is already 16 bytes aligned 
  # when _start is called.
  # From _start is called until now, rsp is not changed,
  # so 16 bytes aligned rule for function calling is filled at this point.
  call main

  # exit from process
  mov rdi, rax           # set exit code with main return value
  mov rax, 231           # syscall number of exit_group(2)
  syscall
