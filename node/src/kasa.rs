//! Sets up and manages interactions with Kasa devices.
//! - Kasa Smart Power Strip HS300

use std::collections::HashMap;
use std::time::Duration;

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

struct KasaDevice {
    topic: String,
    transports: (&'static RwLock<Vec<Box<dyn Transport>>>, Option<usize>),
    publishers: (&'static RwLock<Vec<RwLock<Publisher>>>, Vec<usize>),
    subscribers: (&'static RwLock<Option<HashMap<String, RwLock<Subscriber>>>>, Vec<String>),
    polling_schedule: String,
    mq: &'static RwLock<Option<MessageQueue>>,
    scheduler: &'static RwLock<Option<JobScheduler>>,
}

impl KasaDevice {
    async fn new(
        topic: String,
        polling_schedule: String,
        mq: &'static RwLock<Option<MessageQueue>>,
        scheduler: &'static RwLock<Option<JobScheduler>>,
    ) -> Self {
        static TRANSPORTS: RwLock<Vec<Box<dyn Transport>>> = RwLock::const_new(Vec::new());

        static PUBLISHERS: RwLock<Vec<RwLock<Publisher>>> = RwLock::const_new(Vec::new());

        static SUBSCRIBERS: RwLock<Option<HashMap<String, RwLock<Subscriber>>>> = RwLock::const_new(None);
        {
            let mut subscribers_lock = SUBSCRIBERS.write().await;
            if subscribers_lock.is_none() {
                subscribers_lock.replace(HashMap::new());
            }
        }

        let device: Self = Self {
            topic,
            transports: (&TRANSPORTS, None),
            publishers: (&PUBLISHERS, Vec::new()),
            subscribers: (&SUBSCRIBERS, Vec::new()),
            polling_schedule,
            mq,
            scheduler,
        };
        device
    }

    async fn setup_topic(self) -> Result<Self, Box<dyn std::error::Error>> {
        let mq_lock = self.mq.read().await;
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
            let mut transports_lock = self.transports.0.write().await;
            let index = transports_lock.len();
            transports_lock.push(connect(transport_config).await?);
            self.transports.1 = Some(index);
        }

        Ok(self)
    }

    async fn allocate_publisher(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mq = self.mq.read().await;
        let mut publishers = self.publishers.0.write().await;
        publishers.push(RwLock::new(
            mq.as_ref().unwrap().publisher(self.topic.clone()),
        ));
        self.publishers.1.push(publishers.len() - 1);
        Ok(publishers.len() - 1)
    }

    async fn add_polling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let publisher_index = self.allocate_publisher().await?.clone();
        let scheduler = self.scheduler.read().await;
        let transports = self.transports;
        let index = self.transports.1.unwrap();
        let publishers = self.publishers.0;
        scheduler
            .as_ref()
            .unwrap()
            .add(Job::new_async(
                self.polling_schedule.clone(),
                move |_uuid, _l| {
                    Box::pin(async move {
                        let response = transports.0.read().await[index]
                            .send(INFO)
                            .await
                            .expect("Something went wrong with sampling.");
                        let data: Value = serde_json::from_str(&response.as_str()).unwrap();
                        let publishers = publishers.read().await;
                        let publisher = publishers[publisher_index].read().await;
                        publisher.publish(data).await.unwrap();
                        tracing::debug!("Sampled: {}", publisher.topic());
                    })
                },
            )?)
            .await?;
        Ok(())
    }

    async fn allocate_subscriber(
        &mut self,
        key: &String,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<&'static RwLock<Option<HashMap<String, RwLock<Subscriber>>>>, Box<dyn std::error::Error>> {
        let mq = self.mq.read().await;
        let mut subscribers_lock = self.subscribers.0.write().await;
        let mut subscribers = subscribers_lock.take().expect("No subscribers map.");
        subscribers.insert(key.clone(), RwLock::new(
            mq.as_ref()
                .unwrap()
                .subscriber_with_options_and_mode(self.topic.clone(), options, mode)
                .await?,
        ));
        subscribers_lock.replace(subscribers);
        self.subscribers.1.push(key.clone());
        Ok(self.subscribers.0)
    }
}

pub(crate) struct Kasa {
    devices: HashMap<String, KasaDevice>,
}

impl Kasa {
    pub(crate) async fn new(
        config: &HashMap<String, KasaDeviceConfig>,
        mq: &'static RwLock<Option<MessageQueue>>,
        scheduler: &'static RwLock<Option<JobScheduler>>,
    ) -> Self {
        let mut kasa = Self {
            devices: HashMap::new(),
        };
        for (name, device_config) in config {
            let device = KasaDevice::new(
                name.to_owned(),
                device_config.polling_schedule.clone(),
                mq,
                scheduler,
            )
            .await
            .setup_topic()
            .await
            .expect(format!("Topic creation for [{}] failed.", name).as_str())
            .setup_transport(device_config)
            .await
            .expect(format!("Transport creation for [{}] failed.", name).as_str());
            kasa.devices.insert(name.clone(), device);
        }

        kasa
    }

    pub(crate) async fn add_polling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (_, device) in &mut self.devices {
            device.add_polling().await?;
        }
        Ok(())
    }

    pub(crate) async fn allocate_subscriber(
        &mut self,
        device: String,
        key: String,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<&'static RwLock<Option<HashMap<String, RwLock<Subscriber>>>>, Box<dyn std::error::Error>> {
        self.devices
            .get_mut(&device)
            .unwrap()
            .allocate_subscriber(&key, options, mode)
            .await
    }
}
