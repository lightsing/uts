use super::*;
use crate::{
    codec::v1::opcode::{OpCode, OperationBuffer},
    utils::Hexed,
};
use std::fmt;

pub(crate) fn fmt(
    timestamp: &Timestamp,
    input: Option<&OperationBuffer>,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    fmt_recurse(
        &timestamp,
        input,
        &timestamp.steps.last().unwrap(),
        f,
        0,
        true,
    )
}

fn fmt_recurse(
    timestamp: &Timestamp,
    input: Option<&OperationBuffer>,
    step: &Step,
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
    match step.opcode {
        OpCode::FORK => {
            writeln!(f, "fork")?;
            let mut child_ptr = step.first_child;
            while let Some(child_idx) = resolve_ptr(child_ptr) {
                let child_step = &timestamp.steps[child_idx];
                fmt_recurse(timestamp, input, child_step, f, depth + 1, true)?;
                child_ptr = child_step.next_sibling;
            }
            Ok(())
        }
        OpCode::ATTESTATION => {
            // SAFETY: caller ensures step is attestation step
            let attest_idx = unsafe { timestamp.get_attest_idx(step) } as usize;
            let attest = &timestamp.attestations[attest_idx];

            writeln!(f, "result attested by {attest}")
        }
        op @ _ => {
            let data = unsafe { timestamp.get_step_data(step) };
            if op.has_immediate() {
                writeln!(f, "execute {op} {}", Hexed(&data))?;
            } else {
                writeln!(f, "execute {op}")?;
            }

            let result = if let Some(input) = input {
                let result = op.execute(&input, &data);
                indent(f, depth, false)?;
                writeln!(f, " result {}", Hexed(&result))?;
                Some(result)
            } else {
                None
            };

            if let Some(step_idx) = resolve_ptr(step.first_child) {
                let step = &timestamp.steps[step_idx];
                fmt_recurse(timestamp, result.as_ref(), step, f, depth, false)?;
            }
            Ok(())
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_recurse(self, None, &self.steps.last().unwrap(), f, 0, true)
    }
}
