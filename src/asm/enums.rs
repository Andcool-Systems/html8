///Assembly 64-bit registers
pub enum Registers64 {
    ///Accumulator
    RAX,
    ///Base
    RBX,
    ///Counter
    RCX,
    ///Data
    RDX,
    ///Source index
    RSI,
    ///Destination index
    RDI,
    ///Base pointer
    RBP,
    ///Stack pointer
    RSP,
    ///Code segment
    CS,
    ///Data segment
    DS,
    ///Stack segment
    SS,
}

pub enum Opcodes<T, R> {
    ADD(T, R),
    SUB(T, R),
    MUL(T),
    IMUL(T, R),
    DIV(T),
    IDIV(T),
    INC(T),
    DEC(T),
}
