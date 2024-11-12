pub trait Mmio {
    fn load(self, addr: u32) -> Result<u32, ()>;
    fn store(self, addr: u32, val: u32) -> Result<(), ()>;
}
