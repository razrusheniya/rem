cat $1 | rem > main.asm
shift

nasm -f elf64 -g -F dwarf -o main.o main.asm
gcc -no-pie -O3 -rdynamic -o main main.o $(pkg-config --cflags --libs gtk+-3.0)
./main "$@"
