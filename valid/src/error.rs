use std::fmt::{Debug, Display};

use super::Cause;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct ValidationError<E>(Vec<Cause<E>>);

impl<E: Display> Display for ValidationError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validation Error\n")?;
        let errors = self.as_vec();
        for error in errors {
            f.write_str(format!("{} {}", '\u{2022}', error.message).as_str())?;
            if !error.trace.is_empty() {
                f.write_str(
                    &(format!(
                        " [{}]",
                        error
                            .trace
                            .iter()
                            .cloned()
                            .collect::<Vec<String>>()
                            .join(", ")
                    )),
                )?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl<E> ValidationError<E> {
    pub fn as_vec(&self) -> &Vec<Cause<E>> {
        &self.0
    }

    pub fn combine(mut self, mut other: ValidationError<E>) -> ValidationError<E> {
        self.0.append(&mut other.0);
        self
    }

    pub fn empty() -> Self {
        ValidationError(Vec::new())
    }

    pub fn new(e: E) -> Self {
        ValidationError(vec![Cause::new(e)])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn trace(self, message: &str) -> Self {
        let mut errors = self.0;
        for cause in errors.iter_mut() {
            cause.trace.insert(0, message.to_owned());
        }
        Self(errors)
    }

    pub fn append(self, error: E) -> Self {
        let mut errors = self.0;
        errors.push(Cause::new(error));
        Self(errors)
    }

    pub fn transform<E1>(self, f: &impl Fn(E) -> E1) -> ValidationError<E1> {
        ValidationError(self.0.into_iter().map(|cause| cause.transform(f)).collect())
    }
}

impl<E: Display + Debug> std::error::Error for ValidationError<E> {}

impl<E> From<Cause<E>> for ValidationError<E> {
    fn from(value: Cause<E>) -> Self {
        ValidationError(vec![value])
    }
}

impl<E> From<Vec<Cause<E>>> for ValidationError<E> {
    fn from(value: Vec<Cause<E>>) -> Self {
        ValidationError(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;

    use crate::{Cause, ValidationError};

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Foo {
        a: i32,
    }

    #[test]
    fn test_error_display_formatting() {
        let error = ValidationError::from(vec![
            Cause::new("1").trace(vec!["a", "b"]),
            Cause::new("2"),
            Cause::new("3"),
        ]);
        let expected_output = "\
        |Validation Error
        |• 1 [a, b]
        |• 2
        |• 3
        |"
        .strip_margin();
        assert_eq!(format!("{}", error), expected_output);
    }
}
