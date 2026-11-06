

#[derive(Debug, Clone)]
pub enum Place {
    TempReg(TempReg),
    FrameOffset(usize),
    Address(TempReg),
    Label(String),
    GlobalLabel(String),
}


#[derive(Debug, Clone)]
pub enum MirValue {
    Number(u32),
    Load(Place),
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct TempReg(uid::Id<u32>);

impl TempReg {
    pub fn new() -> TempReg {
        TempReg(uid::Id::new())
    }
}
