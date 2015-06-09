extern crate cpu;

#[cfg(test)]
mod tests {
    use cpu::CPU;

    #[test]
    fn nestest() {
        let mut cpu = CPU::new();
        cpu.test_load("nestest.nes");
        cpu.test_run("nestest.log");
    }
}
