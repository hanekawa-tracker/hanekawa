use std::sync::Arc;

use hanekawa_common::{
    task::{Task, TaskQueue},
    Config, Services,
};

use futures::{Stream, StreamExt};
use lapin::{Channel, Connection};
use tokio_util::sync::CancellationToken;

pub struct AmqpMessage {
    content: Box<dyn Task>,
    delivery: lapin::message::Delivery,
}

impl AmqpMessage {
    pub fn content(&self) -> &dyn Task {
        self.content.as_ref()
    }

    pub async fn ack(self) {
        self.delivery.acker.ack(Default::default()).await.unwrap();
    }

    pub async fn nack(&self) {
        self.delivery.acker.nack(Default::default()).await.unwrap();
    }
}

#[derive(Clone)]
pub struct QueueConnection {
    inner: Arc<Connection>,
}

impl QueueConnection {
    pub async fn connect(cfg: &Config) -> QueueConnection {
        let conn = lapin::Connection::connect(&cfg.message_queue_url, Default::default())
            .await
            .unwrap();

        let inner = Arc::new(conn);

        QueueConnection { inner }
    }
}

pub struct AmqpTaskQueue {
    chan: lapin::Channel,
}

impl AmqpTaskQueue {
    pub async fn new(connection: QueueConnection) -> Self {
        let chan = connection.inner.create_channel().await.unwrap();

        let s = Self { chan };

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

    pub async fn consume(&self) -> impl Stream<Item = AmqpMessage> {
        let consumer = self
            .chan
            .basic_consume("tasks", "", Default::default(), Default::default())
            .await
            .unwrap();

        let stream = consumer.then(|delivery| async move {
            let delivery = delivery.unwrap();
            let payload: Box<dyn Task> = serde_json::from_slice(&delivery.data).unwrap();

            AmqpMessage {
                content: payload,
                delivery,
            }
        });

        Box::pin(stream)
    }
}

#[derive(Clone)]
pub struct BackgroundTaskService {
    services: Services,
    chan: Channel,
}

impl BackgroundTaskService {
    pub async fn new(conn: QueueConnection, services: Services) -> Self {
        let chan = conn.inner.create_channel().await.unwrap();

        Self { services, chan }
    }

    pub async fn run(self, kt: CancellationToken) {
        let task_queue = AmqpTaskQueue { chan: self.chan };

        let mut consumer = task_queue.consume().await;

        loop {
            tokio::select! {
                _ = kt.cancelled() => break,
                Some(message) = consumer.next() => {
                    eprintln!("processing a task..");
                    let task = message.content();

                    let result = task.execute(&self.services)
                    .await;

                    if result.is_some() {
                        message.ack().await;
                    } else {
                        message.nack().await;
                    }
                }
            }
        }
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
}
