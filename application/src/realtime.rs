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
