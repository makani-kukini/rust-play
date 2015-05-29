//types
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::env;
//traits
use std::io::Read;

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub struct CPU {
    //Registers

    //Program Counter
    PC: u16,

    //Stack pointer
    S: u8,

    //Processor status
    P: u8,

    //Accumulator
    A: u8,

    //Index register X
    X: u8,

    //Index register Y
    Y: u8,


    //Memory: FFFF, https://en.wikibooks.org/wiki/NES_Programming/Memory_Map
    //Address   Size    Desc.
    //$0000     $800    2KB of work RAM
    //$0800     $800    Mirror of $000-$7FF
    //$1000     $800    Mirror of $000-$7FF
    //$1800     $800    Mirror of $000-$7FF
    //$2000     8       PPU Ctrl Registers
    //$2008     $1FF8   *Mirror of $2000-$2007
    //$4000     $20     Registers (Mostly APU)
    //$4020     $1FDF   Cartridge Expansion ROM
    //$6000     $2000   SRAM
    //$8000     $4000   PRG-ROM
    //$C000     $4000   PRG-ROM
    //
    //this size is without any mirrors
    mem: Vec<u8>,
}

enum mem_map {
    internal_ram,
    ppu_registers,
    apu_io_registers,
    cartridge_space,
}

/*struct instruction {
    //Short form instruction name
    name: &'static str,

    //Opcode
    opcode: u8,

    //Cycles to fully decode instruction
    length: u8,

    //Cycles to complete instruction
    cycles: u8,

    //Closure that executes the instruction
    execute: ||,
}*/

#[allow(non_snake_case)]
impl CPU {
    pub fn new() -> CPU {
        let mut _mem = Vec::with_capacity(0xFFFF);
        for _ in 0..0xFFFF {
            _mem.push(0);
        }
        let mut _PC = 0;
        let mut _S = 0xFD;
        let mut _P = 0;
        let mut _A = 0;
        let mut _X = 0;
        let mut _Y = 0;

        CPU {
            PC: _PC,
            S: _S,
            P: _P,
            A: _A,
            X: _X,
            Y: _Y,
            mem: _mem,
        }
    }

    // run a program
    pub fn run(&mut self, program: &str) {
        let path = env::current_dir().unwrap().join(program);
        let f = match File::open(&path) {
            Err(why) => panic!("Couldn't open {:?}: {}", path, Error::description(&why)),
            Ok(file) => file,
        };

        let mut i = 0;
        let mut program_mem: Vec<u8> = Vec::new();
        for byte in f.bytes() {
            let curr = match byte {
                Err(_) => return, //EOF
                Ok(byte) => byte,
            };

            i = i + 1;
            //skip the header information
            //only the PRG ROM for testing CPU
            if i > 16 && i <= 16400 {
                program_mem.push(curr);
            }
        }
        self.set_mem(mem_map::cartridge_space, &mut program_mem);
        self.PC = 0xC000;
        self.P = 0x24;

        loop {
            let instr: u8 = self.mem[self.PC as usize];

            print!("{:04X}  {:02X} ", self.PC, instr);
            self.PC = self.PC + 1;

            match instr {
                0x4C => {
                    let operand = self.read_absolute();
                    print!("{:02X} {:02X}  JMP ${:04X}                       ", operand as u8, operand >> 8 as u8, operand);
                    self.JMP(&operand);
                },
                0x86 => {
                    let operand = self.read_zeropage();
                    print!("{:02X}     STX ${:02X} = {:02X}                    ", operand, operand, self.X);
                    self.STX(&operand);
                },
                0xA2 => {
                    let operand = self.read_immediate();
                    print!("{:02X}     LDX #${:02X}                        ", operand, operand);
                    self.LDX(&operand);
                },
                _ => {
                    println!("NOT IMPLEMENTED YET");
                    break;
                },
            }

            self.print_registers();
        }
    }

    fn print_registers(&self) {
        println!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", self.A, self.X, self.Y, self.P, self.S);
    }

    fn set_zero_flag(&mut self) {
        self.PC |= 0x02;
    }

    fn set_neg_flag(&mut self) {
        self.PC |= 0x080;
    }

    fn read_absolute(&mut self) -> u16 {
        let mem = (self.mem[self.PC as usize] as u16) | ((self.mem[(self.PC + 1) as usize]) as u16 ) << 8;
        self.PC = self.PC + 2;
        mem
    }

    fn read_immediate(&mut self) -> u8 {
        let mem = self.mem[self.PC as usize];
        self.PC = self.PC + 1;
        mem
    }

    fn read_zeropage(&mut self) -> u8 {
        let mem = self.mem[self.PC as usize];
        self.PC = self.PC + 1;
        mem
    }

