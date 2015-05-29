use cpu::CPU;

mod cpu;

fn main() {
    let mut cpu = CPU::new();
    cpu.run("nestest.nes");
}
