use std::time::Duration;

use tailor_api::keyboard::{Color, ColorPoint, ColorProfile, ColorTransition};
use tokio::sync::{broadcast, mpsc};
use tuxedo_sysfs::keyboard::KeyboardController;

use super::{dbus, NeverFuture};

pub struct KeyboardRuntime {
    interface: KeyboardController,
    colors: ColorProfile,
    suspend_receiver: broadcast::Receiver<bool>,
    color_receiver: mpsc::Receiver<ColorProfile>,
}

impl KeyboardRuntime {
    pub async fn new(
        suspend_receiver: broadcast::Receiver<bool>,
        color_receiver: mpsc::Receiver<ColorProfile>,
    ) -> Self {
        // Load color profile if available
        let colors = if let Ok(profile_name) = dbus::read_active_profile_file().await {
            if let Ok(colors) = dbus::load_keyboard_colors(&profile_name).await {
                colors
            } else {
                ColorProfile::default()
            }
        } else {
            ColorProfile::default()
        };

        Self {
            interface: KeyboardController::new().await.unwrap(),
            colors,
            suspend_receiver,
            color_receiver,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let KeyboardRuntime {
                interface,
                colors,
                suspend_receiver,
                color_receiver,
            } = self;

            tokio::select! {
                new_colors = color_receiver.recv() => {
                    *colors = new_colors.unwrap();
                }
                _ = Self::update_colors(colors, interface, suspend_receiver) => {

                }
            }
        }
    }

    pub async fn update_colors(
        colors: &ColorProfile,
        interface: &KeyboardController,
        suspend_receiver: &mut broadcast::Receiver<bool>,
    ) {
        match colors {
            ColorProfile::None => NeverFuture().await,
            ColorProfile::Single(color) => {
                interface.set_color_all(color).await.unwrap();
                NeverFuture().await;
            }
            ColorProfile::Multiple(colors) => {
                let color_steps = calculate_color_animation_steps(colors);
                run_color_animation(interface, suspend_receiver, &color_steps).await;
            }
        }
    }
}

async fn run_color_animation(
    interface: &KeyboardController,
    suspend_receiver: &mut broadcast::Receiver<bool>,
    color_steps: &[(Color, u32)],
) {
    for step in color_steps.iter().cycle() {
        interface.set_color_left(&step.0).await.unwrap();

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(step.1 as u64)) => {}
            msg = suspend_receiver.recv() => {
                // Suspended!
                if msg.unwrap() {
                    // Wait until wake up (suspend msg == false).
                    while suspend_receiver.recv().await.unwrap() {}
                }
            }
        }
    }
}

fn calculate_color_animation_steps(colors: &[ColorPoint]) -> Vec<(Color, u32)> {
    let mut color_steps = Vec::new();
    let mut prev_color = colors.last().unwrap().color.clone();

    for color_point in colors {
        let ColorPoint {
            color,
            transition,
            transition_time,
        } = color_point.clone();

        match transition {
            ColorTransition::None => {
                color_steps.push((color, transition_time));
            }
            ColorTransition::Linear => {
                linear_color_transition(&mut color_steps, color, &prev_color, transition_time);
            }
        }

        prev_color = color_point.color.clone();
    }
    color_steps
}

fn linear_color_transition(
    color_steps: &mut Vec<(Color, u32)>,
    color: Color,
    prev_color: &Color,
    transition_time: u32,
) {
    // Max step size 80 ms (12.5 fps).
    // More would be rather CPU intensive for a background
    // job (> 0.5%).
    let steps = transition_time / 80;

    if steps == 0 {
        color_steps.push((color, transition_time));
    } else {
        let r_diff = color.r as f64 - prev_color.r as f64;
        let g_diff = color.g as f64 - prev_color.g as f64;
        let b_diff = color.b as f64 - prev_color.b as f64;

        let decent_steps = decent_linear_steps(transition_time, &[r_diff, g_diff, b_diff]);
        // Use lower step size if possible
        let steps = steps.min(decent_steps);

        let step_time = transition_time / steps;

        for idx in 0..steps {
            let percent = idx as f64 / steps as f64;

            let r = f64_to_u8(prev_color.r as f64 + r_diff * percent);
            let g = f64_to_u8(prev_color.g as f64 + g_diff * percent);
            let b = f64_to_u8(prev_color.b as f64 + b_diff * percent);

            let color = Color { r, g, b };
            color_steps.push((color, step_time));
        }
    }
}

fn f64_to_u8(float: f64) -> u8 {
    float.clamp(0.0, 255.0).round() as u8
}

fn decent_linear_steps(transition_time: u32, diffs: &[f64]) -> u32 {
    let diff_square_sum: f64 = diffs.iter().map(|diff| diff.powi(2)).sum();
    let diff_rms = diff_square_sum.sqrt();

    if diff_rms <= f64::EPSILON {
        1
    } else {
        // A delta of 15 as rgb value per second should be barely
        // visible to the human eye.
        let imperceivable_steps = diff_rms / 15.0;

        // As time becomes larger, make smaller steps because they
        // might become identifiable as individual steps again.
        let time_factor = (transition_time as f64 / 1000.0).sqrt().clamp(0.4, 5.0);

        ((imperceivable_steps * time_factor).round() as u32).max(1)
    }
}

#[cfg(test)]
mod test {
    use crate::keyboard::runtime::decent_linear_steps;

    #[test]
    fn decent_linear_step() {
        let decent_steps = decent_linear_steps(1000, &[0.0]);
        assert_eq!(decent_steps, 1);

        let decent_steps = decent_linear_steps(1000, &[150.0]);
        assert_eq!(decent_steps, 10);

        let decent_steps = decent_linear_steps(3000, &[150.0]);
        assert_eq!(decent_steps, 17);

        let decent_steps = decent_linear_steps(1000, &[75.0]);
        assert_eq!(decent_steps, 5);

        let decent_steps = decent_linear_steps(100, &[75.0]);
        assert_eq!(decent_steps, 2);
    }
}
