#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(u8)]
#[serde(untagged)]
pub enum SubmissionStatus {
    NotSubmitted = 0,
    PartiallySubmitted = 1,
    Submitted = 2,
}
impl SubmissionStatus {
    pub fn show(self) -> &'static str {
        match self {
            SubmissionStatus::NotSubmitted => ":x:",
            SubmissionStatus::PartiallySubmitted => ":alarm_clock:",
            SubmissionStatus::Submitted => ":white_check_mark:",
        }
    }

    pub fn from_int(x: u8) -> Self {
        match x {
            0 => SubmissionStatus::NotSubmitted,
            1 => SubmissionStatus::PartiallySubmitted,
            2 => SubmissionStatus::Submitted,
            _ => panic!(),
        }
    }
}
