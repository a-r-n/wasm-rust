pub enum Error {
    InvalidInput,
    BadVersion,
    UnknownSection,
    UnknownOpcode(u64),
    EndOfData,
    IntSizeViolation,
    StackViolation,
    UnexpectedData(&'static str),
    Misc(&'static str), /* Just to facilitate development for now, or for one-off errors */
}
