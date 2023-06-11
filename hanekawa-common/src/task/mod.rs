#[typetag::serde(tag = "type")]
#[async_trait::async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self, ctx: &crate::Services) -> Option<()>;
}

#[async_trait::async_trait]
pub trait TaskQueue: Send + Sync {
    async fn enqueue(&self, task: &dyn Task) -> Option<()>;
}
