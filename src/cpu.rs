//types
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::env;
//traits
use std::io::Read;

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
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

    //Debug string
    debug: String,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        let mut _debug :String = "".to_owned();

        CPU {
            PC: _PC,
            S: _S,
            P: _P,
            A: _A,
            X: _X,
            Y: _Y,
            mem: _mem,
            debug: _debug,
        }
    }

    // run a program
    pub fn test_load(&mut self, program: &str) {
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
    }

    pub fn test_run(&mut self, log: &str) {
        //for some reason cargo indents the first line of output
        println!("");

        let path = env::current_dir().unwrap().join(log);
        let f = match File::open(&log) {
            Err(why) => panic!("Couldn't open {:?}: {}", path, Error::description(&why)),
            Ok(file) => file,
        };

        let file = BufReader::new(&f);

        for line in file.lines() {
            let mut curr_line = match line {
                Err(why) => panic!("Couldn't read line from {:?}: {}", path, Error::description(&why)),
                Ok(line) => line,
            };

            let instr = self.fetch_instr();
            self.execute_instr(&instr);

            curr_line.truncate(73);
            //println!("Log   line:{}", curr_line);
            //println!("Debug line:{}", self.debug);
            if curr_line != self.debug {
                println!("{}", curr_line);
            }
            assert_eq!(self.debug, curr_line);
        }
    }

    pub fn run(&mut self) {
        loop {
            let instr = self.fetch_instr();
            self.execute_instr(&instr);
        }
    }

    fn fetch_instr(&mut self) -> u8 {
        let instr = self.mem[self.PC as usize];
        self.debug = format!("{:04X}  {:02X} ", self.PC, instr);
        self.PC = self.PC + 1;
        instr
    }

    fn execute_instr(&mut self, instr: &u8) {
        match *instr {
            0x18 => {
                self.debug.push_str(&format!("       CLC                             "));
                self.print_registers();
                self.CLC();
            },
            0x20 => {
                let operand = self.read_absolute();
                self.debug.push_str(&format!("{:02X} {:02X}  JSR ${:04X}                       ", operand as u8, operand >> 8 as u8, operand));
                self.print_registers();
                self.JSR(&operand);
            },
            0x38 => {
                self.debug.push_str(&format!("       SEC                             "));
                self.print_registers();
                self.SEC();
            },
            0x4C => {
                let operand = self.read_absolute();
                self.debug.push_str(&format!("{:02X} {:02X}  JMP ${:04X}                       ", operand as u8, operand >> 8 as u8, operand));
                self.print_registers();
                self.JMP(&operand);
            },
            0x86 => {
                let operand = self.read_zeropage();
                self.debug.push_str(&format!("{:02X}     STX ${:02X} = {:02X}                    ", operand, operand, self.X));
                self.print_registers();
                self.STX(&operand);
            },
            0xA2 => {
                let operand = self.read_immediate();
                self.debug.push_str(&format!("{:02X}     LDX #${:02X}                        ", operand, operand));
                self.print_registers();
                self.LDX(&operand);
            },
            0xB0 => {
                let operand = self.read_relative();
                let branch_location = (self.PC as i32 + operand as i32) as u16;
                //println!("PC: {:04X}, OFFSET: {}", self.PC, operand);
                self.debug.push_str(&format!("{:02x}     BCS ${:04X}                       ", operand, branch_location));
                self.print_registers();
                //println!("{}", self.debug);
                self.BCS(&operand);
            },
            0xEA => {
                self.debug.push_str(&format!("       NOP                             "));
                self.print_registers();
                self.NOP();
            },
            _ => {
                println!("NOT IMPLEMENTED YET");
            },
        }
    }

    fn print_registers(&mut self) {
        self.debug.push_str(&format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", self.A, self.X, self.Y, self.P, self.S));
    }

    fn is_carry_set(&self) -> bool {
        self.P & 0x01 > 0
    }

    fn set_carry_flag(&mut self) {
        self.P |= 0x01;
    }

    fn clear_carry_flag(&mut self) {
        self.P &= !(1 << 0);
    }

    fn set_zero_flag(&mut self) {
        self.P |= 0x02;
    }

    fn set_neg_flag(&mut self) {
        self.P |= 0x080;
    }

    fn read_absolute(&mut self) -> u16 {
        let mem = (self.mem[self.PC as usize] as u16) | ((self.mem[(self.PC + 1) as usize]) as u16 ) << 8;
        self.PC = self.PC + 2;
        mem
    }

    fn read_relative(&mut self) -> i8 {
        let mem = self.mem[self.PC as usize] as i8;
        self.PC = self.PC + 1;
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

    fn push_to_stack(&mut self, value: &u8) {
        if self.S == 0x00 {
            panic!("Stack overflow!");
        }

        self.mem[(0x0100 + self.S as u16) as usize] = *value;
        self.S = self.S - 1;
    }

    fn pop_from_stack(&mut self) -> u8 {
        if self.S >= 0xFF {
            panic!("Stack underflow!");
        }

        self.S = self.S + 1;
        self.mem[(0x0100 + self.S as u16) as usize]
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
    fn BCS(&mut self, operand: &i8) {
        if self.is_carry_set() {
            self.PC = (self.PC as i32 + *operand as i32) as u16;
        }
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
    fn CLC(&mut self) {
        self.clear_carry_flag();
    }

    // SEt Carry
    fn SEC(&mut self) {
        self.set_carry_flag();
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
    fn JSR(&mut self, operand: &u16) {
        let PCH: u8 = (self.PC >> 8) as u8;
        let PCL: u8 = self.PC as u8;
        self.push_to_stack(&PCH);
        self.push_to_stack(&PCL);
        self.PC = *operand;
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
