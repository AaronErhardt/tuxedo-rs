use crate::config::{FanData, FanEvolution, TempProfile};
use crate::Instance;

use std::collections::VecDeque;
use std::{thread, time::Duration};

// struct to calculate the average temperature
// at different intervals
pub struct TempLoad {
    // at current loop
    pub l1: u8,
    // at 5 loops
    pub l5: u8,
    // at 15 loops
    pub l15: u8,
    // at 30 loops
    pub l30: u8,
    // on tho whole history
    pub lall: u8,
}

// main logic
pub fn fan_control_loop(i: &mut Instance) {
    // add the current temperature to history
    i.add_temp();

    let temp_load = calc_temp_load(&i.temp_history);

    println!(
        "Average temperature [{} {} {} {} {}]",
        temp_load.l1, temp_load.l5, temp_load.l15, temp_load.l30, temp_load.lall
    );

    let target_fan_speed =
        calc_temp_segment(i.temp_history.back().unwrap(), &i.config.temp_profile);

    if target_fan_speed == i.fan_data.fan_speed {
        i.fan_data.last_update_loop = 0;
    } else {
        let new_fan_speed = calc_new_speed(target_fan_speed, &i.fan_data, &temp_load);
        i.fan_data.last_update_loop += 1;

        if let Some(new_speed) = new_fan_speed {
            if new_speed != i.fan_data.fan_speed {
                i.set_speed(new_speed);
            }
        }
    }

    thread::sleep(Duration::from_millis(i.config.check_delay));
}

fn calc_temp_load(temp_history: &VecDeque<u8>) -> TempLoad {
    TempLoad {
        l1: s_temp_load(temp_history, Some(1)),
        l5: s_temp_load(temp_history, Some(5)),
        l15: s_temp_load(temp_history, Some(15)),
        l30: s_temp_load(temp_history, Some(30)),
        lall: s_temp_load(temp_history, None),
    }
}

// subfunction for calc_temp_load
fn s_temp_load(temp_history: &VecDeque<u8>, looptimes_o: Option<u32>) -> u8 {
    if let Some(looptimes) = looptimes_o {
        // calculate the average on a specific amount of loops
        if temp_history.len() >= (looptimes as u32).try_into().unwrap() {
            let start_pos = temp_history.len() - looptimes as usize;
            return s_mean(temp_history.range(start_pos..), looptimes);
        }
    }
    // or calculate the average on the whole array
    s_mean(temp_history.range(..), temp_history.len() as u32)
}

fn s_mean(temp_history: std::collections::vec_deque::Iter<'_, u8>, length: u32) -> u8 {
    let mut history_sum: u32 = 0;
    for entry in temp_history {
        history_sum += *entry as u32;
    }
    (history_sum / length) as u8
}

// use the temp profile in configuration to find
// between which entry the current temperature is situated
// returns the corresponding fan speed.
fn calc_temp_segment(current_temp: &u8, temp_profile: &[TempProfile]) -> u8 {
    let mut temp_iter = temp_profile.iter().peekable();
    while let Some(entry) = temp_iter.next() {
        if current_temp < &entry.temp {
            // current temp is lower than the first element
            // returns the first element
            return temp_profile.first().unwrap().fan;
        }
        if let Some(next_entry) = temp_iter.peek() {
            if current_temp >= &entry.temp && current_temp < &next_entry.temp {
                // right in the middle − calculate the target average
                let relative_maxtemp = next_entry.temp - entry.temp;
                let relative_currtemp = current_temp - entry.temp;
                let relative_pct: f32 = relative_currtemp as f32 / relative_maxtemp as f32;

                // apply the percentage on target fan
                let fan_adjust = (next_entry.fan - entry.fan) as f32 * relative_pct;
                return entry.fan + fan_adjust as u8;
            }
        } else {
            // current temp is higher than the last element
            // returns the last element
            return temp_profile.last().unwrap().fan;
        }
    }
    // should not be reached
    temp_profile.last().unwrap().fan
}

