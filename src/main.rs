use std::time::{Instant, Duration};
use std::io::{stdin, stdout, Read, Write};



// obelisk's notes
// https://web.archive.org/web/20210803073202/http://www.obelisk.me.uk/6502/architecture.html
// https://web.archive.org/web/20210803073200/http://www.obelisk.me.uk/6502/instructions.html
// https://web.archive.org/web/20210803072351/http://www.obelisk.me.uk/6502/registers.html

// llx Neil 6502 notes
// https://llx.com/Neil/a2/opcodes.html


// -------------------------------------------------
// Memory Map 64 KB (0x0000 - 0xFFFF)
// -------------------------------------------------
// Range
// Start  End
// 0000 - 00FF : Zero page
// 0100 - 01FF : Stack
// 0200 - FFF9 : User mappable RAM/ROM/BUS devices
// FFFA - FFFB : "non-maskable interrupt handler"
// FFFC - FFFD : "power on reset location"
// FFFE - FFFF : "BRK/interrupt request handler"
// -------------------------------------------------

struct Registers {
    a: u8,
    x: u8,
    y: u8,
    flags: u8,
    pc: u16,
    sp: u8,
}

struct CPU {
    registers: Registers,
    memory: [u8; (1 << 16)],
}

impl CPU {
    fn new() -> Self {
        CPU {
            registers: Registers {
                a: 0,
                x: 0,
                y: 0,
                flags: 0, // see obelisk 6502 register docs
                pc: 0,
                sp: 0,  // stack pointer sets to FF on boot
            },
            memory:[0; (1 << 16)],
        }
    }


    fn boot(&mut self) {
        // load the memory address from the power on reset vector into the program counter

        // address as stored as little endian so low byte is first byte, high byte is the second byte
        let pl = u16::from(self.memory[0xFFFC]);         // low byte
        let ph = u16::from(self.memory[0xFFFD]) << 8;    // high byte

        self.registers.sp = 0xFF; // stack pointer set to FF on boot
        self.registers.pc = pl | ph;
    }

    fn dump_registers(&mut self) {
        println!("========================================================");
        println!("Register dump");
        println!("--------------------------------------------------------");
        println!("PC:       0x {:04X}", self.registers.pc);
        println!("A:        0x   {:02X}", self.registers.a);
        println!("X:        0x   {:02X}", self.registers.x);
        println!("Y:        0x   {:02X}", self.registers.y);
        println!("SP:       0x   {:02X}", self.registers.sp);
        println!("Flags:    0x   {:02X}", self.registers.flags);
    }

    fn dump_page(&mut self, page: u8) {

        let base_addr = (page as u16) << 8;

        println!("========================================================");
        print!("Memory dump of page {:02X} ", page);
        if (page == 0) {
            print!("(Zero page)");
        } else if (page == 1) {
            print!("(Stack)");
        }
        println!("");
        println!("--------------------------------------------------------");

        for i in 0..256 {
            if i % 16 == 0 {
                print!("{:04X}:  ", base_addr | i);
            } else if i % 8 == 0 {
                print!("  ");
            }
            print!("{:02X} ", self.memory[(base_addr | i) as usize]);
            if i % 16  == 15 {
                println!("");
            }
        }
    }

