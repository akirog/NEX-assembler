use std::fmt::Display;

#[derive(Debug)]
pub struct MirValue {
    pub kind: MirValueKind,
}

#[derive(Debug)]
pub enum MirValueKind {
    Number(u32),
    TempReg(TempReg),
    StackSlot(u32),
    GlobalLabel(String),
    Label(String),
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct TempReg(uid::Id<u32>);

impl TempReg {
    pub fn new() -> TempReg {
        TempReg(uid::Id::new())
    }
}