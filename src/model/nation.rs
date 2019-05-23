use crate::model::enums::{NationStatus, SubmissionStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct Nation {
    pub id: u32,
    pub status: NationStatus,
    pub submitted: SubmissionStatus,
    pub connected: bool,
    pub name: String,
    pub era: String,
}
