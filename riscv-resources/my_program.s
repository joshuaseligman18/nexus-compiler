.section .text
.global _start
_start:
nop
la  t1, o_0
li  t0, 0
sb  t0, 0(t1)
li  t0, 0
la  t1, o_0
sb  t0, 0(t1)
while_start_0:
la  t0, o_0
lbu  a0, 0(t0)
li  a1, 3
call compare_neq
beq  a0, zero, while_end_0
la  t2, o_0
lbu  t1, 0(t2)
li  t0, 1
add  t0, t0, t1
la  t1, o_0
sb  t0, 0(t1)
la  t1, i_1
li  t0, 0
sb  t0, 0(t1)
li  t0, 0
la  t1, i_1
sb  t0, 0(t1)
while_start_1:
la  t0, i_1
lbu  a0, 0(t0)
li  a1, 2
call compare_neq
beq  a0, zero, while_end_1
la  t2, i_1
lbu  t1, 0(t2)
li  t0, 1
add  t0, t0, t1
la  t1, i_1
sb  t0, 0(t1)
la  a0, string_2
call print_string
call print_new_line
la  t0, i_1
lbu  a0, 0(t0)
call print_int
call print_new_line
j  while_start_1
while_end_1:
la  a0, string_3
call print_string
call print_new_line
la  t0, o_0
lbu  a0, 0(t0)
call print_int
call print_new_line
j  while_start_0
while_end_0:
li  a7, 93
li  a0, 0
ecall
print_int:
mv t0, a0
li  a7, 64
li  a0, 1
la  a1, print_int_char
li  a2, 1
li  t1, 0
li  t2, 100
li  t3, 3
li  t4, 10
print_int_loop:
divu  t5, t0, t2
addi  t5, t5, 0x30
sb  t5, 0(a1)
ecall
remu  t0, t0, t2
divu  t2, t2, t4
addi  t1, t1, 1
blt  t1, t3, print_int_loop
ret
print_string:
mv  t0, a0
li  a7, 64
li  a0, 1
lhu  a2, 0(t0)
addi  a1, t0, 2
ecall
ret
print_boolean:
beq  a0, zero, print_false
la  a0, string_1
j  print_bool_call
print_false:
la  a0, string_0
print_bool_call:
addi  sp, sp, -4
sw  ra, 0(sp)
call print_string
lw  ra, 0(sp)
addi  sp, sp, 4
ret
print_new_line:
li  a7, 64
li  a0, 1
la  a1, new_line
li  a2, 1
ecall
ret
compare_eq:
beq  a0, a1, compare_eq_true
li  a0, 0
j  compare_eq_ret
compare_eq_true:
li  a0, 1
compare_eq_ret:
ret
compare_neq:
bne  a0, a1, compare_neq_true
li  a0, 0
j  compare_neq_ret
compare_neq_true:
li  a0, 1
compare_neq_ret:
ret
o_0: .byte 0
i_1: .byte 0
new_line: .ascii "\n"
print_int_char: .byte 0
string_0:
.half 5
.ascii "false"
string_1:
.half 4
.ascii "true"
string_2:
.half 6
.ascii " inner"
string_3:
.half 6
.ascii " outer"

