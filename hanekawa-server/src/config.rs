use hanekawa_common::Config;

pub fn load_config() -> Config {
    const CONFIG_FILE: &'static str = "hanekawa.toml";
    const ENV_PREFIX: &'static str = "HKW";

    let cfg = config::Config::builder()
        .add_source(config::File::with_name(&CONFIG_FILE).required(false))
        .add_source(config::Environment::with_prefix(&ENV_PREFIX))
        .build()
        .unwrap();

    fn get_or_panic<T>(key: &str, value: Result<T, config::ConfigError>) -> T {
        use config::ConfigError;

        match value {
            Ok(t) => t,
            Err(ConfigError::Type {
                unexpected,
                expected,
                ..
            }) => {
                panic!("config error: unexpected value for {key}: expected {expected} got {unexpected}");
            }
            Err(ConfigError::NotFound(_)) => {
                panic!("config error: missing value for {key}: supply it in {CONFIG_FILE} or set the {ENV_PREFIX}_{} environment variable", key.to_uppercase());
            }
            _ => {
                panic!("config error: error for {key}");
            }
        }
    }

    let get_string = |key| get_or_panic(key, cfg.get_string(key));

    let get_u32 = |key| {
        let num = get_or_panic(key, cfg.get_int(key));
        let num: u32 = num
            .try_into()
            .expect("config error: value for {key} out of range");
        num
    };

    let get_bool = |key: &str| {
        let b = get_or_panic(key, cfg.get_bool(key));
        b
    };

    let bind_ip = get_string("bind_ip").parse().expect("invalid bind_ip");

    Config {
        database_url: get_string("database_url"),
        bind_ip,
        http_bind_port: get_u32("http_bind_port")
            .try_into()
            .expect("invalid http_bind_port"),
        udp_bind_port: get_u32("udp_bind_port")
            .try_into()
            .expect("invalid udp_bind_port"),
        peer_announce_interval: get_u32("peer_announce_interval"),
        peer_activity_timeout: get_u32("peer_activity_timeout"),
        only_allowed_info_hashes: get_bool("only_allowed_info_hashes"),
        enable_admin_api: get_bool("enable_admin_api"),
    }
}
