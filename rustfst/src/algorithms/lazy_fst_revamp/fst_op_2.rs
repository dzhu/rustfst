use std::fmt::Debug;

use anyhow::Result;

use crate::{Semiring, StateId, TrsVec};
use crate::fst_properties::FstProperties;

pub trait FstOp2<W: Semiring>: Debug {
    // was FstImpl
    fn compute_start(&self) -> Result<Option<StateId>>;
    fn compute_trs_and_final_weight(&self, id: usize) -> Result<(TrsVec<W>, Option<W>)>;

    fn properties(&self) -> FstProperties;
}
