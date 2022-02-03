use std::{error::Error, time::Duration};

use jage_api::instrument::instrument_client::InstrumentClient;
use tonic::transport::Uri;

use crate::client::JageClient;

pub struct Connection;

impl Connection {
    const BACKOFF: Duration = Duration::from_millis(500);
    const MAX_BACKOFF: Duration = Duration::from_secs(5);

    pub async fn connect(uri: Uri) -> JageClient {
        let mut backoff = Duration::from_secs(0);
        loop {
            if backoff == Duration::from_secs(0) {
                tracing::debug!(to = %uri, "connecting");
            } else {
                tracing::debug!(reconnect_in = ?backoff, "reconnecting");
                tokio::time::sleep(backoff).await;
            }

            let try_connect = async {
                let client = InstrumentClient::connect(uri.clone()).await?;
                Ok::<InstrumentClient<_>, Box<dyn Error>>(client)
            };

            match try_connect.await {
                Ok(connected_client) => {
                    tracing::debug!("connected successfully!");
                    return JageClient::new(connected_client);
                }
                Err(error) => {
                    tracing::warn!(%error, "error connecting");
                    backoff = std::cmp::max(backoff + Self::BACKOFF, Self::MAX_BACKOFF);
                }
            };
        }
    }
}
