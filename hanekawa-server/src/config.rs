use hanekawa_common::Config;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};

pub fn load_config() -> Config {
    const CONFIG_FILE: &'static str = "hanekawa.toml";
    const ENV_PREFIX: &'static str = "HKW_";

    let cfg = Figment::new()
        .merge(Toml::file(CONFIG_FILE))
        .merge(Env::prefixed(ENV_PREFIX))
        .extract();

    let cfg: Config = match cfg {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("config error: {}", e);
            std::process::exit(1);
        }
    };

    cfg
}
