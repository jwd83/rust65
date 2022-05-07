use std::time::Instant;


// create memory array
// create registers

// memory map

// https://web.archive.org/web/20210803073202/http://www.obelisk.me.uk/6502/architecture.html
// https://web.archive.org/web/20210803073200/http://www.obelisk.me.uk/6502/instructions.html
// https://web.archive.org/web/20210803072351/http://www.obelisk.me.uk/6502/registers.html

// Range
// Start  End
// 0000 - 00FF : Zero page
// 0100 - 01FF : Stack
// 0200 - FFF9 : USER RAM/ROM
// FFFA - FFFB : "non-maskable interrupt handler"
// FFFC - FFFD : "power on reset location"
// FFFE - FFFF : "BRK/interrupt request handler"

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
                flags: 0, // https://web.archive.org/web/20210803072351/http://www.obelisk.me.uk/6502/registers.html
                pc: 0,
                sp: 0,
            },
            memory:[0; (1 << 16)],
        }
    }


    fn boot(&mut self) {
        // load the memory address from the power on reset vector into the program counter

        // address as stored as little endian so low byte is first byte, high byte is the second byte
        let pl = u16::from(self.memory[0xFFFC]);         // low byte
        let ph = u16::from(self.memory[0xFFFD]) << 8;    // high byte

        self.registers.pc = pl | ph;
    }

    fn dump_registers(&mut self) {
        println!("A: {}", self.registers.a);
        println!("X: {}", self.registers.x);
        println!("Y: {}", self.registers.y);
        println!("PC: {}", self.registers.pc);
        println!("SP: {}", self.registers.sp);
        println!("Flags: {}", self.registers.flags);
    }

    fn dump_page(&mut self, page: u8) {

        let base_addr = (page as u16) << 8;

        println!("Memory dump of page: {}", page);
        println!("Starting base address: {:04X}", base_addr);

        for i in 0..255 {
            print!("{:02X} ", self.memory[ (base_addr | i) as usize]);
            if(i % 8 == 7) {
                println!("");
            }
        }
        println!("");
    }

    fn step(&mut self) {

        let mut advance =       1u16;   // default to 1 byte instruction length. branches/jumps should set this to 0
        let mut flag_mask =     0u8;    // flags that can be affected by this instruction
        let mut flag_output =   0u8;    // flags that are set by this instruction

        let opcode = self.memory[self.registers.pc as usize];


        // check if opcode is a single byte instruction... NOP/SEI/CEI/CLI/CLC/CLV/SEC/SED
        // .. check if opcode is immediate mode or not...
        // ? if immedaite mode length is always (?) 2 bytes?
        // ? in non-immediate mode addressing bit 3 being set appears to denote 3 byte instructions


        // -------------------------------------------------
        // ADC - Add with Carry
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        //       76543210
        // -------------------------------------------------
        // x69  b01101001  2        Immediate
        // x65  b01100101  2        Zero Page
        // x75  b01110101  2        Zero Page,X
        // x6D  b01101101  3        Absolute
        // x7D  b01111101  3        Absolute,X
        // x79  b01111001  3        Absolute,Y
        // x61  b01100001  2        (Indirect,X)
        // x71  b01110001  2        (Indirect),Y
        // -------------------------------------------------

        // -------------------------------------------------
        // INC - Increment Memory
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // xE6  b--------  2        Zero Page
        // xF6  b--------  2        Zero Page,X
        // xEE  b--------  3        Absolute
        // xFE  b--------  3        Absolute,X



        // -------------------------------------------------
        // LDA - Load Accumulator (register A)
        // -------------------------------------------------
        // Opcode          Byte
        // HEX  BIN        Length    Addressing Mode
        // -------------------------------------------------
        // xA9  b10101001  2        Immediate
        // xA5  b10100101  2        Zero Page
        // xB5  b10110101  2        Zero Page,X
        // xAD  b10101101  3        Absolute
        // xBD  b10111101  3        Absolute,X
        // xB9  b10111001  3        Absolute,Y
        // xA1  b10100001  2        (Indirect,X)
        // xB1  b10110001  2        (Indirect),Y
        // -------------------------------------------------
        if(opcode == 0xA9) {
            self.registers.a = self.memory[(self.registers.pc + 1u16) as usize ];
            advance = 2;
        }


        // -------------------------------------------------
        // advance the program counter
        self.registers.pc = ((self.registers.pc as u32 + advance as u32) & 0xFFFF) as u16;

        // calculate the new flags
        self.registers.flags = flag_output;
    }
}

fn main() {

    let mut mos = CPU::new();


    println!("6502 CPU Emulator");

    println!("Pre-Boot registers");
    mos.dump_registers();

    // write a custom reset vector to the memory
    // set reset vector
    mos.memory[0xFFFC] = 0x00;
    mos.memory[0xFFFD] = 0x02;

    // LDA #$69
    mos.memory[0x0200] = 0xA9;
    mos.memory[0x0201] = 0x69;

    // INC $00
    mos.memory[0x0202] = 0xE6;
    mos.memory[0x0203] = 0x00;

    mos.boot();
    mos.dump_registers();
    mos.dump_page(0);

    mos.step();

    mos.dump_registers();

    let benchmark_start = Instant::now();
    for n in 1..200 {
        mos.step();
    }
    let benchmark_end = Instant::now();
    println!("{:?}", benchmark_end.duration_since(benchmark_start));

    mos.dump_registers();

}
