extern crate gamenet;

use gamenet::msg::game::SvTuneParams;
use gamenet::snap_obj::CharacterCore;
use gamenet::snap_obj::PlayerInput;
use gamenet::snap_obj::Tick;
use std::f32::consts::PI;
use std::fmt;
use std::ops;

pub const CHARACTER_SIZE: f32 = 28.0;
pub const DISABLE_HOOK_DISTANCE: f32 = 46.0;
pub const MAX_VELOCITY: f32 = 6000.0;

trait Round {
    fn round_to_int(self) -> i32;
}

impl Round for f32 {
    fn round_to_int(self) -> i32 {
        // TODO: Do overflow checking?
        self.round() as i32
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default)]
pub struct vec2 {
    x: f32,
    y: f32,
}

impl fmt::Debug for vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Display for vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ops::Add for vec2 {
    type Output = vec2;
    fn add(self, other: vec2) -> vec2 {
        vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::Sub for vec2 {
    type Output = vec2;
    fn sub(self, other: vec2) -> vec2 {
        vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Mul<f32> for vec2 {
    type Output = vec2;
    fn mul(self, scalar: f32) -> vec2 {
        vec2::new(self.x * scalar, self.y * scalar)
    }
}

impl vec2 {
    pub fn new(x: f32, y: f32) -> vec2 {
        vec2 { x: x, y: y }
    }
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y)
    }
    pub fn normalize(self) -> vec2 {
        self * (1.0 / self.length())
    }
    pub fn angle(self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }
    pub fn distance(first: vec2, second: vec2) -> f32 {
        (second - first).length()
    }
}

#[derive(Clone, Copy, Default)]
pub struct Angle {
    radians: f32,
}

impl Angle {
    pub fn from_radians(radians: f32) -> Angle {
        Angle {
            radians: radians,
        }
    }
    pub fn to_degrees(self) -> f32 {
        self.radians / 2.0 / PI * 360.0
    }
    pub fn to_radians(self) -> f32 {
        self.radians
    }
    pub fn to_direction(self) -> vec2 {
        let (y, x) = self.to_radians().sin_cos();
        vec2::new(x, y)
    }
    pub fn to_net(self) -> i32 {
        (self.to_radians() * 256.0).trunc() as i32
    }
}

impl fmt::Debug for Angle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}°", self.to_degrees())
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Default for Hook {
    fn default() -> Hook {
        Hook::Idle
    }
}

pub const HOOK_RETRACTED: i32 = -1;
pub const HOOK_IDLE: i32 = 0;
pub const HOOK_RETRACTING0: i32 = 1;
pub const HOOK_RETRACTING1: i32 = 2;
pub const HOOK_RETRACTING2: i32 = 3;
pub const HOOK_FLYING: i32 = 4;
pub const HOOK_ATTACHED_GRABBED: i32 = 5;


#[derive(Clone, Copy)]
pub enum Hook {
    /// Same as `Idle`, but won't fire a new hook until the hook button is
    /// released.
    Retracted,
    Idle,
    // Flying(position, direction)
    Flying(vec2, vec2),
    /// Grabbed a player.
    // Grabbed(player_id, tick)
    Grabbed(u32, u32),
    /// Attached to the ground.
    // Attached(pos)
    Attached(vec2),
    // Retracting0(pos)
    Retracting0(vec2),
    // Retracting1(pos)
    Retracting1(vec2),
    // Retracting2(pos)
    Retracting2(vec2),
}

impl Hook {
    fn net_state(&self) -> i32 {
        match *self {
            Hook::Retracted => HOOK_RETRACTED,
            Hook::Idle => HOOK_IDLE,
            Hook::Flying(..) => HOOK_FLYING,
            Hook::Attached(_) => HOOK_ATTACHED_GRABBED,
            Hook::Grabbed(..) => HOOK_ATTACHED_GRABBED,
            Hook::Retracting0(_) => HOOK_RETRACTING0,
            Hook::Retracting1(_) => HOOK_RETRACTING1,
            Hook::Retracting2(_) => HOOK_RETRACTING2,
        }
    }
    fn pos(&self) -> Option<vec2> {
        Some(match *self {
            Hook::Retracted => return None,
            Hook::Idle => return None,
            Hook::Flying(pos, _) => pos,
            Hook::Attached(pos) => pos,
            Hook::Grabbed(..) => unimplemented!(),
            Hook::Retracting0(pos) => pos,
            Hook::Retracting1(pos) => pos,
            Hook::Retracting2(pos) => pos,
        })
    }
    fn net_pos(&self) -> vec2 {
        self.pos().unwrap_or_default()
    }
    fn dir(&self) -> Option<vec2> {
        if let Hook::Flying(_, dir) = *self {
            Some(dir)
        } else {
            None
        }
    }
    fn net_dir(&self) -> vec2 {
        self.dir().unwrap_or_default()
    }
}

#[derive(Clone, Copy)]
enum MoveDirection {
    Left = -1,
    None,
    Right,
}

impl Default for MoveDirection {
    fn default() -> MoveDirection {
        MoveDirection::None
    }
}

impl MoveDirection {
    fn from_int(i: i32) -> MoveDirection {
        // TODO: Ensure that only -1, 0, 1 are passed to this function and then
        // assert that.
        if i < 0 {
            MoveDirection::Left
        } else if i > 0 {
            MoveDirection::Right
        } else {
            MoveDirection::None
        }
    }
    fn as_int(self) -> i32 {
        match self {
            MoveDirection::Left => -1,
            MoveDirection::None => 0,
            MoveDirection::Right => 1,
        }
    }
    fn as_float(self) -> f32 {
        self.as_int() as f32
    }
}

pub struct Character {
    pos: vec2,
    vel: vec2,
    hook: Hook,
    jumped_already: bool,
    used_airjump: bool,
    angle: Angle,
    move_direction: MoveDirection,
}

pub enum CollisionType {
    Normal,
    Unhookable,
}

pub trait Collision {
    fn check_point(&mut self, pos: vec2) -> Option<CollisionType>;
    fn check_line(&mut self, from: vec2, to: vec2) -> Option<(vec2, CollisionType)>;
    fn move_box(&mut self, pos: vec2, vel: vec2, box_: vec2) -> vec2;
}

impl Character {
    pub fn spawn(pos: vec2) -> Character {
        Character {
            pos: pos,
            vel: Default::default(),
            hook: Default::default(),
            jumped_already: Default::default(),
            used_airjump: Default::default(),
            angle: Default::default(),
            move_direction: Default::default(),
        }
    }
    pub fn tick<C: Collision>(&mut self, collision: &mut C, input: PlayerInput, tuning: &SvTuneParams)
    {
        // Code copied from CCharacterCore::Tick

        const SIZE: f32 = CHARACTER_SIZE;
        let bottom_left = self.pos + vec2::new(-SIZE / 2.0, SIZE / 2.0 + 5.0);
        let bottom_right = self.pos + vec2::new(SIZE / 2.0, SIZE / 2.0 + 5.0);
        let grounded =
            collision.check_point(bottom_left).is_some() ||
            collision.check_point(bottom_right).is_some();

        let target_dir = vec2::new(input.target_x as f32, input.target_y as f32).normalize();
        let max_speed = (if grounded { tuning.ground_control_speed } else { tuning.air_control_speed }).to_float();
        let accel = (if grounded { tuning.ground_control_accel } else { tuning.air_control_accel }).to_float();
        let friction = (if grounded { tuning.ground_friction } else { tuning.air_friction }).to_float();

        self.move_direction = MoveDirection::from_int(input.direction);
        self.angle = target_dir.angle();

        if input.jump != 0 {
            if !self.jumped_already {
                if grounded {
                    self.vel.y = -tuning.ground_jump_impulse.to_float();
                } else if !self.used_airjump {
                    self.vel.y = -tuning.air_jump_impulse.to_float();
                    self.used_airjump = true;
                }
                self.jumped_already = true;
            }
        } else {
            self.jumped_already = false;
        }

        if input.hook != 0 {
            if let Hook::Idle = self.hook {
                self.hook = Hook::Flying(self.pos + target_dir * SIZE * 1.5, target_dir);
            }
        } else {
            self.hook = Hook::Idle;
        }

        self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, self.move_direction.as_float() * accel);
        if let MoveDirection::None = self.move_direction {
            self.vel.x *= friction;
        }

