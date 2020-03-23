use crate::algorithms::lookahead_matchers::interval_set::IntervalSet;
use crate::algorithms::lookahead_matchers::state_reachable::StateReachable;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, MutableArcIterator, MutableFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, EPS_LABEL, NO_LABEL};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use failure::Fallible;
use crate::fst_properties::FstProperties;

pub struct LabelReachableData {
    reach_input: bool,
    final_label: Label,
    label2index: HashMap<Label, Label>,
    interval_sets: Vec<IntervalSet>,
}

impl LabelReachableData {
    pub fn new(reach_input: bool) -> Self {
        Self {
            reach_input,
            final_label: NO_LABEL,
            label2index: HashMap::new(),
            interval_sets: Vec::new(),
        }
    }

    pub fn interval_set(&self, s: StateId) -> Fallible<&IntervalSet> {
        self.interval_sets.get(s).ok_or_else(|| format_err!("Missing state {}", s))
    }

    pub fn final_label(&self) -> Label {
        self.final_label
    }
}

pub struct LabelReachable {
    data: LabelReachableData,
    label2state: HashMap<Label, StateId>,
    reach_fst_input: bool
}

impl LabelReachable {
    pub fn new<W: Semiring + 'static>(mut fst: VectorFst<W>, reach_input: bool) -> Fallible<Self> {
        // TODO: In OpenFst, the Fst is converted to a VectorFst
        let mut label_reachable = Self {
            data: LabelReachableData::new(reach_input),
            label2state: HashMap::new(),
            reach_fst_input: false
        };

        let nstates = fst.num_states();
        label_reachable.transform_fst(&mut fst);
        label_reachable.find_intervals(&fst, nstates)?;

        Ok(label_reachable)
    }

    pub fn reach_input(&self) -> bool {
        self.data.reach_input
    }

    // Redirects labeled arcs (input or output labels determined by ReachInput())
    // to new label-specific final states. Each original final state is
    // redirected via a transition labeled with kNoLabel to a new
    // kNoLabel-specific final state. Creates super-initial state for all states
    // with zero in-degree.
    fn transform_fst<W: Semiring + 'static>(&mut self, fst: &mut VectorFst<W>) {
        let ins = fst.num_states();
        let mut ons = ins;
        let mut indeg = vec![0; ins];
        // Redirects labeled arcs to new final states.
        for s in 0..ins {
            for arc in unsafe { fst.arcs_iter_unchecked_mut(s) } {
                let label = if self.data.reach_input {
                    arc.ilabel
                } else {
                    arc.olabel
                };
                if label != EPS_LABEL {
                    arc.nextstate = match self.label2state.entry(label) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                }
                indeg[arc.nextstate] += 1;
            }

            if let Some(final_weight) = unsafe { fst.final_weight_unchecked(s) } {
                if !final_weight.is_zero() {
                    let nextstate = match self.label2state.entry(NO_LABEL) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                    unsafe {
                        fst.add_arc_unchecked(
                            s,
                            Arc::new(NO_LABEL, NO_LABEL, final_weight.clone(), nextstate),
                        )
                    };
                    indeg[nextstate] += 1;
                    unsafe { fst.delete_final_weight_unchecked(s) }
                }
            }
        }

        // Adds new final states to the FST.
        while fst.num_states() < ons {
            let s = fst.add_state();
            unsafe { fst.set_final_unchecked(s, W::one()) };
        }

        // Creates a super-initial state for all states with zero in-degree.
        let start = fst.add_state();
        unsafe { fst.set_start_unchecked(start) };
        for s in 0..start {
            if indeg[s] == 0 {
                unsafe {
                    fst
                        .add_arc_unchecked(start, Arc::new(0, 0, W::one(), s))
                };
            }
        }
    }

    fn find_intervals<W: Semiring + 'static>(&mut self, fst: &VectorFst<W>, ins: StateId) -> Fallible<()> {
        let state_reachable = StateReachable::new(fst)?;
        let state2index = &state_reachable.state2index;
        let interval_sets = &mut self.data.interval_sets;
        *interval_sets = state_reachable.isets;
        interval_sets.resize_with(ins, IntervalSet::default);
        let label2index = &mut self.data.label2index;
        for (label, state) in self.label2state.iter() {
            let i = state2index[*state];
            if *label == NO_LABEL {
                self.data.final_label = i;
            }
        }
        self.label2state.clear();
        Ok(())
    }

    pub fn reach_init<F: ExpandedFst>(&mut self, fst: &F, reach_input: bool) -> Fallible<()> {
        self.reach_fst_input = reach_input;
        let props = fst.properties()?;

        let true_prop = if self.reach_fst_input {
            FstProperties::I_LABEL_SORTED
        } else {
            FstProperties::O_LABEL_SORTED
        };

        if !props.contains(true_prop) {
            bail!("LabelReachable::ReachInit: Fst is not sorted")
        }
        Ok(())
    }

    // Can reach this label from current state?
    // Original labels must be transformed by the Relabel methods above.
    pub fn reach_label(&self, current_state: StateId, label: Label) -> Fallible<bool> {
        if label == EPS_LABEL {
            return Ok(false);
        }
        Ok(self.data.interval_set(current_state)?.member(label))
    }

    // Can reach final state (via epsilon transitions) from this state?
    pub fn reach_final(&self, current_state: StateId) -> Fallible<bool> {
        Ok(self.data.interval_set(current_state)?.member(self.data.final_label()))
    }
}
