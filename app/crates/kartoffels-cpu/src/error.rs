use thiserror::Error;

pub type FwResult<T, E = FwError> = std::result::Result<T, E>;
pub type TickResult<T, E = TickError> = std::result::Result<T, E>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum FwError {
    #[error(
        "expected a 32-bit binary, but got a 64-bit one\n\n\
         this is most likely the outcome of a backwards-incompatible change \
         introduced in kartoffels v0.7 - if you're following the kartoffel \
         repository, simply clone it again and copy your code there\n\n\
         sorry for the trouble and godspeed!"
    )]
    MismatchedArchitecture,

    #[error("expected a little-endian binary, but got a big-endian one")]
    MismatchedEndianess,

    #[error("found no segments")]
    NoSegments,

    #[error(
        "segment #{idx} spans outside the available memory (it starts at \
         0x{addr:0x}, which is before 0x{limit:0x})"
    )]
    SegmentUnderflow { idx: usize, addr: u32, limit: u32 },

    #[error(
        "segment #{idx} spans outside the available memory (it ends at \
         0x{addr:0x}, which is after 0x{limit:0x})"
    )]
    SegmentOverflow { idx: usize, addr: u32, limit: u32 },

    #[error(transparent)]
    Object(#[from] object::Error),
}

#[derive(Clone, Copy, Debug, Error)]
pub enum TickError {
    #[error("null-pointer access on 0x{addr:08x}+{size}")]
    NullPointerAccess { addr: u32, size: u8 },

    #[error("invalid access on 0x{addr:08x}+{size}")]
    InvalidAccess { addr: u32, size: u8 },

    #[error("unknown instruction: 0x{word:08x}")]
    UnknownInstruction { word: u32 },

    #[error("got `ebreak`")]
    Ebreak,
}
