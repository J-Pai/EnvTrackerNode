//! Sets up and manages interactions with Kasa devices.
//! - Kasa Smart Power Strip HS300

use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;

use chrono::DateTime;
use chrono::Local;
use kasa_core::Credentials;
use kasa_core::DeviceConfig;
use kasa_core::Transport;
use kasa_core::commands::INFO;
use kasa_core::connect;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::AsyncMessagePublisher;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageQueue;
use tokio_memq::Publisher;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;

use crate::config::KasaDeviceConfig;
use crate::traits::Subscribale;

struct KasaDevice {
    topic: String,
    transport_index: Option<usize>,
    publisher_indices: Vec<usize>,
    subscriber_indices: Vec<usize>,
    polling_schedule: String,
}

impl KasaDevice {
    fn new(topic: String, polling_schedule: String) -> Self {
        let device: Self = Self {
            topic,
            transport_index: None,
            publisher_indices: Vec::new(),
            subscriber_indices: Vec::new(),
            polling_schedule,
        };
        device
    }

    async fn setup_topic(
        self,
        mq: &'static Mutex<Option<MessageQueue>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mq_lock = mq.lock().await;
        let mq = mq_lock.as_ref().unwrap();
        mq.create_topic(
            "kasa".to_string(),
            TopicOptions {
                // Should track data for up to 3 months.
                max_messages: Some(Duration::as_secs(&Duration::from_hours(24 * 90)) as usize),
                ..Default::default()
            },
        )
        .await?;
        Ok(self)
    }

    async fn setup_transport(
        mut self,
        config: &KasaDeviceConfig,
        transports: &'static Mutex<Vec<Box<dyn Transport>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let transport_config = DeviceConfig::new(config.ip.as_str()).with_credentials(
            Credentials::new(config.username.as_str(), config.password.as_str()),
        );

        {
            let mut transports_lock = transports.lock().await;
            let index = transports_lock.len();
            transports_lock.push(connect(transport_config).await?);
            self.transport_index = Some(index);
        }

        Ok(self)
    }

    async fn allocate_publisher(
        &mut self,
        mq: &MessageQueue,
        publishers: &'static Mutex<Vec<Mutex<Option<Publisher>>>>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut publishers = publishers.lock().await;
        publishers.push(Mutex::new(Some(mq.publisher(self.topic.clone()))));
        self.publisher_indices.push(publishers.len() - 1);
        Ok(publishers.len() - 1)
    }

    async fn add_polling(
        &mut self,
        mq: &'static Mutex<Option<MessageQueue>>,
        publishers: &'static Mutex<Vec<Mutex<Option<Publisher>>>>,
        scheduler: &'static Mutex<Option<JobScheduler>>,
        transports: &'static Mutex<Vec<Box<dyn Transport>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mq = mq.lock().await;
        let publisher_index = self
            .allocate_publisher(&mq.as_ref().unwrap(), &publishers)
            .await?;
        let scheduler = scheduler.lock().await;
        let index = self.transport_index.unwrap();
        scheduler
            .as_ref()
            .unwrap()
            .add(Job::new_async(
                self.polling_schedule.clone(),
                move |_uuid, _l| {
                    Box::pin(async move {
                        let system_time = SystemTime::now();
                        let datetime: DateTime<Local> = system_time.into();
                        println!("[{}] Sampling...", datetime.format("%d/%m/%Y %T"));
                        let response = transports.lock().await[index]
                            .send(INFO)
                            .await
                            .expect("Something went wrong with sampling.");
                        let data: Value = serde_json::from_str(&response.as_str()).unwrap();
                        let publishers = publishers.lock().await;
                        let mut publisher_lock = publishers[publisher_index].lock().await;
                        let publisher = publisher_lock.take().unwrap();
                        publisher.publish(data).await.unwrap();
                        publisher_lock.replace(publisher);
                    })
                },
            )?)
            .await?;
        Ok(())
    }
}

impl Subscribale for KasaDevice {
    async fn allocate_subscriber(
            &mut self,
            mq: &MessageQueue,
            subscribers: &'static Mutex<Vec<Mutex<Option<Subscriber>>>>,
            options: TopicOptions,
            mode: ConsumptionMode,
        ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut subscribers = subscribers.lock().await;
        subscribers.push(Mutex::new(Some(
            mq.subscriber_with_options_and_mode(self.topic.clone(), options, mode)
                .await?,
        )));
        self.subscriber_indices.push(subscribers.len() - 1);
        Ok(subscribers.len() - 1)
    }

}

pub(crate) struct Kasa {
    devices: HashMap<String, KasaDevice>,
}

impl Kasa {
    pub(crate) async fn new(
        config: &HashMap<String, KasaDeviceConfig>,
        mq: &'static Mutex<Option<MessageQueue>>,
        transports: &'static Mutex<Vec<Box<dyn Transport>>>,
    ) -> Self {
        let mut kasa = Self {
            devices: HashMap::new(),
        };
        for (name, device_config) in config {
            let device = KasaDevice::new(name.to_owned(), "1/1 * * * * *".to_string())
                .setup_topic(mq)
                .await
                .expect(format!("Topic creation for [{}] failed.", name).as_str())
                .setup_transport(device_config, transports)
                .await
                .expect(format!("Transport creation for [{}] failed.", name).as_str());
            kasa.devices.insert(name.clone(), device);
        }

        kasa
    }

    pub(crate) async fn add_polling(
        &mut self,
        mq: &'static Mutex<Option<MessageQueue>>,
        publishers: &'static Mutex<Vec<Mutex<Option<Publisher>>>>,
        scheduler: &'static Mutex<Option<JobScheduler>>,
        transports: &'static Mutex<Vec<Box<dyn Transport>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (_, device) in &mut self.devices {
            device
                .add_polling(mq, publishers, scheduler, transports)
                .await?;
        }
        Ok(())
    }
}