        if grounded {
            self.used_airjump = false;
        }

        match self.hook {
            Hook::Idle => {},
            Hook::Retracted => {},
            Hook::Flying(pos, dir) => {
                let new_pos = pos + dir * tuning.hook_fire_speed.to_float();
                if vec2::distance(new_pos, self.pos) > tuning.hook_length.to_float() {
                    let new_pos = self.pos + (new_pos - self.pos) * tuning.hook_length.to_float();
                    self.hook = Hook::Retracting0(new_pos);
                }
                if let Some((p, t)) = collision.check_line(pos, new_pos) {
                    match t {
                        CollisionType::Normal => self.hook = Hook::Attached(p),
                        CollisionType::Unhookable => self.hook = Hook::Retracting0(p),
                    }
                }
                if let Hook::Flying(..) = self.hook {
                    self.hook = Hook::Flying(new_pos, dir);
                }
            }
            Hook::Retracting0(pos) => self.hook = Hook::Retracting1(pos),
            Hook::Retracting1(pos) => self.hook = Hook::Retracting2(pos),
            Hook::Retracting2(_) => self.hook = Hook::Retracted,
            Hook::Grabbed(..) => unimplemented!(),
            Hook::Attached(_) => {}, // See below.
        }

        if let Hook::Attached(hook_pos) = self.hook {
            // Disable hook drag if we're too close.
            if vec2::distance(hook_pos, self.pos) > DISABLE_HOOK_DISTANCE {
                let mut hook_vel = (hook_pos - self.pos).normalize() * tuning.hook_drag_accel.to_float();

                // Hooking down has 30% of the power of hooking up.
                if hook_vel.y > 0.0 {
                    hook_vel.y *= 0.3;
                }

                // If we're hooking in the direction we want to move, give it
                // more power.
                if hook_vel.x * self.move_direction.as_float() > 0.0 {
                    hook_vel.x *= 0.95;
                } else {
                    hook_vel.x *= 0.75;
                }

                let new_vel = self.vel + hook_vel;
                // Only increase the velocity if it's below the limit for hook,
                // or if we're slowing down.
                if new_vel.length() < tuning.hook_drag_speed.to_float()
                    || new_vel.length() < self.vel.length()
                {
                    self.vel = new_vel;
                }
            }
        }

