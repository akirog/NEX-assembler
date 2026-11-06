use ratatui::widgets::canvas::Label;

#[derive(Debug, Clone)]
pub enum Address {
    Absolute(TempReg),
    FrameOffset(usize),
}


#[derive(Debug, Clone)]
pub enum Place {
    TempReg(TempReg),
    Address(Address),
    GlobalLabel(String),
}


#[derive(Debug, Clone)]
pub enum MirValue {
    Const(Constant),
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

#[derive(Debug, Clone)]
pub enum Constant {
    Number(u32),
    Label(String),
}

