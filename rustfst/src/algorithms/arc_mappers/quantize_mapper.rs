use anyhow::Result;

use crate::algorithms::{TrMapper, FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::{Semiring, WeightQuantize};
use crate::Tr;
use crate::KDELTA;

/// Mapper to quantize all weights.
pub struct QuantizeMapper {}

pub fn map_weight<W: WeightQuantize>(weight: &mut W) -> Result<()> {
    weight.quantize_assign(KDELTA)
}

impl<S: WeightQuantize + Semiring> TrMapper<S> for QuantizeMapper {
    fn arc_map(&self, arc: &mut Tr<S>) -> Result<()> {
        map_weight(&mut arc.weight)
    }

    fn final_arc_map(&self, final_arc: &mut FinalTr<S>) -> Result<()> {
        map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

impl<S> WeightConverter<S, S> for QuantizeMapper
where
    S: WeightQuantize,
{
    arc_mapper_to_weight_convert_mapper_methods!(S);
}
