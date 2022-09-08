use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::{cmp::Ordering, collections::VecDeque};
use tuxedo_ioctl::high_level::{Fan, IoInterface};

const CONFIG_FILE: &str = "./config.toml";
const CONFIG_VERSION: u8 = 1;
const MINIMAL_HISTORY_STORE: u32 = 30;
const MINIMAL_HISTORY_ENTRIES: u32 = 30;
const MINIMAL_CHECK_DELAY: u64 = 100;
const MAXIMAL_CHECK_DELAY: u64 = 10000;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub history_store: u32,
    pub check_delay: u64,
    pub temp_profile: Vec<TempProfile>,
    pub version: u8,
}

#[derive(Debug)]
pub struct Instance {
    // stores the last <history_max_entries> minutes of temperature.
    pub temp_history: VecDeque<u8>,
    // the calculated number of max history entries
    pub history_max_entries: u32,
    // see struct below
    pub fan_data: FanData,
    // device i/o
    pub io: IoInterface,
    // the configuration file
    pub config: Config,
}

#[derive(Debug)]
pub struct FanData {
    // number of loop iterations since temperature has been changed
    pub last_update_loop: u32,
    // set if last update was increasing or decreasing fan speed
    pub last_fan_evolution: FanEvolution,
    // percentage of the current fan speed
    pub fan_speed: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FanEvolution {
    Increasing,
    Decreasing,
}

#[derive(Deserialize, Debug)]
pub struct TempProfile {
    pub temp: u8,
    pub fan: u8,
}

impl Config {
    pub fn init() -> Self {
        let mut conffile = File::open(CONFIG_FILE).expect("Config file not found.");
        let mut confstr = String::new();
        conffile
            .read_to_string(&mut confstr)
            .expect("Couldn't read config to string");
        toml::from_str(&confstr).unwrap()
    }
    pub fn check(&self) {
        // check config version
        if self.version != CONFIG_VERSION {
            eprintln!("Your configuration file is obsolete ({}). Please edit it and update its version to {}.", self.version, CONFIG_VERSION);
            panic!();
        }

        // check if history_max_entries will be high enough
        if (self.history_store * 1000 / self.check_delay as u32) < MINIMAL_HISTORY_ENTRIES {
            eprintln!("Your history store is defined to {}, it is too low compared to your check_delay {}. Please raise history_store or diminish check_delay.", self.history_store, self.check_delay);
            panic!();
        }

        // check if history_store is high enough
        if self.history_store < MINIMAL_HISTORY_STORE {
            eprintln!(
                "Your history store is defined to {}. You must set it at least above {}.",
                self.history_store, MINIMAL_HISTORY_STORE
            );
            panic!();
        }

        if self.history_store < MINIMAL_HISTORY_STORE * 2 {
            // issue a non-blocking warning
            eprintln!("Your history store is defined to {}. It’s quite low, you may want to set it above {}.", self.history_store, MINIMAL_HISTORY_STORE * 2);
        }

        // check if check_delay seems reasonable
        if self.check_delay < MINIMAL_CHECK_DELAY {
            eprintln!("Your check delay ({}) is too low! Checking the fan temperature this often is pointless. Raise it at least to {}.", self.check_delay, MINIMAL_CHECK_DELAY);
            panic!();
        }

        if self.check_delay > MAXIMAL_CHECK_DELAY {
            eprintln!("Your check delay ({}) is too high! You need to check the fan temperature more often. Reduce it at least to {}.", self.check_delay, MAXIMAL_CHECK_DELAY);
            panic!();
        }

        // check if the defined temperature profile seems correct

        // the fan speed AND temperature must always be defined between 1-100
        let mut temp_minimal = 0;
        let mut fan_minimal = 0;
        for temp_entry in &self.temp_profile {
            if temp_entry.temp > 100 {
                eprintln!("One of the temperatures in your profile is higher than 100 degrees Celsius. Please fix it.");
                panic!();
            }

            if temp_entry.fan > 100 {
                eprintln!("One of the fan speeds in your profile is higher than 100 percents. Please fix it.");
                panic!();
            }

            match temp_minimal.cmp(&temp_entry.temp) {
                Ordering::Less => {
                    temp_minimal = temp_entry.temp;
                }
                Ordering::Equal | Ordering::Greater => {
                    eprintln!("Your temperature profile is not consistent: temperature must increase gradually at each new entry, which is not the case for at least one of your entries. Please fix it.");
                    panic!();
                }
            }
            match fan_minimal.cmp(&temp_entry.fan) {
                Ordering::Less => {
                    fan_minimal = temp_entry.fan;
                }
                Ordering::Equal | Ordering::Greater => {
                    eprintln!("Your temperature profile is not consistent: the fan speed is reduced while temperature is going up. This is probably not intended, please fix it.");
                    panic!();
                }
            }
        }
    }
}

impl Instance {
    // initialize global instance at startup
    pub fn init() -> Instance {
        let config = Config::init();
        let io = IoInterface::new().unwrap();
        let fan_speed = io.get_fan_speed_percent(Fan::Fan1).unwrap();

        let fan_data = FanData {
            last_update_loop: 0,
            last_fan_evolution: FanEvolution::Decreasing,
            fan_speed,
        };

        Instance {
            temp_history: VecDeque::new(),
            history_max_entries: config.history_store * 1000 / config.check_delay as u32,
            fan_data,
            io,
            config,
        }
    }

    // check for blatant configuration issues

    // adds entries to history store respecting configured history rules
    pub fn add_temp(&mut self) {
        let temp = self.io.get_fan_temperature(Fan::Fan1).unwrap();

        if self.temp_history.len() >= self.history_max_entries as usize {
            self.temp_history.pop_front();
        }
        self.temp_history.push_back(temp);
    }

    pub fn set_speed(&mut self, new_speed: u8) {
        if self.fan_data.fan_speed < new_speed {
            self.fan_data.last_fan_evolution = FanEvolution::Increasing;
        } else {
            self.fan_data.last_fan_evolution = FanEvolution::Decreasing;
        }
        self.fan_data.last_update_loop = 0;
        self.fan_data.fan_speed = new_speed;

        if new_speed <= 100 {
            println!("Fan speed update: {}", new_speed);
            self.io.set_fan_speed_percent(Fan::Fan1, new_speed).unwrap();
        }
    }
}
