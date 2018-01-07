#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum SubmissionStatus {
    NotSubmitted = 0,
    PartiallySubmitted = 1,
    Submitted = 2,
}
impl SubmissionStatus {
    pub fn show(self) -> &'static str {
        match self {
            SubmissionStatus::NotSubmitted => "X",
            SubmissionStatus::PartiallySubmitted => "/",
            SubmissionStatus::Submitted => "✓",
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
