#[derive(Debug, PartialEq, Eq, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Started,
    Completed,
    Stopped,
}
