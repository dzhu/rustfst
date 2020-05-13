use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, SequenceComposeFilter};
use crate::algorithms::compose::matchers::{GenericMatcher, Matcher};
use crate::algorithms::compose::{ComposeFstOp, ComposeFstOpOptions, ComposeStateTuple};
use crate::algorithms::lazy_fst_revamp::{LazyFst, SimpleHashMapCache, StateTable};
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::Semiring;
use crate::{SymbolTable, TrsVec};
use std::sync::Arc;

#[derive(Debug)]
pub struct ComposeFst<W: Semiring, CF: ComposeFilter<W>>(
    LazyFst<W, ComposeFstOp<W, CF>, SimpleHashMapCache<W>>,
);

fn create_base<W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>>(
    fst1: Arc<F1>,
    fst2: Arc<F2>,
) -> Result<ComposeFstOp<W, SequenceComposeFilter<W, GenericMatcher<W, F1>, GenericMatcher<W, F2>>>>
{
    // TODO: change this once Lookahead matchers are supported.
    let opts = ComposeFstOpOptions::<
        GenericMatcher<_, _>,
        GenericMatcher<_, _>,
        SequenceComposeFilter<_, _, _>,
        _,
    >::default();
    let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
    Ok(compose_impl)
}

impl<W: Semiring, CF: ComposeFilter<W>> ComposeFst<W, CF> {
    pub fn new_with_options(
        fst1: Arc<<CF::M1 as Matcher<W>>::F>,
        fst2: Arc<<CF::M2 as Matcher<W>>::F>,
        opts: ComposeFstOpOptions<CF::M1, CF::M2, CF, StateTable<ComposeStateTuple<CF::FS>>>,
    ) -> Result<Self>
    where
        W: 'static,
    {
        let isymt = fst1.input_symbols().cloned();
        let osymt = fst2.output_symbols().cloned();
        let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
        let fst_cache = SimpleHashMapCache::new();
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }

    // TODO: Change API, no really user friendly
    pub fn new(
        fst1: Arc<<CF::M1 as Matcher<W>>::F>,
        fst2: Arc<<CF::M2 as Matcher<W>>::F>,
    ) -> Result<Self>
    where
        W: 'static,
    {
        Self::new_with_options(fst1, fst2, ComposeFstOpOptions::default())
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}

impl<W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>>
    ComposeFst<W, SequenceComposeFilter<W, GenericMatcher<W, F1>, GenericMatcher<W, F2>>>
{
    pub fn new_auto(fst1: Arc<F1>, fst2: Arc<F2>) -> Result<Self> {
        let isymt = fst1.input_symbols().cloned();
        let osymt = fst2.output_symbols().cloned();
        let compose_impl = create_base(fst1, fst2)?;
        let fst_cache = SimpleHashMapCache::new();
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }
}

impl<W, CF> CoreFst<W> for ComposeFst<W, CF>
where
    W: Semiring,
    CF: ComposeFilter<W>,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }
}

impl<'a, W, CF> StateIterator<'a> for ComposeFst<W, CF>
where
    W: Semiring,
    CF: ComposeFilter<W> + 'a,
{
    type Iter =
        <LazyFst<W, ComposeFstOp<W, CF>, SimpleHashMapCache<W>> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, CF> FstIterator<'a, W> for ComposeFst<W, CF>
where
    W: Semiring,
    CF: ComposeFilter<W> + 'a,
{
    type FstIter =
        <LazyFst<W, ComposeFstOp<W, CF>, SimpleHashMapCache<W>> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, CF> Fst<W> for ComposeFst<W, CF>
where
    W: Semiring,
    CF: ComposeFilter<W> + 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_output_symbols()
    }
}
