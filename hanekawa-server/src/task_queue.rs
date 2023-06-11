use std::pin::Pin;

use hanekawa_common::{
    task::{Task, TaskQueue},
    Config,
};

use futures::{Stream, StreamExt};

pub struct AmqpTaskQueue {
    conn: lapin::Connection,
    chan: lapin::Channel,
}

impl AmqpTaskQueue {
    pub async fn start(cfg: &Config) -> Self {
        let conn = lapin::Connection::connect(&cfg.message_queue_url, Default::default())
            .await
            .unwrap();

        let chan = conn.create_channel().await.unwrap();

        let s = Self { conn, chan };

        s.initialize_topology().await;
        s
    }

    async fn initialize_topology(&self) {
        let _tasks = self
            .chan
            .queue_declare("tasks", Default::default(), Default::default())
            .await
            .unwrap();
    }
}

#[async_trait::async_trait]
impl TaskQueue for AmqpTaskQueue {
    async fn enqueue(&self, task: &dyn hanekawa_common::task::Task) -> Option<()> {
        let payload = serde_json::to_string(task).unwrap();

        self.chan
            .basic_publish(
                "",
                "tasks",
                Default::default(),
                payload.as_bytes(),
                Default::default(),
            )
            .await
            .unwrap();

        Some(())
    }

    async fn consume(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Box<dyn hanekawa_common::task::Task>> + Send>> {
        let consumer = self
            .chan
            .basic_consume("tasks", "", Default::default(), Default::default())
            .await
            .unwrap();

        let stream = consumer.then(|delivery| async move {
            let delivery = delivery.unwrap();
            let payload: Box<dyn Task> = serde_json::from_slice(&delivery.data).unwrap();

            payload
        });

        Box::pin(stream)
    }
}
