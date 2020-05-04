use std::fmt;

use crate::fst_impls::ConstFst;
use crate::fst_traits::{TrIterator, CoreFst, FinalStatesIterator, StateIterator};
use crate::semirings::SerializableSemiring;

display_fst_trait!(W, ConstFst<W>);