    fn step(&mut self) {

        let mut advance =       1u16;   // default to 1 byte instruction length. branches/jumps should set this to 0
        let mut flag_mask =     0u8;    // flags that can be affected by this instruction
        let mut flag_output =   0u8;    // flags that are set by this instruction

        let opcode = self.memory[self.registers.pc as usize];
        let bp1 = self.memory[((self.registers.pc as u32 + 1 as u32) & 0xFFFF) as usize];
        let bp2 = self.memory[((self.registers.pc as u32 + 2 as u32) & 0xFFFF) as usize];
        let offset = u16::from(bp1) | (u16::from(bp2) << 8);
        let offset_plus_one = offset.wrapping_add(1);
        let indirect_byte = self.memory[offset as usize];
        let indirect_addr = u16::from(indirect_byte) | (u16::from(self.memory[offset_plus_one as usize]) << 8);

        // check if opcode is a single byte instruction... NOP/SEI/CEI/CLI/CLC/CLV/SEC/SED
        // .. check if opcode is immediate mode or not...
        // ? if immediate mode length is always (?) 2 bytes?
        // ? in non-immediate mode addressing bit 3 being set appears to denote 3 byte instructions
        //
        // see llx's notes above on opcodes for more info
        // Most instructions that explicitly reference memory locations have bit patterns of the form aaabbbcc.
        // The aaa and cc bits determine the opcode, and the bbb bits determine the addressing mode.

        // -------------------------------------------------
        // ADC - Add with Carry
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        //       76543210
        // -------------------------------------------------
        // x69  b01101001  2         Immediate
        // x65  b01100101  2         Zero Page
        // x75  b01110101  2         Zero Page,X
        // x6D  b01101101  3         Absolute
        // x7D  b01111101  3         Absolute,X
        // x79  b01111001  3         Absolute,Y
        // x61  b01100001  2         (Indirect,X)
        // x71  b01110001  2         (Indirect),Y
        // -------------------------------------------------

        // -------------------------------------------------
        // INC - Increment Memory
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // xE6  b--------  2         Zero Page
        // xF6  b--------  2         Zero Page,X
        // xEE  b--------  3         Absolute
        // xFE  b--------  3         Absolute,X
        if opcode == 0xE6  {
            advance = 2;
            self.memory[bp1 as usize] = self.memory[bp1 as usize].wrapping_add(1);
        }

        // -------------------------------------------------
        // JMP - Jump (sets the program counter)
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // x4C  b01001000  3         Absolute
        // x6C  b01101000  3         Indirect
        if opcode == 0x4C {
            advance = 0;
            self.registers.pc = offset;
        }

        if opcode == 0x6C {
            advance = 0;
            self.registers.pc = indirect_addr;
        }

        // -------------------------------------------------
        // LDA - Load Accumulator (register A)
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // xA9  b10101001  2         Immediate
        // xA5  b10100101  2         Zero Page
        // xB5  b10110101  2         Zero Page,X
        // xAD  b10101101  3         Absolute
        // xBD  b10111101  3         Absolute,X
        // xB9  b10111001  3         Absolute,Y
        // xA1  b10100001  2         (Indirect,X)
        // xB1  b10110001  2         (Indirect),Y
        // -------------------------------------------------
        if opcode == 0xA9 {
            advance = 2;
            self.registers.a = bp1;
        }

        if opcode == 0xA5 {
            advance = 2;
            self.registers.a = self.memory[bp1 as usize];
        }

        if opcode == 0xAD {
            advance = 2;
            self.registers.a = self.memory[offset as usize];
        }

        // -------------------------------------------------
        // PHA - Push Accumulator
        // Pushes a copy of the accumulator on to the stack.
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // x48  b01001000  1         Implied

        if opcode == 0x48 {
            advance = 1;
            let full_stack_address = (0x0100 as u16 | (self.registers.sp as u16)) as usize;
            self.memory[full_stack_address] = self.registers.a;
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        }

        // -------------------------------------------------
        // PLA - Pull Accumulator
        // Pulls an 8 bit value from the stack and into
        // the accumulator.
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // x68  b01001000  1         Implied
        if opcode == 0x68 {
            advance = 1;
            self.registers.a = self.memory[(0x0100 as u16 | (self.registers.sp as u16)) as usize];
            self.registers.sp = self.registers.sp.wrapping_add(1);
        }

        // -------------------------------------------------
        // advance the program counter
        // todo : compare performance to wrapping add
        // todo : compare performance checking if advance is > 0 . possibly faster branch emulation but slower for everything else.. worth it?
        self.registers.pc = ((self.registers.pc as u32 + advance as u32) & 0xFFFF) as u16;

        // calculate the new flags
        self.registers.flags = flag_output;
    }
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
    println!("");
}

fn pretty_print_int(i: isize) {
    let mut s = String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ',');
        }
        s.insert(0, val);
    }
    print!("{}", s);
}

fn main() {

    let mut mos = CPU::new();


    println!("6502 CPU Emulator");

    // write a custom reset vector to the memory
    // set reset vector
    mos.memory[0xFFFC] = 0x00;
    mos.memory[0xFFFD] = 0x02;

    // set a custom JMP address for JMP indirect
    mos.memory[0x0500] = 0x02;  // this will skip the load 69 into a instruction
    mos.memory[0x0501] = 0x02;

    // beginning of program memory
    mos.memory[0x0200] = 0xA9;          // LDA #$69
    mos.memory[0x0201] = 0x69;

    mos.memory[0x0202] = 0xE6;          // INC $00
    mos.memory[0x0203] = 0x00;

    mos.memory[0x0204] = 0xE6;          // INC $00
    mos.memory[0x0205] = 0x00;

    mos.memory[0x0206] = 0xE6;          // INC $01
    mos.memory[0x0207] = 0x01;

    mos.memory[0x0208] = 0x48;          // PHA

    mos.memory[0x0209] = 0xA5;          // LDA $01
    mos.memory[0x020A] = 0x01;

    // JMP $0300
    mos.memory[0x020B] = 0x4C;
    mos.memory[0x020C] = 0x00; // account for endianness
    mos.memory[0x020D] = 0x03;

    // JMP ($0500) - Jump the the address specified at 0500
    mos.memory[0x0300] = 0x6C;
    mos.memory[0x0301] = 0x00; // account for endianness
    mos.memory[0x0302] = 0x05;



    // boot the cpu
    mos.boot();
    println!("Post-Boot registers");
    mos.dump_registers();
    mos.dump_page(0);
    mos.dump_page(1);

    // run the first batch instructions by hand
    for _n in 1..20 {
        println!("Execution halted. PC at 0x{:04X}", mos.registers.pc);
        pause();
        mos.step();
        mos.dump_registers();
        mos.dump_page(0);
        mos.dump_page(1);
    }

    println!("Running 1 second benchmark...");

    // benchmark the emulator
    let instant = Instant::now();
    let one_second = Duration::from_secs(1);
    let mut instructions = 0u64;
    while instant.elapsed() < one_second {
        mos.step();
        instructions += 1;
    }
    // dump the contents of registers and the 0 page after the benchmark concludes
    mos.dump_registers();
    mos.dump_page(0);
    mos.dump_page(1);
    // display the number of instructions executed
    pretty_print_int(instructions as isize);
    println!(" instructions in 1 second");
}