    fn set_mem(&mut self, start_addr: mem_map, program_mem: &mut Vec<u8>) {
        match start_addr {
            mem_map::internal_ram => {
                let start_addr = 0x0000;
                for i in 0..program_mem.len() {
                    self.mem[start_addr + i] = program_mem[i];
                }
            },
            mem_map::ppu_registers => {
                let start_addr = 0x2000;
                for i in 0..program_mem.len() {
                    self.mem[start_addr + i] = program_mem[i];
                }
            },
            mem_map::apu_io_registers => {
                let start_addr = 0x4000;
                for i in 0..program_mem.len() {
                    self.mem[start_addr + i] = program_mem[i];
                }
            },
            mem_map::cartridge_space => {
                let start_addr = 0xC000;
                for i in 0..program_mem.len() {
                    if (0xC000 + i) < 0xFFFF {
                        self.mem[start_addr + i] = program_mem[i];
                    }
                }
            },
        }
    }

    /*fn initialize(&self) {
    }*/

    // ADd with Carry
    fn ADC(&self) {

    }

    // bitwise AND with accumulator
    fn AND(&self) {

    }

    // Arithmetic Shift Left
    fn ASL(&self) {

    }

    // test BITs
    fn BIT(&self) {

    }

    // Branch on PLus
    fn BPL(&self) {

    }

    // Branch on MInus
    fn BMI(&self) {

    }

    // Branch on oVerflow Clear
    fn BVC(&self) {

    }

    // Branch on oVerflow Set
    fn BVS(&self) {

    }

    // Branch on Carry Clear
    fn BCC(&self) {

    }

    // Branch on Carry Set
    fn BCS(&self) {

    }

    // Branch on Not Equal
    fn BNE(&self) {

    }

    // Branch on EQual
    fn BEQ(&self) {

    }

    //BReaK
    fn BRK(&self) {

    }

    // CoMPare accumulator
    fn CMP(&self) {

    }

    // ComPare X register
    fn CPX(&self) {

    }

    // ComPare Y register
    fn CPY(&self)
    {

    }

    // DECrement Memory
    fn DEC(&self)
    {

    }

    // bitwise Exclusive org
    fn EOR(&self) {

    }

    // CLear Carry
    fn CLC(&self) {

    }

    // SEt Carry
    fn SEC(&self) {

    }

    // CLear Interrupt
    fn CLI(&self) {

    }

    // SEt Interrupt
    fn SEI(&self) {

    }

    // CLear oVerflow
    fn CLV(&self) {

    }

    // CLear Decimal
    fn CLD(&self) {

    }

    // SEt Decimal
    fn SED(&self) {

    }

    // INCrement Memory
    fn INC(&self) {

    }

    // JuMP
    fn JMP(&mut self, operand: &u16) {
        self.PC = *operand;
    }

    // Jump to SubRoutine
    fn JSR(&self) {

    }

    // LoaD Accumulator
    fn LDA(&self) {

    }

    // LoaD X register
    fn LDX(&mut self, operand: &u8) {
        self.X = *operand;
        if self.X == 0 {
            self.set_zero_flag();
        }
        if self.X & 0x80 > 0 {
            self.set_neg_flag();
        }
    }

    // LoaD Y register
    fn LDY(&self) {

    }

    // Logical Shift Right
    fn LSR(&self) {

    }

    // No OPeration
    fn NOP(&self) {

    }

    // bitwise OR with Accumulator
    fn ORA(&self) {

    }

    // Transfer A to X
    fn TAX(&self)
    {

    }

    // Transfer X to A
    fn TXA(&self)
    {

    }

    // DEcrement X
    fn DEX(&self) {

    }

    // INcrement X
    fn INX(&self) {

    }

    // Transfer A to Y
    fn TAY(&self) {

    }

    // Transfer Y to A
    fn TYA(&self) {

    }

    // DEcrement Y
    fn DEY(&self) {

    }

    // INcrement Y
    fn INY(&self) {

    }

    // ROtate Left
    fn ROL(&self) {

    }

    // ROtate Right
    fn ROR(&self) {

    }

    // ReTurn from Interrupt
    fn RTI(&self) {

    }

    // ReTurn from Subroutine
    fn RTS(&self) {

    }

    // SuBtract with Carry
    fn SBC(&self) {

    }

    // STore Accumulator
    fn STA(&self)
    {

    }

    // Transfer X to Stack ptr
    fn TXS(&self) {

    }

    // Transfer Stack ptr to X
    fn TSX(&self) {

    }

    // PusH Accumulator
    fn PHA(&self) {

    }

    // PuLl Accumulator
    fn PLA(&self) {

    }

    // PusH Processor status
    fn PHP(&self) {

    }

    // PuLl Processor status
    fn PLP(&self) {

    }

    // STore X register
    fn STX(&mut self, operand: &u8) {
        self.mem[*operand as usize] = self.X;
    }

    // STore Y register
    fn STY(&self) {

    }
}
