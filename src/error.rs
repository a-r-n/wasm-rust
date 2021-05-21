pub enum Error {
    InvalidInput,
    BadVersion,
    UnknownSection,
    UnknownOpcode,
    EndOfData,
    IntSizeViolation,
    StackViolation,
    UnexpectedData(&'static str),
    Misc(&'static str), /* Just to facilitate development for now, or for one-off errors */
}
