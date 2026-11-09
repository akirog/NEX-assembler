#[derive(Debug, Clone)]
pub enum Address {
    Absolute(TempReg),
    FrameOffset(usize),
}


#[derive(Debug, Clone)]
pub enum MirValue {
    Const(Constant),
    Register(TempReg),
    Load(Address),
    FrameOffsetAddress(usize),
    Array(Vec<MirValue>),
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct TempReg(uid::Id<u32>);

impl TempReg {
    pub fn new() -> TempReg {
        TempReg(uid::Id::new())
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Number(u32),
    Label(String),
}

