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

// impl Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
// }
