//! Sets up and manages interactions with Kasa devices.
//! - Kasa Smart Power Strip HS300

use std::collections::HashMap;
use std::sync::Arc;
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
    transport: Arc<RwLock<Option<Box<dyn Transport>>>>,
    polling_schedule: String,
    mq: Arc<RwLock<MessageQueue>>,
    scheduler: Arc<RwLock<JobScheduler>>,
}

impl KasaDevice {
    async fn new(
        topic: String,
        polling_schedule: String,
        mq: Arc<RwLock<MessageQueue>>,
        scheduler: Arc<RwLock<JobScheduler>>,
    ) -> Self {
        let device: Self = Self {
            topic,
            transport: Arc::new(RwLock::const_new(None)),
            polling_schedule,
            mq,
            scheduler,
        };
        device
    }

    async fn setup_topic(self) -> Result<Self, Box<dyn std::error::Error>> {
        let mq_lock = self.mq.clone();
        let mq = mq_lock.read().await;
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
        self,
        config: &KasaDeviceConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let transport_config = DeviceConfig::new(config.ip.as_str()).with_credentials(
            Credentials::new(config.username.as_str(), config.password.as_str()),
        );

        {
            let mut transport_lock = self.transport.write().await;
            transport_lock.replace(connect(transport_config).await?);
        }

        Ok(self)
    }

    async fn allocate_publisher(
        &mut self,
    ) -> Result<Arc<RwLock<Publisher>>, Box<dyn std::error::Error>> {
        let mq = self.mq.read().await;
        Ok(Arc::new(RwLock::new(mq.publisher(self.topic.clone()))))
    }

    async fn add_polling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let publisher = self.allocate_publisher().await?.clone();
        let scheduler = self.scheduler.read().await;
        let transport = self.transport.clone();
        scheduler
            .add(Job::new_async(
                self.polling_schedule.clone(),
                move |_uuid, _l| {
                    Box::pin({
                        let transport = transport.clone();
                        let publisher = publisher.clone();
                        async move {
                            let response = transport
                                .read()
                                .await
                                .as_ref()
                                .unwrap()
                                .send(INFO)
                                .await
                                .expect("Something went wrong with sampling.");
                            let data: Value = serde_json::from_str(&response.as_str()).unwrap();
                            let publisher = publisher.read().await;
                            publisher.publish(data).await.unwrap();
                            tracing::debug!("Sampled: {}", publisher.topic());
                        }
                    })
                },
            )?)
            .await?;
        Ok(())
    }

    async fn allocate_subscriber(
        &mut self,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<Arc<RwLock<Subscriber>>, Box<dyn std::error::Error>> {
        let mq = self.mq.read().await;
        Ok(Arc::new(RwLock::new(
            mq.subscriber_with_options_and_mode(self.topic.clone(), options, mode)
                .await?,
        )))
    }
}

pub(crate) struct Kasa {
    devices: HashMap<String, KasaDevice>,
}

impl Kasa {
    pub(crate) async fn new(
        config: &HashMap<String, KasaDeviceConfig>,
        mq: Arc<RwLock<MessageQueue>>,
        scheduler: Arc<RwLock<JobScheduler>>,
    ) -> Self {
        let mut kasa = Self {
            devices: HashMap::new(),
        };
        for (name, device_config) in config {
            let device = KasaDevice::new(
                name.to_owned(),
                device_config.polling_schedule.clone(),
                mq.clone(),
                scheduler.clone(),
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
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<Arc<RwLock<Subscriber>>, Box<dyn std::error::Error>> {
        self.devices
            .get_mut(&device)
            .unwrap()
            .allocate_subscriber(options, mode)
            .await
    }
}
