//! `FieldRef.SetRange` with an Integer, BigInteger, Decimal or Boolean value on a Text or Code field: the value goes in as its
//! `Format` text, where `ops::convert` of a Variant holding it would raise TypeConversion. Unprobed in the corpus; assumption
//! `fieldref-setrange-scalar-on-text` (a partner probe on BC 29: `SetRange(0)` filters `0`, `SetRange(1.5)` `1.5`,
//! `SetRange(true)` `YES` on a Code field, `SetRange(0, 5)` `0..5`).

use al_builtins::assumptions::{Assumption, assume};
use al_builtins::native::Signal;
use al_builtins::value::{RtType, Value};

use crate::Session;

impl Session {
    /// Replaces each scalar bound of a `SetRange` on a Text or Code field (`ty`) by its `Format` text.
    pub(super) fn fieldref_setrange_scalars_as_text(&self, ty: &RtType, bounds: &mut [Value]) -> Result<(), Signal> {
        if !matches!(ty, RtType::Text(_) | RtType::Code(_)) {
            return Ok(());
        }
        for bound in bounds.iter_mut() {
            if !matches!(
                bound,
                Value::Integer(_) | Value::BigInteger(_) | Value::Decimal(_) | Value::Boolean(_)
            ) {
                continue;
            }
            assume(Assumption::FieldrefSetrangeScalarOnText);
            let text = al_builtins::format::try_format(bound, 0, 0, self).map_err(Signal::unsupported)?;
            *bound = Value::Text(text.into());
        }
        Ok(())
    }
}
