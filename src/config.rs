use crate::{MyError, MyResult};
use std::{env, sync::OnceLock};

#[allow(non_snake_case)]
pub struct MyConfig{
    pub WEB_FOLDER: String, 
}

fn get_env(x: &str) -> MyResult<String>{
    env::var(x).map_err(|e| MyError::ConfigEnvMissing(e))
}

impl MyConfig{
    fn load_env() -> MyResult<Self>{
        Ok(Self{
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?, 
        })
    }
}

pub fn config() -> &'static MyConfig{
    static CONFIG_INIT: OnceLock<MyConfig> = OnceLock::new();
    CONFIG_INIT.get_or_init(|| {
        MyConfig::load_env().unwrap_or_else(|x| {
            panic!("fatal error: {:?}", x);
        })
    })
}




