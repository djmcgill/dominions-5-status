use model::enums::{NationStatus, SubmissionStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct Nation {
    pub id: usize,
    pub status: NationStatus,
    pub submitted: SubmissionStatus,
    pub connected: bool,
    pub name: String,
    pub era: String,
}
