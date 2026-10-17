use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SectionType {
    PersonalInfo,
    Education,
    Frameworks,
    Languages,
    Projects,
    Skills,
    Experience,
}

impl SectionType {
    /// The wire value of the section, which is also what a revision records as
    /// the action that produced it.
    pub fn as_str(&self) -> &'static str {
        match self {
            SectionType::PersonalInfo => "personalinfo",
            SectionType::Education => "education",
            SectionType::Frameworks => "frameworks",
            SectionType::Languages => "languages",
            SectionType::Projects => "projects",
            SectionType::Skills => "skills",
            SectionType::Experience => "experience",
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResumeChangedAction {
    Created,
    Updated(SectionType),
    Deleted,
}

/// Port through which use cases report resume mutations. The api layer
/// supplies the implementation (the WebSocket hub); use cases depend on
/// this trait so event selection lives next to the decision that
/// produced the change.
pub trait ResumeChangePublisher: Send + Sync {
    fn resume_changed(&self, resume_id: Uuid, action: ResumeChangedAction);
}
