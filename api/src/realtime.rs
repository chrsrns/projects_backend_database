use rocket::tokio::sync::broadcast;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ResumeChangedEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub resume_id: i32,
    pub action: ResumeChangedAction,
}

#[derive(Clone)]
pub struct Hub {
    inner: Arc<HubInner>,
}

struct HubInner {
    topics: RwLock<HashMap<i32, broadcast::Sender<ResumeChangedEvent>>>,
    buffer: usize,
}

impl Hub {
    pub fn new() -> Self {
        Hub {
            inner: Arc::new(HubInner {
                topics: RwLock::new(HashMap::new()),
                buffer: 128,
            }),
        }
    }

    fn sender_for(&self, resume_id: i32) -> broadcast::Sender<ResumeChangedEvent> {
        {
            let guard = self
                .inner
                .topics
                .read()
                .expect("realtime hub topics read lock");

            if let Some(sender) = guard.get(&resume_id) {
                return sender.clone();
            }
        }

        let mut guard = self
            .inner
            .topics
            .write()
            .expect("realtime hub topics write lock");

        if let Some(sender) = guard.get(&resume_id) {
            return sender.clone();
        }

        let (sender, _) = broadcast::channel(self.inner.buffer);
        guard.insert(resume_id, sender.clone());
        sender
    }

    pub fn subscribe(&self, resume_id: i32) -> broadcast::Receiver<ResumeChangedEvent> {
        self.sender_for(resume_id).subscribe()
    }

    pub fn publish_resume_changed(&self, resume_id: i32, action: ResumeChangedAction) {
        let sender = self.sender_for(resume_id);
        let _ = sender.send(ResumeChangedEvent {
            kind: "resume.changed".to_string(),
            resume_id,
            action,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_then_publish_delivers_event() {
        let hub = Hub::new();
        let mut rx = hub.subscribe(123);

        hub.publish_resume_changed(123, ResumeChangedAction::Updated(SectionType::PersonalInfo));

        let evt = rx.try_recv().expect("event should be delivered");
        assert_eq!(evt.resume_id, 123);
        assert_eq!(
            evt.action,
            ResumeChangedAction::Updated(SectionType::PersonalInfo)
        );
        assert_eq!(evt.kind, "resume.changed");
    }
}
