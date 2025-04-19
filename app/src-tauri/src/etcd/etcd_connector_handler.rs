use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::transport::event::{DisconnectCase, SessionDisconnectedEvent};

use super::remove_connector;

#[derive(Clone, Default)]
pub struct EtcdConnectorHandler {
    app_handle: Option<AppHandle>,
    session_id: i32,
}

impl EtcdConnectorHandler {
    pub fn new(app_handle: AppHandle, session_id: i32) -> Self {
        Self {
            app_handle: Some(app_handle),
            session_id,
        }
    }

    pub fn disconnected(&self, case: DisconnectCase) {
        if let Some(app) = self.app_handle.as_ref() {
            let _ = app.emit_all(
                "sessionDisconnected",
                SessionDisconnectedEvent {
                    session_id: self.session_id,
                    case,
                },
            );
        }

        let session_id = self.session_id;
        if session_id != 0 {
            tokio::spawn(async move {
                remove_connector(&session_id).await;
            });
        }
    }
}
