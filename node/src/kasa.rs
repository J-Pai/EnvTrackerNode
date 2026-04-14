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
use tokio::sync::RwLock;
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
    transports: &'static RwLock<Vec<Box<dyn Transport>>>,
    transport_index: Option<usize>,
    publishers: &'static RwLock<Vec<RwLock<Publisher>>>,
    publisher_indices: Vec<usize>,
    subscribers: &'static RwLock<Vec<RwLock<Subscriber>>>,
    subscriber_indices: Vec<usize>,
    polling_schedule: String,
}

impl KasaDevice {
    fn new(topic: String, polling_schedule: String) -> Self {
        static TRANSPORTS: RwLock<Vec<Box<dyn Transport>>> = RwLock::const_new(Vec::new());

        static PUBLISHERS: RwLock<Vec<RwLock<Publisher>>> = RwLock::const_new(Vec::new());

        static SUBSCRIBERS: RwLock<Vec<RwLock<Subscriber>>> = RwLock::const_new(Vec::new());

        let device: Self = Self {
            topic,
            transports: &TRANSPORTS,
            transport_index: None,
            publishers: &PUBLISHERS,
            publisher_indices: Vec::new(),
            subscribers: &SUBSCRIBERS,
            subscriber_indices: Vec::new(),
            polling_schedule,
        };
        device
    }

    async fn setup_topic(
        self,
        mq: &'static RwLock<Option<MessageQueue>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mq_lock = mq.read().await;
        let mq = mq_lock.as_ref().unwrap();
        mq.create_topic(
            self.topic.clone(),
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let transport_config = DeviceConfig::new(config.ip.as_str()).with_credentials(
            Credentials::new(config.username.as_str(), config.password.as_str()),
        );

        {
            let mut transports_lock = self.transports.write().await;
            let index = transports_lock.len();
            transports_lock.push(connect(transport_config).await?);
            self.transport_index = Some(index);
        }

        Ok(self)
    }

    async fn allocate_publisher(
        &mut self,
        mq: &'static RwLock<Option<MessageQueue>>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mq = mq.read().await;
        let mut publishers = self.publishers.write().await;
        publishers.push(RwLock::new(
            mq.as_ref().unwrap().publisher(self.topic.clone()),
        ));
        self.publisher_indices.push(publishers.len() - 1);
        Ok(publishers.len() - 1)
    }

    async fn add_polling(
        &mut self,
        mq: &'static RwLock<Option<MessageQueue>>,
        scheduler: &'static RwLock<Option<JobScheduler>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let publisher_index = self.allocate_publisher(mq).await?.clone();
        let scheduler = scheduler.read().await;
        let transports = self.transports;
        let publishers = self.publishers;
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
                        let response = transports.read().await[index]
                            .send(INFO)
                            .await
                            .expect("Something went wrong with sampling.");
                        let data: Value = serde_json::from_str(&response.as_str()).unwrap();
                        let publishers = publishers.read().await;
                        let publisher = publishers[publisher_index].read().await;
                        publisher.publish(data).await.unwrap();
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
        mq: &'static RwLock<Option<MessageQueue>>,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<(&'static RwLock<Vec<RwLock<Subscriber>>>, usize), Box<dyn std::error::Error>> {
        let index = {
            let mq = mq.read().await;
            let mut subscribers = self.subscribers.write().await;
            subscribers.push(RwLock::new(
                mq.as_ref()
                    .unwrap()
                    .subscriber_with_options_and_mode(self.topic.clone(), options, mode)
                    .await?,
            ));
            self.subscriber_indices.push(subscribers.len() - 1);
            subscribers.len() - 1
        };
        Ok((self.subscribers, index))
    }
}

pub(crate) struct Kasa {
    devices: HashMap<String, KasaDevice>,
}

impl Kasa {
    pub(crate) async fn new(
        config: &HashMap<String, KasaDeviceConfig>,
        mq: &'static RwLock<Option<MessageQueue>>,
    ) -> Self {
        let mut kasa = Self {
            devices: HashMap::new(),
        };
        for (name, device_config) in config {
            let device = KasaDevice::new(name.to_owned(), "1/1 * * * * *".to_string())
                .setup_topic(mq)
                .await
                .expect(format!("Topic creation for [{}] failed.", name).as_str())
                .setup_transport(device_config)
                .await
                .expect(format!("Transport creation for [{}] failed.", name).as_str());
            kasa.devices.insert(name.clone(), device);
        }

        kasa
    }

    pub(crate) async fn add_polling(
        &mut self,
        mq: &'static RwLock<Option<MessageQueue>>,
        scheduler: &'static RwLock<Option<JobScheduler>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (_, device) in &mut self.devices {
            device
                .add_polling(mq, scheduler)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn allocate_subscriber(
        &mut self,
        device: String,
        mq: &'static RwLock<Option<MessageQueue>>,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<(&'static RwLock<Vec<RwLock<Subscriber>>>, usize), Box<dyn std::error::Error>> {
        self.devices
            .get_mut(&device)
            .unwrap()
            .allocate_subscriber(mq, options, mode)
            .await
    }
}
