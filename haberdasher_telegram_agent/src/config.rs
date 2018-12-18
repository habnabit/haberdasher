use clacks_mtproto::mtproto;
use std::borrow::Cow;

type Result<T> = std::result::Result<T, failure::Error>;

pub trait Kind {
    type Stored: serde::Serialize + for<'a> serde::Deserialize<'a>;
    fn key(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Entry<K> {
    kind: K,
    key: String,
}

impl<K: Kind> Entry<K> {
    pub fn new(kind: K) -> Self {
        let key = kind.key();
        Entry { kind, key }
    }

    pub fn get(&self, tree: &sled::Tree) -> Result<Option<K::Stored>> {
        let value = match tree.get(self.key.as_bytes())? {
            Some(ref b) if b.is_empty() => None,
            None => None,
            Some(b) => Some(serde_json::from_slice(&b)?),
        };
        Ok(value)
    }

    pub fn set(&self, tree: &sled::Tree, value: &K::Stored) -> Result<()> {
        let serialized = serde_json::to_vec(value)?;
        tree.set(self.key.as_bytes(), serialized)?;
        Ok(())
    }
}

impl<K> std::ops::Deref for Entry<K> {
    type Target = K;
    fn deref(&self) -> &K { &self.kind }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataV1 {
    pub native_dc: u32,
}

#[derive(Debug, Clone)]
pub struct UserData<'a> {
    pub phone_number: Cow<'a, str>,
}

impl<'a> UserData<'a> {
    pub fn as_auth_key(&self) -> UserAuthKey<'a> {
        UserAuthKey { phone_number: self.phone_number.clone() }
    }
}

impl<'a> Kind for UserData<'a> {
    type Stored = UserDataV1;
    fn key(&self) -> String { format!("user/+{}/data", self.phone_number) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthKeyV1 {
    pub auth_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UserAuthKey<'a> {
    pub phone_number: Cow<'a, str>,
}

impl<'a> Kind for UserAuthKey<'a> {
    type Stored = UserAuthKeyV1;
    fn key(&self) -> String { format!("user/+{}/auth-key", self.phone_number) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramServersV1 {
    pub servers: Vec<mtproto::dc_option::DcOption>,
}

impl TelegramServersV1 {
    pub fn iter_addresses<'this>(&'this self) -> impl Iterator<Item = std::net::SocketAddr> + 'this {
        self.servers.iter()
            .filter(|dc| !dc.cdn && !dc.media_only)
            .map(|dc| -> Result<std::net::SocketAddr> {
                use std::net::*;
                let ip: IpAddr = if dc.ipv6 {
                    dc.ip_address.parse::<Ipv6Addr>()?.into()
                } else {
                    dc.ip_address.parse::<Ipv4Addr>()?.into()
                };
                Ok(SocketAddr::new(ip, dc.port as u16))
            })
            .filter_map(|r| r
                .map_err(|e| println!("XXX parse error? {:?}", e))
                .ok())
    }
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TelegramDatacenter {
    pub number: u32,
    pub test_mode: bool,
}

impl Kind for TelegramDatacenter {
    type Stored = TelegramServersV1;
    fn key(&self) -> String { format!("datacenter/{}/{}", self.number, if self.test_mode { "test" } else { "prod" }) }
}

impl Entry<TelegramDatacenter> {
    pub fn get_or_default(&self, tree: &sled::Tree) -> Result<TelegramServersV1> {
        self.get(tree).map(|o| o.unwrap_or_else(|| self.hardcoded_servers()))
    }

    fn hardcoded_servers(&self) -> TelegramServersV1 {
        if self.test_mode {
            TelegramServersV1 { servers: vec![
                mtproto::dc_option::DcOption {
                    cdn: false,
                    id: 2,
                    ip_address: "149.154.167.40".to_owned(),
                    ipv6: false,
                    media_only: false,
                    port: 80,
                    static_: false,
                    tcpo_only: false,
                }
            ] }
        } else {
            TelegramServersV1 { servers: vec![
                mtproto::dc_option::DcOption {
                    cdn: false,
                    id: 2,
                    ip_address: "149.154.167.50".to_owned(),
                    ipv6: false,
                    media_only: false,
                    port: 443,
                    static_: false,
                    tcpo_only: false,
                }
            ] }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HaberdasherConfig {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    pub api_id: i32,
    pub api_hash: String,
    pub users: Vec<String>,
}

impl TelegramConfig {
    pub fn as_app_id(&self) -> clacks_transport::AppId {
        clacks_transport::AppId {
            api_id: self.api_id,
            api_hash: self.api_hash.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub db: DatabaseConfig,
    pub haberdasher: HaberdasherConfig,
    pub telegram: TelegramConfig,
}

pub fn load_config_file(path: &std::path::Path) -> Result<AgentConfig> {
    use std::io::Read;
    let mut infile = std::fs::File::open(path)?;
    let mut content = String::new();
    infile.read_to_string(&mut content)?;
    Ok(toml::from_str(&content)?)
}
