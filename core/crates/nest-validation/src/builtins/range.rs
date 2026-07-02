//! Numeric range validator.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::Validator;

/// Validates that a number is within an inclusive range.
pub struct RangeValidator<T> {
    min: T,
    max: T,
}

impl<T> RangeValidator<T> {
    /// Creates a range validator for the given inclusive bounds.
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> Validator<T> for RangeValidator<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    fn validate(&self, value: &T, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if *value < self.min || *value > self.max {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.range"),
                format!("Value must be between {} and {}", self.min, self.max),
            )]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_range_passes() {
        let validator = RangeValidator::new(1, 10);
        assert!(validator.validate(&5, &ValidationContext::new()).is_empty());
    }

    #[test]
    fn out_of_range_fails() {
        let validator = RangeValidator::new(1, 10);
        assert!(!validator
            .validate(&11, &ValidationContext::new())
            .is_empty());
    }
}
