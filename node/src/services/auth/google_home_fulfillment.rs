//! Handler for Google Home fulfillment requests.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::response::IntoResponse;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::jwt::DecodingKey;
use http::HeaderMap;
use http::StatusCode;
use http::header::AUTHORIZATION;
use tokio::sync::RwLock;

use crate::services::auth::Auth;
use crate::services::auth::ClientJsonWeb;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Intent {
    intent: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct FulfillmentRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    inputs: Vec<Intent>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Name {
    name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct LightColorTemperatureRange {
    #[serde(rename = "temperatureMinK")]
    temperature_min_k: i64,
    #[serde(rename = "temperatureMaxK")]
    temperature_max_k: i64,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Attributes {
    #[serde(rename = "colorTemperatureRange")]
    color_temperature_range: Option<LightColorTemperatureRange>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct DeviceInfo {
    manufacturer: String,
    model: String,
    #[serde(rename = "hwVersion")]
    hw_version: String,
    #[serde(rename = "swVersion")]
    sw_version: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Device {
    id: String,
    #[serde(rename = "type")]
    device_type: String,
    traits: Vec<String>,
    name: Name,
    #[serde(rename = "willReportState")]
    will_report_state: bool,
    attributes: Attributes,
    #[serde(rename = "deviceInfo")]
    device_info: DeviceInfo,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct Payload {
    #[serde(rename = "agentUserId")]
    agent_user_id: String,
    devices: Option<Vec<Device>>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct FulfillmentResponse {
    #[serde(rename = "requestId")]
    request_id: String,
    payload: Payload,
}

impl Auth {
    pub(super) async fn google_home_fulfillment_handler(
        headers: HeaderMap,
        Json(json): Json<FulfillmentRequest>,
        db: Arc<dyn AuthCache + Send + Sync>,
        certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
        client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        tracing::info!("HEADERS : {headers:#?}");

        let id = if let Some(bearer_token) = headers.get(AUTHORIZATION)
            && let Ok(bearer_token) = bearer_token.to_str()
        {
            let split = bearer_token.split(" ");
            let Some(bearer_token) = split.last() else {
                tracing::error!("No bearer token in authorization field.");
                return StatusCode::UNAUTHORIZED.into_response();
            };

            if let Ok(Some(auth_session)) = db
                .get_auth_session(&format!("google_home_auth_token|{bearer_token}"))
                .await
                && let Ok(id) =
                    Self::validate_session(certs, &auth_session, &[client_json.client_id]).await
            {
                id
            } else {
                tracing::error!("Bearer/Access Token not found. {bearer_token}");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        } else {
            tracing::error!("No authorization field.");
            return StatusCode::UNAUTHORIZED.into_response();
        };

        tracing::info!("[{}] JSON: {json:#?}", id.claims.email.unwrap());

        let sub = id.claims.sub;

        for inputs in &json.inputs {
            let intent = inputs.intent.clone();
            let request_id = json.request_id.clone();
            match intent.as_str() {
                "action.devices.SYNC" => {
                    let response = FulfillmentResponse {
                        request_id,
                        payload: Payload {
                            agent_user_id: sub.clone(),
                            devices: Some(vec![Device {
                                id: format!("{}_dlight", sub.clone()),
                                device_type: "action.devices.types.LIGHT".to_string(),
                                traits: [
                                    "action.devices.traits.ColorSetting".to_string(),
                                    "action.devices.traits.Brightness".to_string(),
                                    "action.devices.traits.OnOff".to_string(),
                                ]
                                .to_vec(),
                                name: Name {
                                    name: "glamp".to_string(),
                                },
                                will_report_state: false,
                                attributes: Attributes {
                                    color_temperature_range: Some(LightColorTemperatureRange {
                                        temperature_min_k: 2600,
                                        temperature_max_k: 6000,
                                    }),
                                },
                                device_info: DeviceInfo {
                                    manufacturer: "Me".to_string(),
                                    model: "dLight".to_string(),
                                    hw_version: "1".to_string(),
                                    sw_version: "1".to_string(),
                                },
                            }]),
                        },
                    };

                    return Json(response).into_response();
                }
                _ => {
                    let response = FulfillmentResponse {
                        request_id,
                        payload: Payload {
                            agent_user_id: sub.clone(),
                            devices: None,
                        },
                    };
                    return Json(response).into_response();
                }
            }
        }

        Json("{}").into_response()
    }
}
