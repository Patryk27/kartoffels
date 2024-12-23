pub trait Mmio {
    fn load(self, addr: u32) -> Result<u32, ()>;
    fn store(self, addr: u32, val: u32) -> Result<(), ()>;
}

impl Mmio for () {
    fn load(self, _: u32) -> Result<u32, ()> {
        Err(())
    }

    fn store(self, _: u32, _: u32) -> Result<(), ()> {
        Err(())
    }
}
