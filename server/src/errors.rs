pub type RoutinesResult<T> = Result<T, RoutinesError>;

#[derive(Debug, thiserror::Error)]
pub enum RoutinesError {
    #[error("{0}")]
    Storage(#[from] sqlx::Error),
}
