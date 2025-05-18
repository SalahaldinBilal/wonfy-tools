#[derive(Debug)]
pub struct UnknownError {
    pub name: String,
    pub value: String,
    pub expected: Vec<(Vec<String>, String)>,
}

impl std::fmt::Display for UnknownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unknown {} variant {}. \nexpected: ",
            self.name, self.value
        )?;

        for (items, name) in self.expected.iter() {
            write!(f, "\n- ")?;

            for (index, item) in items.iter().enumerate() {
                write!(
                    f,
                    "{}{}",
                    item,
                    if index == items.len() - 1 { "" } else { " | " }
                )?;
            }

            write!(f, " => {}.", name)?;
        }

        Ok(())
    }
}

impl std::error::Error for UnknownError {}

#[derive(Debug)]
pub struct MissingFieldError(pub String);

impl std::fmt::Display for MissingFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Did not set the field \"{}\" when building", self.0)
    }
}

macro_rules! unknown_error_expected {
    ($($head:literal $(| $tail:literal)* => $type:literal),*) => {
        vec![
            $(
                (
                    vec![$head.to_string() $(, $tail.to_string())*],
                    $type.to_string()
                )
            ),*
        ]
    };
}

pub(crate) use unknown_error_expected;
