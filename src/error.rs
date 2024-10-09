//! Some traits that all errors should implement

/// All errors should have an id that can be human readable
/// Id's should also be useful to deduplicate errors before presenting them to the user
pub trait HasId {
    fn id(&self) -> String;
}

/// Implemented for all vectors of items that implement `HasId`
pub trait VecHasIdExtensions {
    fn filter_by_excludes(self, excludes: Vec<String>) -> Self;
    fn dedupe_by_id(self) -> Self;
}

/// Used for filtering out items that start with the exclude id
impl<T: HasId> VecHasIdExtensions for Vec<T> {
    fn filter_by_excludes(mut self, excludes: Vec<String>) -> Self {
        self.retain(|item| {
            !excludes.iter().any(|exclude| {
                item.id()
                    .to_lowercase()
                    .starts_with(&exclude.to_lowercase())
            })
        });
        self
    }

    fn dedupe_by_id(mut self) -> Self {
        self.dedup_by(|a, b| a.id().to_lowercase() == b.id().to_lowercase());
        self
    }
}
