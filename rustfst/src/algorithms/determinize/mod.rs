pub use determinize_fsa_impl::DeterminizeFsaImpl;
pub use determinize_static::{determinize, determinize_with_distance};
pub(self) use divisors::{DefaultCommonDivisor, GallicCommonDivisor, LabelCommonDivisor};
pub(self) use element::{DeterminizeElement, DeterminizeStateTuple, DeterminizeTr, WeightedSubset};

mod determinize_fsa_impl;
mod determinize_static;
mod divisors;
mod element;

/// Determinization type.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DeterminizeType {
    /// Input transducer is known to be functional (or error).
    DeterminizeFunctional,
    /// Input transducer is NOT known to be functional.
    DeterminizeNonFunctional,
    /// Input transducer is not known to be functional but only keep the min of
    /// of ambiguous outputs.
    DeterminizeDisambiguate,
}
