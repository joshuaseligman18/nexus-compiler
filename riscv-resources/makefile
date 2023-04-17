build:
	rm -f my_program.o
	rm -f my_program
	riscv64-unknown-elf-as -c my_program.s -o my_program.o
	riscv64-unknown-elf-gcc my_program.o -o my_program -nostdlib -static

run:
	riscv64-unknown-elf-run my_program

dump:
	riscv64-unknown-elf-objdump -d -M no-aliases my_program.o
