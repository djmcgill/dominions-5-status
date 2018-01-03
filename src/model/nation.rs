use model::enums::nation_status::NationStatus;
use model::enums::submission_status::SubmissionStatus;

pub struct Nation {
    pub id: usize,
    pub status: NationStatus,
    pub submitted: SubmissionStatus,
    pub connected: bool,
    pub name: String,
    pub era: String,
}
