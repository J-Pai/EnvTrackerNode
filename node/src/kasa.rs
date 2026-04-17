//! Sets up and manages interactions with Kasa devices.
//! - Kasa Smart Power Strip HS300

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use kasa_core::Credentials;
use kasa_core::DeviceConfig;
use kasa_core::Transport;
use kasa_core::commands::INFO;
use kasa_core::commands::energy_for_child;
use kasa_core::connect;
use serde::Deserialize;
use serde::Serialize;
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
use crate::error::NodeError;

#[derive(Clone, Serialize, Deserialize)]
struct KasaDeviceChild {
    /// Human-readable name of the device.
    alias: String,
    /// On/Off state.
    state: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct EMeter {
    current_ma: u64,
    power_mw: u64,
    voltage_mv: u64,
    total_wh: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct KasaChildInfo {
    info: KasaDeviceChild,
    emeter: EMeter,
}

struct KasaDevice {
    topic: String,
    alias: String,
    transport: Arc<RwLock<Option<Box<dyn Transport>>>>,
    polling_schedule: String,
    mq: Arc<RwLock<MessageQueue>>,
    scheduler: Arc<RwLock<JobScheduler>>,
    /// Child Kasa devices keys is the id hash.
    children: Arc<RwLock<HashMap<String, KasaDeviceChild>>>,
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
            alias: String::new(),
            transport: Arc::new(RwLock::const_new(None)),
            polling_schedule,
            mq,
            scheduler,
            children: Arc::new(RwLock::const_new(HashMap::new())),
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

    async fn setup_system_info(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let transport = self.transport.clone();

        let response = transport
            .read()
            .await
            .as_ref()
            .unwrap()
            .send(INFO)
            .await
            .expect("System info inaccessible");
        let data: Value = serde_json::from_str(&response.as_str()).unwrap();

        let system = if let Value::Object(system) = data {
            system
        } else {
            return Err(NodeError::new("`system` parsing error"));
        };

        let system: &Value = system
            .get("system")
            .ok_or(NodeError::new("No `system` in response"))?;

        let get_sysinfo = if let Value::Object(get_sysinfo) = system {
            get_sysinfo
        } else {
            return Err(NodeError::new("`get_sysinfo` parsing error"));
        };

        let get_sysinfo: &Value = get_sysinfo
            .get("get_sysinfo")
            .ok_or(NodeError::new("No `get_sysinfo` in system"))?;

        self.alias = get_sysinfo
            .get("alias")
            .ok_or(NodeError::new("No `alias` in get_sysinfo"))?
            .as_str()
            .unwrap()
            .to_string();

        let children = get_sysinfo
            .get("children")
            .ok_or(NodeError::new("No `children` in get_sysinfo"))?
            .as_array()
            .ok_or(NodeError::new("`children` parsing error"))?;

        for child in children {
            tracing::debug!("Child: {:#?}", child);

            let c = child
                .as_object()
                .ok_or(NodeError::new("`child` parsing error"))?;

            let id = c
                .get("id")
                .ok_or(NodeError::new("No `id` in child"))?
                .as_str()
                .unwrap()
                .to_string();

            self.children.write().await.insert(
                id.clone(),
                KasaDeviceChild {
                    alias: c
                        .get("alias")
                        .ok_or(NodeError::new("No `alias` in child"))?
                        .as_str()
                        .unwrap()
                        .to_string(),
                    state: c
                        .get("state")
                        .ok_or(NodeError::new("No `alias` in child"))?
                        .as_i64()
                        .unwrap()
                        == 1,
                },
            );
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
        let children = self.children.clone();
        scheduler
            .add(Job::new_async(
                self.polling_schedule.clone(),
                move |_uuid, _l| {
                    Box::pin({
                        let transport = transport.clone();
                        let publisher = publisher.clone();
                        let children = children.clone();
                        async move {
                            let children = children.read().await;
                            let publisher = publisher.read().await;

                            let mut associated_data: Vec<KasaChildInfo> = Vec::new();

                            for (k, v) in children.iter() {
                                let response = transport
                                    .read()
                                    .await
                                    .as_ref()
                                    .unwrap()
                                    .send(&energy_for_child(k.as_str()))
                                    .await
                                    .expect("emeter info inaccessible");
                                let data: Value = serde_json::from_str(response.as_str()).unwrap();

                                let data = match data.get("emeter") {
                                    Some(data) => data,
                                    None => {
                                        tracing::warn!("Malformed emeter: {}", data);
                                        return;
                                    }
                                };

                                let data = match data.get("get_realtime") {
                                    Some(data) => data,
                                    None => {
                                        tracing::warn!("Malformed get_realtime: {}", data);
                                        return;
                                    }
                                };

                                associated_data.push(KasaChildInfo {
                                    info: v.clone(),
                                    emeter: EMeter {
                                        current_ma: data
                                            .get("current_ma")
                                            .unwrap()
                                            .as_u64()
                                            .unwrap(),
                                        power_mw: data.get("power_mw").unwrap().as_u64().unwrap(),
                                        voltage_mv: data
                                            .get("voltage_mv")
                                            .unwrap()
                                            .as_u64()
                                            .unwrap(),
                                        total_wh: data.get("total_wh").unwrap().as_u64().unwrap(),
                                    },
                                })
                            }

                            let data: Value = serde_json::from_str(
                                serde_json::to_string(&associated_data)
                                    .unwrap()
                                    .clone()
                                    .as_str(),
                            )
                            .unwrap();

                            publisher.publish(data).await.unwrap();
                            tracing::debug!("Published to: {}", publisher.topic());
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
            .expect(format!("Topic creation for [{}] failed", name).as_str())
            .setup_transport(device_config)
            .await
            .expect(format!("Transport creation for [{}] failed", name).as_str())
            .setup_system_info()
            .await
            .expect(format!("System Info extraction for [{}] failed", name).as_str());
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
