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

pub const DEFAULT_CERTFILE: &str = "cert.pem";
pub const DEFAULT_KEYFILE: &str = "pkey.secret";

pub const DEFAULT_TLS_TOGGLE: bool = false;

pub const DEFAULT_KNOWN_MARKETS_URL: &str = "http://localhost:3030/book";
pub const DEFAULT_EXTERNAL_BOOK_URL: &str = "http://localhost:3030/book/";

#[derive(Clone, Debug)]
pub struct Arguments {
    pub listen_address: IpAddr,
    pub listen_port: u16,
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
    pub force_no_tls: bool,
    pub known_markets_url: String,
    pub external_book_url: String,
}

impl TryFrom<ArgMatches<'_>> for Arguments {
    type Error = &'static str;

    fn try_from(value: ArgMatches<'_>) -> Result<Self, Self::Error> {
        /* start with the hardcoded values as defaults */
        let mut listen_address: IpAddr = IpAddr::from_str(DEFAULT_IP).unwrap();
        let mut listen_port: u16 = DEFAULT_PORT.parse::<u16>().unwrap();
        let mut certificate_path: PathBuf = DEFAULT_CERTFILE.into();
        let mut private_key_path: PathBuf = DEFAULT_KEYFILE.into();
        let mut force_no_tls: bool = DEFAULT_TLS_TOGGLE;
        let mut known_markets_url: String = DEFAULT_KNOWN_MARKETS_URL.to_string();
        let mut external_book_url: String = DEFAULT_EXTERNAL_BOOK_URL.to_string();

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

        /* handle TLS toggle */
        if value.is_present("force-no-tls") {
            force_no_tls = true;
        } else {
            match env::var("OME_FORCE_NO_TLS") {
                Ok(t) => force_no_tls = t.parse::<bool>().unwrap(),
                Err(_e) => {}
            }
        }

        /* handle known markets url */
        if let Some(t) = value.value_of("known_markets_url") {
            known_markets_url = t.to_string();
        } else {
            match env::var("KNOWN_MARKETS_URL") {
                Ok(t) => known_markets_url = t,
                Err(_e) => return Err("Invalid known markets url")
            }
        }

        /* handle external book url */
        if let Some(t) = value.value_of("external_book_url") {
            external_book_url = t.to_string();
        } else {
            match env::var("EXTERNAL_BOOK_URL") {
                Ok(t) => external_book_url = t,
                Err(_e) => return Err("Invalid external book url")
            }
        }


        Ok(Self {
            listen_address,
            listen_port,
            certificate_path,
            private_key_path,
            force_no_tls,
            known_markets_url,
            external_book_url,
        })
    }
}
