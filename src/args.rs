use std::convert::TryFrom;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

use clap::ArgMatches;

/// The default IP address that the OME will listen on
pub const DEFAULT_IP: &str = "0.0.0.0";

/// The default TCP port number that the OME will listen on
pub const DEFAULT_PORT: &str = "8989";

/// The default file path for reading and writing state dumps
pub const DEFAULT_DUMPFILE: &str = ".omedump.json";

pub const DEFAULT_EXECUTIONER: &str = "http://localhost:3000";

pub const DEFAULT_CERTFILE: &str = "cert.pem";
pub const DEFAULT_KEYFILE: &str = "pkey.secret";

#[derive(Clone, Debug)]
pub struct Arguments {
    pub listen_address: IpAddr,
    pub listen_port: u16,
    pub executioner_address: String,
    pub dumpfile_path: PathBuf,
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
}

impl TryFrom<ArgMatches<'_>> for Arguments {
    type Error = &'static str;

    fn try_from(value: ArgMatches<'_>) -> Result<Self, Self::Error> {
        /* start with the hardcoded values as defaults */
        let mut listen_address: IpAddr = IpAddr::from_str(DEFAULT_IP).unwrap();
        let mut listen_port: u16 = DEFAULT_PORT.parse::<u16>().unwrap();
        let mut executioner_address: String = DEFAULT_EXECUTIONER.to_string();
        let mut dumpfile_path: PathBuf = DEFAULT_DUMPFILE.into();
        let mut certificate_path: PathBuf = DEFAULT_CERTFILE.into();
        let mut private_key_path: PathBuf = DEFAULT_KEYFILE.into();

        /* handle listening address */
        if let Some(t) = value.value_of("listen") {
            listen_address = match IpAddr::from_str(t) {
                Ok(p) => p,
                Err(_e) => return Err("Invalid listening address"),
            };
        } else {
            match env::var("OME_LISTEN_ADDRESS") {
                Ok(t) => {
                    listen_address = match IpAddr::from_str(&t) {
                        Ok(p) => p,
                        Err(_err) => return Err("Invalid listening address"),
                    }
                }
                Err(_e) => {}
            };
        }

        /* handle listening port */
        if let Some(t) = value.value_of("port") {
            listen_port = match t.parse::<u16>() {
                Ok(p) => p,
                Err(_e) => return Err("Invalid listening port"),
            };
        } else {
            match env::var("OME_LISTEN_PORT") {
                Ok(t) => match t.parse::<u16>() {
                    Ok(p) => listen_port = p,
                    Err(_err) => return Err("Invalid listening port"),
                },
                Err(_e) => {}
            }
        }

        /* handle executioner address */
        if let Some(t) = value.value_of("executioner_address") {
            executioner_address = t.to_string();
        } else {
            match env::var("OME_EXECUTIONER_ADDRESS") {
                Ok(t) => executioner_address = t,
                Err(_e) => {}
            }
        }

        /* handle dumpfile path */
        if let Some(t) = value.value_of("dumpfile_path") {
            dumpfile_path = t.into();
        };

        /* handle TLS certificate path */
        if let Some(t) = value.value_of("certificate_path") {
            certificate_path = t.into();
        } else {
            match env::var("OME_CERTIFICATE_PATH") {
                Ok(t) => certificate_path = t.into(),
                Err(_e) => {}
            }
        }

        /* handle TLS private key path */
        if let Some(t) = value.value_of("private_key_path") {
            private_key_path = t.into();
        } else {
            match env::var("OME_PRIVATE_KEY_PATH") {
                Ok(t) => certificate_path = t.into(),
                Err(_e) => {}
            }
        }

        Ok(Self {
            listen_address,
            listen_port,
            executioner_address,
            dumpfile_path,
            certificate_path,
            private_key_path,
        })
    }
}
