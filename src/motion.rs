use wiiboard::WiiBoardRealtime;
use nalgebra::{Vector3, Matrix4};
use gilrs::{Axis, GamepadId, Gilrs};
use anyhow::{Context, Result, format_err};
use pid::Pid;

const SENSITIVITY_X: f32 = 0.01;
const SENSITIVITY_Y: f32 = 0.02;
const SPEED: f32 = 0.1;

pub struct PlayerMovement {
    position: Vector3<f32>,
    yaw: f32,
    speed: f32,
    input_device: Box<dyn TwoAxis>,
}

impl PlayerMovement {
    pub fn new(balance: bool) -> Result<Self> {
        let input_device: Box<dyn TwoAxis> = match balance {
            true => Box::new(WiiBoardRealtime::new(5, 5)),
            false => Box::new(GamepadAxes::new()?),
        };

        let yaw = std::f32::consts::FRAC_PI_2;

        Ok(Self {
            position: Vector3::zeros(),
            yaw,
            speed: 0.0,
            input_device,
        })
    }

    pub fn player_transform(&mut self) -> Matrix4<f32> {
        let (x, y) = self.input_device.get_axes().expect("Input error");
        self.yaw += x * SENSITIVITY_X;
        self.speed += y * SENSITIVITY_Y;

        Matrix4::new_translation(&self.position) * Matrix4::from_euler_angles(0., self.yaw, 0.)
    }
}

trait TwoAxis {
    fn get_axes(&mut self) -> Result<(f32, f32)>;
}

impl TwoAxis for WiiBoardRealtime {
    fn get_axes(&mut self) -> Result<(f32, f32)> {
         if let Some(data) = self.poll()? {
            let total = data.top_left + data.top_right + data.bottom_left + data.bottom_right;
            if total > 0.0 {
                let x = ((data.top_right + data.bottom_right) / total) * 2. - 1.;
                let y = ((data.top_left + data.top_right) / total) * 2. - 1.;
                return Ok((x, y));
            }
         }

        Ok((0., 0.))
    }
}

struct GamepadAxes {
    gilrs: Gilrs,
    gamepad: GamepadId,
}

impl GamepadAxes {
    pub fn new() -> Result<Self> {
        let gilrs = Gilrs::new().map_err(|e| format_err!("gilrs failed to init {}", e))?;
        let (gamepad, _) = gilrs.gamepads().next().context("No gamepads found")?;
        Ok(Self { gilrs, gamepad })
    }
}

impl TwoAxis for GamepadAxes {
    fn get_axes(&mut self) -> Result<(f32, f32)> {
        self.gilrs.next_event();
        let x = self
            .gilrs
            .gamepad(self.gamepad)
            .axis_data(Axis::LeftStickX)
            .map(|v| v.value())
            .unwrap_or(0.0);
        let y = self
            .gilrs
            .gamepad(self.gamepad)
            .axis_data(Axis::LeftStickY)
            .map(|v| v.value())
            .unwrap_or(0.0);
        Ok((-x, y))
    }
}
