

#[derive(Debug, Clone)]
pub enum Address {
    Absolute(TempReg),
    FrameOffset(usize),
}


#[derive(Debug, Clone)]
pub enum Place {
    TempReg(TempReg),
    Address(Address),
    Label(String),
    GlobalLabel(String),
}


#[derive(Debug, Clone)]
pub enum MirValue {
    Number(u32),
    Register(TempReg),
    Load(Address),
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct TempReg(uid::Id<u32>);

impl TempReg {
    pub fn new() -> TempReg {
        TempReg(uid::Id::new())
    }
}
