//! This crate provides utilities for working with SQL databases.

mod alloy;
mod marcos;

/// Wrapper type for implementing sqlx Encode and Decode for types by converting them to and from text.
#[derive(Debug)]
pub struct TextWrapper<T>(pub T);
