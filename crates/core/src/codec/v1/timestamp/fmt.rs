use super::*;
use crate::utils::Hexed;
use core::fmt;

impl<A: Allocator + Clone> fmt::Display for Timestamp<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_recurse(None, f, 0, true)
    }
}

impl<A: Allocator + Clone> Timestamp<A> {
    pub(crate) fn fmt(&self, input: Option<&[u8]>, f: &mut fmt::Formatter) -> fmt::Result {
        let input = match input {
            Some(input) => Some(input),
            None => match self {
                Self::Step(step) => step.input.get().map(|v| v.as_slice()),
                Self::Attestation(_) => None,
            },
        };
        self.fmt_recurse(input, f, 0, true)
    }

    fn fmt_recurse(
        &self,
        input: Option<&[u8]>,
        f: &mut fmt::Formatter,
        depth: usize,
        first_line: bool,
    ) -> fmt::Result {
        fn indent(f: &mut fmt::Formatter, depth: usize, first_line: bool) -> fmt::Result {
            if depth == 0 {
                return Ok(());
            }

            for _ in 0..depth - 1 {
                f.write_str("    ")?;
            }
            if first_line {
                f.write_str("--->")?;
            } else {
                f.write_str("    ")?;
            }
            Ok(())
        }

        indent(f, depth, first_line)?;
        match self {
            Self::Step(step) if step.op == OpCode::FORK => {
                writeln!(f, "fork")?;
                for child in &step.next {
                    child.fmt_recurse(input, f, depth + 1, true)?;
                }
                Ok(())
            }
            Self::Step(step) => {
                let op = step.op;
                if op.has_immediate() {
                    writeln!(f, "execute {op} {}", Hexed(&step.data))?;
                } else {
                    writeln!(f, "execute {op}")?;
                }

                let result = if let Some(value) = step.next.first().and_then(|next| next.input()) {
                    Some(value.to_vec_in(step.allocator().clone()))
                } else if let Some(input) = input {
                    let result = op.execute_in(input, &step.data, step.allocator().clone());
                    indent(f, depth, false)?;
                    writeln!(f, " result {}", Hexed(&result))?;
                    Some(result)
                } else {
                    None
                };

                for child in &step.next {
                    child.fmt_recurse(result.as_deref(), f, depth, false)?;
                }
                Ok(())
            }
            Self::Attestation(attestation) => {
                writeln!(f, "result attested by {attestation}")
            }
        }
    }
}
