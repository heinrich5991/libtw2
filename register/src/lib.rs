#![cfg(not(test))]

#[macro_use]
extern crate log;

use libtw2_packer::Unpacker;
#[allow(unused_imports)]
use libtw2_polyfill_1_63::OptionExt as _;
use libtw2_serverbrowse::protocol as browse_protocol;
use serde_derive::Deserialize;
use std::str;
use std::str::FromStr as _;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::pin;
use tokio::select;
use tokio::sync::futures::Notified;
use tokio::sync::Notify;
use tokio::time::sleep_until;
use tokio::time::Instant;
use uuid::Uuid;

mod ip_version;

use self::ip_version::IpVersion;
use self::ip_version::IP_VERSIONS;

const INTERVAL_HEARTBEAT: Duration = Duration::from_secs(15);
const INTERVAL_INFO_CHANGE: Duration = Duration::from_secs(1);
const INTERVAL_TOKEN_REQUIRED: Duration = Duration::from_secs(0);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "status")]
#[must_use]
enum RegisterResult {
    Success,
    NeedChallenge,
    NeedInfo,
    Error(RegisterError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct RegisterError {
    message: Arc<str>,
}

impl From<String> for RegisterError {
    fn from(message: String) -> RegisterError {
        RegisterError {
            message: message.into(),
        }
    }
}

async fn send_register(
    client: &reqwest::Client,
    url: &str,
    port: u16,
    info_serial: u64,
    info: Option<&str>,
    secret: &str,
    challenge_secret: &str,
    challenge_token: Option<&str>,
    community_token: Option<&str>,
) -> RegisterResult {
    let mut request = client
        .post(url)
        .header(
            "Address",
            format!("tw-0.6+udp://connecting-address.invalid:{port}"),
        )
        .header("Secret", secret)
        .header("Challenge-Secret", challenge_secret)
        .header("Info-Serial", info_serial);

    if let Some(info) = info {
        request = request
            .header("Content-Type", "application/json")
            .body(String::from(info));
    } else {
        request = request.header("Content-Length", 0);
    }
    if let Some(challenge_token) = challenge_token {
        request = request.header("Challenge-Token", challenge_token);
    }
    if let Some(community_token) = community_token {
        request = request.header("Community-Token", community_token);
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(err) => return RegisterResult::Error(err.to_string().into()),
    };
    let status = response.status();
    let body = match response.bytes().await {
        Ok(body) => body,
        Err(err) => return RegisterResult::Error(format!("receiving: {err}").into()),
    };
    debug!(
        "register: {} {}",
        status.as_u16(),
        String::from_utf8_lossy(&body).replace("\n", ""),
    );
    let result: RegisterResult = match serde_json::from_slice(&body) {
        Ok(result) => result,
        Err(err) => return RegisterResult::Error(format!("parsing: {err}").into()),
    };
    if status.as_u16() >= 400 && !matches!(result, RegisterResult::Error(_)) {
        return RegisterResult::Error(
            format!("non-success status code {status} from master without error code").into(),
        );
    }
    result
}

#[derive(Clone)]
struct RegisterData {
    info_serial: u64,
    info: Arc<str>,
    last_successful_info_serial: Option<u64>,
}

struct RegisterShared {
    data: Mutex<RegisterData>,
    port: u16,
    user_agent: Box<str>,
    register_url: Box<str>,
    community_token: Option<Box<str>>,
    challenge_packet_prefix: Box<[u8]>,
    challenge_secret: Box<str>,
    secret: Box<str>,
    period: Option<Duration>,
}

struct RegisterTaskShared {
    // If you want to have both the `RegisterShared` and this lock, take the
    // `RegisterShared` lock first, to avoid deadlocks.
    data: Mutex<RegisterTaskData>,
    ip_version: IpVersion,
    next_register_changed: Notify,
}

#[derive(Clone)]
struct RegisterTaskData {
    token: Option<Arc<str>>,
    prev_result: Option<RegisterResult>,
    prev_register: Instant,
    // `None` means that no further register calls are planned, barring outside
    // notifications.
    next_register: Option<Instant>,
}

impl RegisterData {
    fn on_success(&mut self, info_serial: u64) {
        if self
            .last_successful_info_serial
            .is_none_or(|serial| serial >= info_serial)
        {
            return;
        }
        self.last_successful_info_serial = Some(info_serial);
    }
    fn on_need_info(&mut self, info_serial: u64) {
        if self.last_successful_info_serial != Some(info_serial) {
            return;
        }
        self.last_successful_info_serial = None;
    }
}

impl RegisterTaskData {
    fn set_next_register(&mut self, next_register: Instant, change: &Notify) {
        if let Some(old) = self.next_register {
            if old <= next_register {
                return;
            }
        }
        self.next_register = Some(next_register);
        change.notify_one();
    }
    fn set_wait_time(&mut self, wait_time: Duration, change: &Notify) {
        self.set_next_register(self.prev_register + wait_time, change)
    }
}

#[derive(Default)]
pub struct RegisterBuilder {
    require_external_heartbeats: bool,
    register_url: Option<String>,
    user_agent: Option<String>,
    community_token: Option<String>,
}

impl RegisterBuilder {
    pub fn require_external_heartbeats(mut self) -> RegisterBuilder {
        assert!(!self.require_external_heartbeats);
        self.require_external_heartbeats = true;
        self
    }
    pub fn register_url(mut self, register_url: String) -> RegisterBuilder {
        assert!(self.register_url.is_none());
        self.register_url = Some(register_url);
        self
    }
    pub fn user_agent(mut self, user_agent: String) -> RegisterBuilder {
        assert!(self.user_agent.is_none());
        self.user_agent = Some(user_agent);
        self
    }
    pub fn community_token(mut self, community_token: String) -> RegisterBuilder {
        assert!(self.community_token.is_none());
        self.community_token = Some(community_token);
        self
    }
    pub fn build(self, port: u16, info: Arc<str>) -> Register {
        Register::new(self, port, info)
    }
}

pub struct Register {
    shared: Arc<RegisterShared>,
    tasks: [Arc<RegisterTaskShared>; 2],
}

async fn register_task(shared: Arc<RegisterShared>, task: Arc<RegisterTaskShared>) -> ! {
    let client = reqwest::Client::builder()
        .user_agent(&*shared.user_agent)
        .local_address(task.ip_version.bind_all())
        .build()
        .unwrap();

    let challenge_secret: Box<str> =
        format!("{}:{}", shared.challenge_secret, task.ip_version).into();

    loop {
        // send register
        {
            let (data, task_data) = {
                let data = shared.data.lock().unwrap();
                let mut task_data = task.data.lock().unwrap();
                let now = Instant::now();
                task_data.prev_register = now;
                task_data.next_register = shared.period.map(|p| now + p);
                (data.clone(), task_data.clone())
            };
            let result = send_register(
                &client,
                &shared.register_url,
                shared.port,
                data.info_serial,
                if data.last_successful_info_serial == Some(data.info_serial) {
                    None
                } else {
                    Some(&data.info)
                },
                &shared.secret,
                &challenge_secret,
                task_data.token.as_deref(),
                shared.community_token.as_deref(),
            )
            .await;
            {
                let prev_token = task_data.token;
                let prev_info_serial = data.info_serial;
                let mut data = shared.data.lock().unwrap();
                let mut task_data = task.data.lock().unwrap();
                if task_data.prev_result.as_ref().is_none_or(|r| *r != result) {
                    match &result {
                        RegisterResult::Success => info!("server registered"),
                        RegisterResult::NeedInfo => {}
                        RegisterResult::NeedChallenge => {}
                        RegisterResult::Error(err) => {
                            error!("error registering: {}", err.message);
                        }
                    }
                }
                match task_data.prev_result.insert(result) {
                    RegisterResult::Success => data.on_success(prev_info_serial),
                    RegisterResult::NeedInfo => data.on_need_info(prev_info_serial),
                    RegisterResult::NeedChallenge => {
                        if prev_token != task_data.token {
                            // Re-register immediately if we got a different
                            // token now.
                            continue;
                        }
                    }
                    RegisterResult::Error(_) => {}
                }
            }
        }

        // wait
        loop {
            let next_register = {
                let task_data = task.data.lock().unwrap();
                // consume notification
                {
                    pin! {
                        let notified = task.next_register_changed.notified();
                    }
                    Notified::enable(notified);
                }
                // with Rust 1.63 (MSRV), the following is still unstable:
                //Notified::enable(pin!(task.next_register_changed.notified()));
                task_data.next_register
            };
            match next_register {
                None => task.next_register_changed.notified().await,
                Some(next_register) => {
                    if Instant::now() >= next_register {
                        break;
                    }
                    select! {
                        biased;
                        () = sleep_until(next_register) => break,
                        () = task.next_register_changed.notified() => {},
                    }
                }
            }
        }
    }
}

impl Register {
    pub fn builder() -> RegisterBuilder {
        RegisterBuilder::default()
    }
    // Theoretically, the challenge token packet might arrive before we finish
    // constructing the `Register`. This is not really a problem, because we'll
    // try again. In practice, this is also unlikely to happen because the HTTP
    // call will need several roundtrips. Thus, we ignore the problem.
    fn new(
        RegisterBuilder {
            require_external_heartbeats,
            register_url,
            user_agent,
            community_token,
        }: RegisterBuilder,
        port: u16,
        info: Arc<str>,
    ) -> Register {
        let period = if require_external_heartbeats {
            None
        } else {
            Some(INTERVAL_HEARTBEAT)
        };

        let challenge_secret = Uuid::new_v4().to_string();
        let challenge_packet_prefix: Vec<u8> = [
            browse_protocol::CHALLENGE_6,
            challenge_secret.as_bytes(),
            b":",
        ]
        .into_iter()
        .flatten()
        .copied()
        .collect();

        let shared = Arc::new(RegisterShared {
            data: Mutex::new(RegisterData {
                info_serial: 0,
                info,
                last_successful_info_serial: None,
            }),
            port,
            register_url: register_url
                .unwrap_or_else(|| "https://master1.ddnet.org/ddnet/15/register".into())
                .into(),
            user_agent: user_agent
                .unwrap_or_else(|| {
                    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).into()
                })
                .into(),
            community_token: community_token.map(Into::into),
            challenge_packet_prefix: challenge_packet_prefix.into(),
            challenge_secret: challenge_secret.into(),
            secret: Uuid::new_v4().to_string().into(),
            period,
        });

        let now = Instant::now();
        let tasks = IP_VERSIONS.map(|ip_version| {
            let task = Arc::new(RegisterTaskShared {
                data: Mutex::new(RegisterTaskData {
                    token: None,
                    prev_result: None,
                    // dummy time, will be overwritten by register task
                    // immediately.
                    prev_register: now,
                    next_register: period.map(|p| now + p),
                }),
                ip_version,
                next_register_changed: Notify::new(),
            });
            let _ = tokio::spawn(register_task(shared.clone(), task.clone()));
            task
        });

        Register { shared, tasks }
    }
    pub fn on_new_info(&self, info: Arc<str>) {
        let mut data = self.shared.data.lock().unwrap();
        if data.info == info {
            return;
        }
        data.info_serial += 1;
        data.info = info;

        // Lock all the task data once.
        let mut task_data: Vec<_> = self.tasks.iter().map(|t| t.data.lock().unwrap()).collect();
        // Expedite the next register that is closest to execution, but don't
        // move it closer than `INTERVAL_INFO_CHANGE` from the previous
        // register.
        let minimum_next_register_idx = task_data
            .iter()
            .enumerate()
            .filter_map(|(i, d)| d.next_register.map(|n| (i, n)))
            .max_by_key(|&(_, n)| n)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let maximum_prev_register = task_data.iter().map(|d| d.prev_register).min().unwrap();
        task_data[minimum_next_register_idx].set_next_register(
            maximum_prev_register + INTERVAL_INFO_CHANGE,
            &self.tasks[minimum_next_register_idx].next_register_changed,
        );
    }
    pub fn on_udp_packet(&self, data: &[u8]) {
        if let Some(payload) = data.strip_prefix(&*self.shared.challenge_packet_prefix) {
            let mut unpacker = Unpacker::new(payload);
            match (
                unpacker
                    .read_string()
                    .ok()
                    .and_then(|s| str::from_utf8(s).ok())
                    .and_then(|s| IpVersion::from_str(s).ok()),
                unpacker
                    .read_string()
                    .ok()
                    .and_then(|s| str::from_utf8(s).ok()),
            ) {
                (Some(ip_version), Some(token)) => self.on_token(ip_version, token),
                _ => error!("invalid challenge packet from mastersrv"),
            }
        }
    }
    fn on_token(&self, ip_version: IpVersion, token: &str) {
        debug!("{ip_version} challenge_token={token:?}");
        let task = &self.tasks[ip_version.index()];
        let mut task_data = task.data.lock().unwrap();
        if Some(token) != task_data.token.as_deref() {
            task_data.token = Some(String::from(token).into_boxed_str().into());
            if let Some(RegisterResult::NeedChallenge) = task_data.prev_result {
                task_data.set_wait_time(INTERVAL_TOKEN_REQUIRED, &task.next_register_changed);
            }
        }
    }
    pub fn on_heartbeat(&self) {
        for task in &self.tasks {
            task.data
                .lock()
                .unwrap()
                .set_wait_time(INTERVAL_HEARTBEAT, &task.next_register_changed);
        }
    }
}
