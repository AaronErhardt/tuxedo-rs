const TEMP_HISTORY_LENGTH: usize = 5;

#[derive(Debug)]
pub struct TemperatureBuffer {
    // Stores the temperature history.
    temp_history: Box<[u8; TEMP_HISTORY_LENGTH]>,
    position: usize,
}

impl TemperatureBuffer {
    pub(super) fn new(temp: u8) -> Self {
        Self {
            temp_history: Box::new([temp; TEMP_HISTORY_LENGTH]),
            position: 0,
        }
    }

    pub(super) fn update(&mut self, temp: u8) {
        self.position = (self.position + 1) % TEMP_HISTORY_LENGTH;
        self.temp_history[self.position] = temp;
    }

    /// Returns the difference between the latest temperature value
    /// and the smallest value in the history.
    /// If the values are rising fast, we should update the fanspeed
    /// more often.
    /// If the values are not changing by a lot, we can update the
    /// fanspeed less often to reduce CPU usage.
    pub(super) fn diff_to_min_in_history(&self) -> u8 {
        let current = self.temp_history[self.position];
        let min = self.temp_history.iter().min().unwrap();
        current - min
    }

    pub(crate) fn get_latest(&self) -> u8 {
        self.temp_history[self.position]
    }
}