        // Clamp velocity.
        if self.vel.length() > MAX_VELOCITY {
            self.vel = self.vel.normalize() * MAX_VELOCITY;
        }
    }
    pub fn move_<C: Collision>(&mut self, collision: &mut C, tuning: &SvTuneParams) {
        let ramp_value = velocity_ramp(self.vel.length() * 50.0, tuning);
        let modified_vel = vec2::new(self.vel.x * ramp_value, self.vel.y);
        let box_ = vec2::new(CHARACTER_SIZE, CHARACTER_SIZE);
        self.pos = collision.move_box(self.pos, modified_vel, box_);
    }
    fn net_jumped(&self) -> i32 {
        ((self.used_airjump as i32) << 1) | (self.jumped_already as i32)
    }
    pub fn write(&self) -> CharacterCore {
        let network_vel = self.vel * 256.0;
        let hook_pos = self.hook.net_pos();
        let hook_dir = self.hook.net_dir() * 256.0;
        CharacterCore {
            x: self.pos.x.round_to_int(),
            y: self.pos.y.round_to_int(),
            vel_x: network_vel.x.round_to_int(),
            vel_y: network_vel.y.round_to_int(),
            hook_state: self.hook.net_state(),
            // TODO!
            hook_tick: Tick(0),
            hook_x: hook_pos.x.round_to_int(),
            hook_y: hook_pos.y.round_to_int(),
            hook_dx: hook_dir.x.round_to_int(),
            hook_dy: hook_dir.y.round_to_int(),
            hooked_player: -1,
            jumped: self.net_jumped(),
            direction: self.move_direction.as_int(),
            angle: self.angle.to_net(),
            tick: 0,
        }
    }
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn saturated_add(min: f32, max: f32, value: f32, modifier: f32) -> f32 {
    if modifier < 0.0 {
        if value < min {
            return value;
        }
    } else {
        if value > max {
            return value;
        }
    }
    clamp(value + modifier, min, max)
}

pub fn velocity_ramp(value: f32, tuning: &SvTuneParams) -> f32 {
    let start = tuning.velramp_start.to_float();
    if value < start {
        1.0
    } else {
        let curvature = tuning.velramp_curvature.to_float();
        let range = tuning.velramp_range.to_float();
        1.0 / curvature.powf((value - start) / range)
    }
}
