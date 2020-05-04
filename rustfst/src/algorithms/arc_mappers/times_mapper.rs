use anyhow::Result;

use crate::algorithms::{TrMapper, FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Tr;

/// Mapper to (right) multiply a constant to all weights.
pub struct TimesMapper<W: Semiring> {
    to_multiply: W,
}

impl<W: Semiring> TimesMapper<W> {
    pub fn new(value: W::Type) -> Self {
        TimesMapper {
            to_multiply: W::new(value),
        }
    }

    pub fn from_weight(value: W) -> Self {
        TimesMapper { to_multiply: value }
    }

    pub fn map_weight(&self, weight: &mut W) -> Result<()> {
        weight.times_assign(&self.to_multiply)
    }
}

impl<S: Semiring> TrMapper<S> for TimesMapper<S> {
    fn arc_map(&self, arc: &mut Tr<S>) -> Result<()> {
        self.map_weight(&mut arc.weight)
    }

    fn final_arc_map(&self, final_arc: &mut FinalTr<S>) -> Result<()> {
        self.map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(TimesMapper<S>);
