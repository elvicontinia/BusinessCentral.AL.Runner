//! `Get()."Field" := value`: an assignment to a field of the record a procedure returns.
use super::*;

impl Lowerer<'_> {
    /// The record a procedure returns is a copy, so the assignment goes to a hidden local that nothing reads again; the call
    /// still runs and a value that does not fit the field still raises. Continia's Delivery Network extended tests write
    /// `Factory.GetEOrderHeader()."Change Sequence Number" := ...` (40 tests were refused with "is not assignable").
    /// Not pinned by the corpus (docs/probes-needed.md).
    pub(crate) fn assign_to_returned_record(
        &mut self,
        target: &SyntaxNode,
        source: &SyntaxNode,
    ) -> L<Option<ir::Stmt>> {
        let Some(base) = target.children().next() else {
            return Ok(None);
        };
        if target.kind() != SyntaxKind::MemberExpr || base.kind() != SyntaxKind::CallExpr {
            return Ok(None);
        }
        let record_ty = self.ty_ref(&base);
        if !matches!(record_ty, Ty::Record { .. }) {
            return Ok(None);
        }
        let Some(Res::Field { table, id }) = self.res(target) else {
            return Ok(None);
        };
        let (table, id) = (*table, *id);
        self.assume(al_builtins::assumptions::Assumption::AssignFieldOfReturnedRecord);
        let slot_ty = self.rt(record_ty)?;
        let call = self.expr(&base)?;
        let value = self.expr(source)?;
        let value = self.convert_assigned(self.ty_ref(target), self.ty_ref(source), value)?;
        let copy = self.hidden_base + self.hidden_locals.len();
        self.hidden_locals.push(ir::Slot::new("__returned_record", slot_ty));
        let field = ir::Place::Field(Box::new(ir::Place::Local(copy)), self.field_ref(table, id));
        Ok(Some(ir::Stmt::If(
            ir::Expr::Const(ir::Const::Boolean(true)),
            vec![
                ir::Stmt::Assign(ir::Place::Local(copy), call),
                ir::Stmt::Assign(field, value),
            ],
            Vec::new(),
        )))
    }
}
