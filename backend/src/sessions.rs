use crate::{
    application::{AppError, AppResult, ErrorKind},
    domain::ConnectionInput,
};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

struct Session {
    input: ConnectionInput,
    expires: Instant,
}
#[derive(Default)]
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Session>>,
}
pub const TTL_SECONDS: u64 = 60 * 60 * 8;
impl SessionStore {
    pub fn purge_expired(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.retain(|_, session| session.expires > Instant::now());
        }
    }
    pub fn insert(&self, input: ConnectionInput, previous: Option<&str>) -> AppResult<String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError(ErrorKind::Internal, "Session unavailable."))?;
        sessions.retain(|_, s| s.expires > Instant::now());
        if let Some(id) = previous {
            sessions.remove(id);
        }
        if sessions.len() >= 32 {
            return Err(AppError(
                ErrorKind::Busy,
                "Too many connections. Disconnect a session and try again.",
            ));
        }
        let token = uuid::Uuid::new_v4().to_string();
        sessions.insert(
            token.clone(),
            Session {
                input,
                expires: Instant::now() + Duration::from_secs(TTL_SECONDS),
            },
        );
        Ok(token)
    }
    pub fn get(&self, token: Option<&str>) -> AppResult<ConnectionInput> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError(ErrorKind::Internal, "Session unavailable."))?;
        sessions.retain(|_, s| s.expires > Instant::now());
        token
            .and_then(|id| sessions.get(id))
            .map(|s| s.input.clone())
            .ok_or(AppError(
                ErrorKind::NotConnected,
                "Your session has ended. Connect again to continue.",
            ))
    }
    pub fn remove(&self, token: Option<&str>) {
        if let (Some(id), Ok(mut sessions)) = (token, self.sessions.lock()) {
            sessions.remove(id);
        }
    }
}
