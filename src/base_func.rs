use crate::party::PartyId;

use std::hash::Hasher;

#[derive(PartialEq, Copy, Clone, Eq, Hash, Debug)]
pub enum FuncId {
    Fcomcomp = 1,
    Fcom = 2,
    Fmpc = 3,
    Fthresh = 4,
    Ftabit = 5,
    Frand = 6,
    Fcote = 7,
    Fmult = 8,
    Fabit = 9,

    Fnet = 999,
    Ftest = 1000,
    Fcontroller = 10000,
    Other = 65535,
}

impl From<u16> for FuncId {
    fn from(item: u16) -> Self {
        match item {
            1 => FuncId::Fcomcomp,
            2 => FuncId::Fcom,
            3 => FuncId::Fmpc,
            4 => FuncId::Fthresh,
            5 => FuncId::Ftabit,
            6 => FuncId::Frand,
            7 => FuncId::Fcote,
            8 => FuncId::Fmult,
            9 => FuncId::Fabit,
            999 => FuncId::Fnet,
            1000 => FuncId::Ftest,
            10000 => FuncId::Fcontroller,
            65535 => FuncId::Other,
            x => panic!("Unexpected function id {}", x),
        }
    }
}

impl From<FuncId> for u16 {
    fn from(item: FuncId) -> Self {
        item as u16
    }
}

#[derive(PartialEq, Copy, Clone, Eq, Hash, Debug)]
pub struct SessionId {
    pub parent: FuncId,
    pub id: u64,
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?}, {})", self.parent, self.id)
    }
}

impl SessionId {
    pub fn new(caller: FuncId) -> Self {
        SessionId {
            parent: caller,
            id: 0,
        }
    }

    pub fn next(mut self) -> Self {
        self.id += 1;
        self
    }

    pub fn derive_ssid(&self, caller: FuncId) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        h.write_u16(self.parent.into());
        h.write_u64(self.id);
        // _probably_ collision free in our limited use case
        // use top 48 bits as the parent id, bottom 16 as counter
        let subid = h.finish() << 16;
        SessionId {
            parent: caller,
            id: subid,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FuncContext {
    pub party: PartyId,
    pub func: FuncId,
    pub sid: SessionId,
}

impl std::fmt::Display for FuncContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {:?} {}", self.party, self.func, self.sid)
    }
}

pub trait BaseFunc {
    const FUNC_ID: FuncId;
    const REQUIRED_FUNCS: &'static [FuncId];

    fn party(&self) -> PartyId;

    fn ctx(&self, sid: SessionId) -> FuncContext {
        let c = FuncContext {
            party: self.party(),
            func: Self::FUNC_ID,
            sid: sid,
        };
        c
    }

    fn err(&self, sid: SessionId, msg: impl std::fmt::Display) -> String {
        let c = self.ctx(sid);
        format!("{c}: {msg}")
    }

    fn unexpected(&self, sid: SessionId, msg: impl std::fmt::Display) -> UnexpectedError {
        anyhow::anyhow!(self.err(sid, msg)).into()
    }

    fn cheat(&self, sid: SessionId, cheater: Option<PartyId>, msg: String) -> CheatDetectedError {
        let c = self.ctx(sid);

        CheatDetectedError::new(c, cheater, msg)
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct UnexpectedError(
    #[from]
    #[backtrace]
    anyhow::Error,
);

#[derive(thiserror::Error, Debug)]
#[error("{ctx}: Cheat detected by party {cheater:?}, {msg}")]
pub struct CheatDetectedError {
    ctx: FuncContext,
    cheater: Option<PartyId>,
    msg: String,
}

impl CheatDetectedError {
    pub fn new(ctx: FuncContext, cheater: Option<PartyId>, msg: String) -> Self {
        Self { ctx, cheater, msg }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheatOrUnexpectedError {
    #[error(transparent)]
    CheatDetected(
        #[from]
        #[backtrace]
        CheatDetectedError,
    ),
    #[error(transparent)]
    Unexpected(
        #[from]
        #[backtrace]
        UnexpectedError,
    ),
}

impl From<anyhow::Error> for CheatOrUnexpectedError {
    fn from(e: anyhow::Error) -> Self {
        CheatOrUnexpectedError::Unexpected(e.into())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_ssid_stability() {
        let sid = SessionId::new(FuncId::Ftest);

        let ssid = sid.derive_ssid(FuncId::Fcontroller);
        // within a program instance this should be deterministic
        assert_eq!(ssid, sid.derive_ssid(FuncId::Fcontroller));

        let sid2 = sid.clone().next();
        let ssid2 = sid2.derive_ssid(FuncId::Fcontroller);

        // different parents should result in different sub sids
        assert!(ssid != ssid2);
    }
}
