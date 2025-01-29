#![no_std]

macro_rules! bots {
    ([ $(static $id:ident = $name:literal;)* ]) => {
        $(
            #[cfg(not(target_arch = "riscv32"))]
            pub static $id: &[u8] = include_bytes!(concat!(
                env!("OUT_DIR"),
                "/target.riscv/riscv32-kartoffel-bot/release/",
                $name
            ));
        )*
    };
}

bots!([
    static CHL_ACYCLIC_MAZE = "chl-acyclic-maze";
    static CHL_DIAMOND_HEIST = "chl-diamond-heist";
    static CHL_DIAMOND_HEIST_GUARD = "chl-diamond-heist-guard";
    static CHL_PERSONAL_ROOMBA = "chl-personal-roomba";
    static DUMMY = "dummy";
    static ROBERTO = "roberto";
    static TUT_01 = "tut-01";
    static TUT_02 = "tut-02";
    static TUT_03 = "tut-03";
    static TUT_04 = "tut-04";
]);
