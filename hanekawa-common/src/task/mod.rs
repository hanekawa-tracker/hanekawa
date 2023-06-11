#[typetag::serde]
#[async_trait::async_trait]
pub trait Task {
    fn queue_name(&self) -> &'static str;

    async fn execute(&self, ctx: &crate::Services) -> Option<()>;
}

#[async_trait::async_trait]
pub trait TaskQueue {
    async fn enqueue(&self, task: &dyn Task) -> Option<()>;
    async fn dequeue(&self) -> Box<dyn Task>;
}
