use serde::{Deserialize, Serialize};

pub trait Driver {
    fn to_link(self) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub datasource: Datasource,
    pub redis: Redis,
    pub cleanup: Option<Cleanup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cleanup {
    /// 清理间隔（秒），默认3600秒（1小时）
    pub interval_secs: Option<u64>,
    /// 批量处理大小，默认1000
    pub batch_size: Option<usize>,
    /// 是否启用分布式锁，默认false
    pub enable_distributed_lock: Option<bool>,
    /// 分布式锁超时时间（秒），默认300秒（5分钟）
    pub lock_timeout_secs: Option<u64>,
    /// 分布式锁key
    pub lock_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Datasource {
    pub host: Option<String>,
    pub port: Option<usize>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub max_pool_size: Option<usize>,
    pub min_pool_size: Option<usize>,
    pub idle_timeout: Option<usize>,
    pub db_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Redis {
    pub host: Option<String>,
    pub port: Option<usize>,
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub max_size: Option<usize>,
    pub database: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            datasource: Datasource::default(),
            redis: Redis::default(),
            cleanup: Some(Cleanup::default()),
        }
    }
}

impl Default for Cleanup {
    fn default() -> Self {
        Self {
            interval_secs: Some(3600),
            batch_size: Some(1000),
            enable_distributed_lock: Some(false),
            lock_timeout_secs: Some(300),
            lock_key: Some("cleanup:lock".to_string()),
        }
    }
}

impl Default for Datasource {
    fn default() -> Self {
        Self {
            host: Some("localhost".to_string()),
            port: Some(5432),
            user: Some("postgres".to_string()),
            password: None,
            max_pool_size: None,
            min_pool_size: None,
            idle_timeout: None,
            db_name: None,
        }
    }
}

impl Default for Redis {
    fn default() -> Self {
        Self {
            host: Some("localhost".to_string()),
            port: Some(6379),
            user_name: None,
            password: None,
            max_size: None,
            database: Some(0),
        }
    }
}

impl Driver for Datasource {
    fn to_link(self) -> String {
        let host = self.host.unwrap_or("localhost".to_string());
        let port = self.port.unwrap_or(5432);
        let user = self.user.unwrap_or("postgres".to_string());
        let password = self.password.unwrap_or("".to_string());
        let db_name = self.db_name.unwrap_or("".to_string());
        format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, db_name
        )
    }
}

impl Driver for Redis {
    fn to_link(self) -> String {
        let host = self.host.unwrap_or("localhost".to_string());
        let port = self.port.unwrap_or(6379);
        let password = self.password;
        let user_name = self.user_name;

        let mut uri = "redis://".to_string();
        match user_name {
            Some(account) => {
                uri.push_str(&account);
            }
            _ => {}
        }
        match password {
            Some(pwd) => {
                uri.push_str(":");
                uri.push_str(&pwd);
                uri.push_str("@");
            }
            _ => {}
        }
        uri.push_str(&host);
        uri.push_str(":");
        uri.push_str(&port.to_string());
        uri.to_string()
    }
}
