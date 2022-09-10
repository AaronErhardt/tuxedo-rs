use std::{future::pending, time::Duration};

use tailor_api::keyboard::{Color, ColorPoint, ColorProfile, ColorTransition};
use tokio::sync::{broadcast, mpsc};
use tuxedo_sysfs::keyboard::KeyboardController;

use crate::suspend::process_suspend;

pub struct KeyboardRuntime {
    io: KeyboardController,
    profile: ColorProfile,
    suspend_receiver: broadcast::Receiver<bool>,
}

impl KeyboardRuntime {
    pub async fn new(profile: ColorProfile, suspend_receiver: broadcast::Receiver<bool>) -> Self {
        Self {
            io: KeyboardController::new().await.unwrap(),
            profile,
            suspend_receiver,
        }
    }

    pub async fn run(
        mut self,
        mut keyboard_receiver: mpsc::Receiver<ColorProfile>,
        mut color_receiver: mpsc::Receiver<Color>,
    ) {
        loop {
            tokio::select! {
                new_colors = keyboard_receiver.recv() => {
                    if let Some(colors) = new_colors {
                        self.profile = colors;
                    }
                }
                override_color = color_receiver.recv() => {
                    if let Some(mut color) = override_color {
                        loop {
                            if let Err(err) = self.io.set_color_left(&color).await {
                                tracing::error!("Failed to update keyboard color: `{}`", err.to_string());
                                break;
                            }
                            tokio::select! {
                                override_color = color_receiver.recv() => {
                                    if let Some(new_color) = override_color {
                                        color = new_color
                                    }
                                }
                                _ = tokio::time::sleep(Duration::from_millis(500)) => break,
                            }
                        }
                    }
                }
                _ = self.update_colors() => {}
            }
        }
    }

    pub async fn update_colors(&mut self) {
        match &self.profile {
            ColorProfile::None => pending().await,
            ColorProfile::Single(color) => {
                self.io.set_color_all(color).await.unwrap();
                pending().await
            }
            ColorProfile::Multiple(colors) => {
                let color_steps = calculate_color_animation_steps(colors);
                self.run_color_animation(&color_steps).await;
            }
        }
    }

    /// Infinitely run a color animation and
    /// stop the animation while suspended.
    async fn run_color_animation(&mut self, color_steps: &[(Color, u32)]) {
        for step in color_steps.iter().cycle() {
            if let Err(err) = self.io.set_color_left(&step.0).await {
                tracing::error!("Failed setting keyboard colors: `{err}`")
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(step.1 as u64)) => {}
                _ = process_suspend(&mut self.suspend_receiver) => {}
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
    // More would be rather CPU intensive for a background job.
    let steps = transition_time / 80;

    if steps == 0 {
        color_steps.push((color, transition_time));
    } else {
        let r_diff = color.r as f64 - prev_color.r as f64;
        let g_diff = color.g as f64 - prev_color.g as f64;
        let b_diff = color.b as f64 - prev_color.b as f64;

        let decent_steps = decent_linear_steps(transition_time, &[r_diff, g_diff, b_diff]);

        // Use a lower step size if the animation is slow.
        // The human eye won't notice the lower fps but
        // the CPU usage will drop significantly.
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