// The Algorithm…
// I suck at math. Did you notice?
// Here are the rules, it’s magic number time.
//
// 1. Avoid to change the speed too frequently, esp. when not needed
// ------------------------------------------------------------------
// 1a. If speed has changed since <= 3 iterations, don’t change it.
// 1b. If speed has changed since less than 15 iterations
//      AND the difference between the current and targeted fan speed
//      is <= 5%, do not change anything.
// 1c. If speed has changed since less than 30 iterations
//      AND the difference between the current and targeted fan speed
//      is <= 2%, do not change anything.
//
// 2. Attempt to stabilize fan speed and prevent it to oscillate
//      between low/high fan speed constantly.
// ------------------------------------------------------------------
// 2a. If target_fan_speed is below fan_speed AND last_fan_evolution
//      is Increasing, evolution_coeff is 0.5.
// 2b. Else, if target_fan_speed is above fan_speed AND
//      last_fan_evolution is Decreasing, evolution_coeff is 0.5.
// 2c. Else, evolution_coeff is 1.0.
//
// 3. Smoothen the curve (make it increase/decrease slowly).
// ------------------------------------------------------------------
// 3. A weighting system is used to calculate the new fan speed.
//      The absolute difference (d) is calculated between the current
//      temperature and the previous temperatures.
//      - d(l1, l5) = d5
//      - d(l1, l15) = d15
//      - d(l1, l30) = d30
//      - d(l1, lall) = dall
//      We calculate the the fraction of each result from dall:
//      - (dall - d5) / dall) = f5
//      - (dall - d15) / dall) = f15
//      - (dall - d30) / dall) = f30
//      (if dall == 0, the step is skipped and variation_coeff = 0.5)
//      A weighted mean is calculated from the results.
//      - f5 is weighted 0.75
//      - f15 is weighted 1.5.
//      - f30 is weighted 1.25.
//      All those numbers are capped between 0 and 1 before
//      the weighted mean is calculated.
//      The result is variation_coeff.
//
// 4. The coefficients are applied to the target fan speed.
fn calc_new_speed(target_fan_speed: u8, fan_data: &FanData, temp_load: &TempLoad) -> Option<u8> {
    // STEP 1
    if fan_data.last_update_loop <= 3
        || fan_data.last_update_loop <= 15 && target_fan_speed.abs_diff(fan_data.fan_speed) <= 5
        || fan_data.last_update_loop <= 30 && target_fan_speed.abs_diff(fan_data.fan_speed) <= 2
    {
        return None;
    }

    // STEP 2
    let evolution_coeff = if target_fan_speed < fan_data.fan_speed
        && fan_data.last_fan_evolution == FanEvolution::Increasing
        || target_fan_speed > fan_data.fan_speed
            && fan_data.last_fan_evolution == FanEvolution::Decreasing
    {
        0.5
    } else {
        1.0
    };

    // STEP 3
    let d5 = temp_load.l1.abs_diff(temp_load.l5);
    let d15 = temp_load.l1.abs_diff(temp_load.l15);
    let d30 = temp_load.l1.abs_diff(temp_load.l30);
    let dall = temp_load.l1.abs_diff(temp_load.lall);

    let variation_coeff = if dall != 0 {
        let f5 = cap(dall.abs_diff(d5) as f32 / dall as f32);
        let f15 = cap(dall.abs_diff(d15) as f32 / dall as f32);
        let f30 = cap(dall.abs_diff(d30) as f32 / dall as f32);

        (f5 * 0.75 + f15 * 1.5 + f30 * 1.25) / 3.5
    } else {
        0.5
    };

    // STEP 4
    if target_fan_speed != 0 {
        let target_var = target_fan_speed as f32 - fan_data.fan_speed as f32;
        let adjusted_target = target_var * evolution_coeff * variation_coeff;
        let final_target = (fan_data.fan_speed as f32 + adjusted_target) as u8;
        Some(final_target)
    } else {
        Some(0)
    }
}

fn cap(f: f32) -> f32 {
    f.clamp(0.0, 1.0)
}
