use libtw2_common::num::Cast;
use libtw2_common::num::CastFloat;
use libtw2_gamenet::msg::game::SvTuneParams;
use libtw2_gamenet::snap_obj::CharacterCore;
use libtw2_gamenet::snap_obj::PlayerInput;
use libtw2_gamenet::snap_obj::Tick;
use std::f32::consts::PI;
use std::fmt;
use std::ops;

pub const CHARACTER_SIZE: f32 = 28.0;
pub const DISABLE_HOOK_DISTANCE: f32 = 46.0;
pub const MAX_VELOCITY: f32 = 6000.0;
pub const MAX_HOOK_GRAB_TIME: u32 = 60; // 1.2 s with 50 Hz ticks.

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CharacterId(pub u32);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default)]
pub struct vec2 {
    pub x: f32,
    pub y: f32,
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

impl ops::Div<f32> for vec2 {
    type Output = vec2;
    fn div(self, scalar: f32) -> vec2 {
        vec2::new(self.x / scalar, self.y / scalar)
    }
}

impl ops::AddAssign for vec2 {
    fn add_assign(&mut self, rhs: vec2) {
        *self = *self + rhs;
    }
}

impl ops::MulAssign<f32> for vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl vec2 {
    pub fn new(x: f32, y: f32) -> vec2 {
        vec2 { x: x, y: y }
    }
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn normalize(self) -> vec2 {
        self * (1.0 / self.length())
    }
    pub fn angle(self) -> Angle {
        // TODO: Might be a source of inconsistency. :(
        Angle::from_radians(self.y.atan2(self.x))
    }
    pub fn distance(first: vec2, second: vec2) -> f32 {
        (second - first).length()
    }
    pub fn dot(first: vec2, second: vec2) -> f32 {
        first.x * second.x + first.y * second.y
    }
    pub fn mix(first: vec2, second: vec2, v: f32) -> vec2 {
        // Needs to be an exact copy of the original, otherwise could use:
        // first * (1.0 - v) + second * v
        first + (second - first) * v
    }
    pub fn closest_point_on_line(self, line_p0: vec2, line_p1: vec2) -> vec2 {
        let line = line_p1 - line_p0;
        let t = vec2::dot(line.normalize(), self - line_p0) / line.length();
        vec2::mix(line_p0, line_p1, clamp(t, 0.0, 1.0))
    }
}

#[derive(Clone, Copy, Default)]
pub struct Angle {
    radians: f32,
}

impl Angle {
    pub fn from_radians(radians: f32) -> Angle {
        Angle { radians: radians }
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
        (self.to_radians() * 256.0).trunc_to_i32()
    }
    pub fn from_net(net: i32) -> Angle {
        Angle::from_radians((net as f32) / 256.0)
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
    Grabbed(CharacterId, u32),
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
            Hook::Grabbed(..) => return None,
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
    fn tick(&self) -> Option<u32> {
        match *self {
            Hook::Grabbed(_, tick) => Some(tick),
            _ => None,
        }
    }
    fn net_tick(&self) -> Tick {
        Tick(self.tick().unwrap_or_default().assert_i32())
    }
    fn hooked_player(&self) -> Option<CharacterId> {
        match *self {
            Hook::Grabbed(cid, _) => Some(cid),
            _ => None,
        }
    }
    fn net_hooked_player(&self) -> i32 {
        self.hooked_player().map(|c| c.0 as i32).unwrap_or(-1)
    }
    fn from_net(
        hook_state: i32,
        pos: vec2,
        dir: vec2,
        hooked_player: i32,
        hook_tick: Tick,
    ) -> Hook {
        // TODO: Warn on weird values.
        match hook_state {
            HOOK_RETRACTED => Hook::Retracted,
            HOOK_IDLE => Hook::Idle,
            HOOK_FLYING => Hook::Flying(pos, dir),
            HOOK_ATTACHED_GRABBED => {
                if hooked_player == -1 {
                    Hook::Attached(pos)
                } else {
                    Hook::Grabbed(CharacterId(hooked_player as u32), hook_tick.0 as u32)
                }
            }
            HOOK_RETRACTING0 => Hook::Retracting0(pos),
            HOOK_RETRACTING1 => Hook::Retracting1(pos),
            HOOK_RETRACTING2 => Hook::Retracting2(pos),
            // TODO: Don't fail silently. :(
            _ => Hook::Retracted,
        }
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

#[derive(Clone, Copy)]
pub struct Character {
    pos: vec2,
    vel: vec2,
    hook: Hook,
    jumped_already: bool,
    used_airjump: bool,
    angle: Angle,
    move_direction: MoveDirection,
}

pub trait OtherCharacters {
    type Iter;
    fn is_self(&self, cid: CharacterId) -> bool;
    // Should panic if the cid points to itself or to no character at all.
    fn get(&self, cid: CharacterId) -> Character;
    fn modify<F: FnOnce(&mut Character)>(&self, cid: CharacterId, f: F);
    fn iter(&self) -> Self::Iter;
    fn next(&self, iter: &mut Self::Iter) -> Option<(CharacterId, Character)>;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CollisionType {
    Normal,
    Unhookable,
}

pub trait Collision {
    fn check_point(&mut self, pos: vec2) -> Option<CollisionType>;
    fn check_line(&mut self, from: vec2, to: vec2) -> Option<(vec2, CollisionType)> {
        let dist = vec2::distance(from, to);
        // Overflow?
        let end = (dist + 1.0).trunc_to_i32();
        // Note: `0..end+1` would work better here, but we have to stay
        // compatible to the original code.
        for i in 0..end {
            // Note: Should rather be `/ end` instead of `/ dist`.
            let point = vec2::mix(from, to, i as f32 / dist);
            if let Some(col) = self.check_point(point) {
                return Some((point, col));
            }
        }
        None
    }
    fn check_box(&mut self, pos: vec2, box_: vec2) -> bool {
        let diff1 = box_ * 0.5;
        let diff2 = vec2::new(diff1.x, -diff1.y);
        false
            || self.check_point(pos + diff1).is_some()
            || self.check_point(pos - diff1).is_some()
            || self.check_point(pos + diff2).is_some()
            || self.check_point(pos - diff2).is_some()
    }
    fn move_box(&mut self, mut pos: vec2, mut vel: vec2, box_: vec2) -> (vec2, vec2) {
        let dist = vel.length();
        // Magic number :(
        if dist > 0.00001 {
            let end = dist.round_to_i32();
            let fraction = 1.0 / (end + 1) as f32;
            for _ in 0..end + 1 {
                let mut new_pos = pos + vel * fraction;
                if self.check_box(new_pos, box_) {
                    let mut hit = false;
                    if self.check_box(vec2::new(pos.x, new_pos.y), box_) {
                        new_pos.y = pos.y;
                        vel.y = 0.0;
                        hit = true;
                    }
                    if self.check_box(vec2::new(new_pos.x, pos.y), box_) {
                        new_pos.x = pos.x;
                        vel.x = 0.0;
                        hit = true;
                    }
                    if !hit {
                        // Original comment: This is a real _corner case_!
                        //
                        // Unfortunately, you actually see this happen, when
                        // diagonally moving towards an corner.
                        new_pos = pos;
                        vel = vec2::new(0.0, 0.0);
                    }
                }
                pos = new_pos;
            }
        }
        (pos, vel)
    }
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
    pub fn tick<C, OC>(
        &mut self,
        collision: &mut C,
        other_characters: &mut OC,
        input: PlayerInput,
        tuning: &SvTuneParams,
    ) where
        C: Collision,
        OC: OtherCharacters,
    {
        // Code copied from CCharacterCore::Tick

        const SIZE: f32 = CHARACTER_SIZE;
        let bottom_left = self.pos + vec2::new(-SIZE / 2.0, SIZE / 2.0 + 5.0);
        let bottom_right = self.pos + vec2::new(SIZE / 2.0, SIZE / 2.0 + 5.0);
        let grounded = collision.check_point(bottom_left).is_some()
            || collision.check_point(bottom_right).is_some();

        let target_dir = vec2::new(input.target_x as f32, input.target_y as f32).normalize();
        let max_speed = (if grounded {
            tuning.ground_control_speed
        } else {
            tuning.air_control_speed
        })
        .to_float();
        let accel = (if grounded {
            tuning.ground_control_accel
        } else {
            tuning.air_control_accel
        })
        .to_float();
        let friction = (if grounded {
            tuning.ground_friction
        } else {
            tuning.air_friction
        })
        .to_float();

        self.move_direction = MoveDirection::from_int(input.direction);
        self.angle = target_dir.angle();

        self.vel.y += tuning.gravity.to_float();

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

        self.vel.x = saturated_add(
            -max_speed,
            max_speed,
            self.vel.x,
            self.move_direction.as_float() * accel,
        );
        if let MoveDirection::None = self.move_direction {
            self.vel.x *= friction;
        }

        if grounded {
            self.used_airjump = false;
        }

        match self.hook {
            Hook::Idle => {}
            Hook::Retracted => {}
            Hook::Flying(pos, dir) => {
                let mut new_pos = pos + dir * tuning.hook_fire_speed.to_float();
                if vec2::distance(new_pos, self.pos) > tuning.hook_length.to_float() {
                    new_pos =
                        self.pos + (new_pos - self.pos).normalize() * tuning.hook_length.to_float();
                    self.hook = Hook::Retracting0(new_pos);
                }
                if let Some((p, t)) = collision.check_line(pos, new_pos) {
                    match t {
                        CollisionType::Normal => self.hook = Hook::Attached(p),
                        CollisionType::Unhookable => self.hook = Hook::Retracting0(p),
                    }
                }
                if tuning.player_hooking.to_float() != 0.0 {
                    let mut iter = other_characters.iter();
                    let mut min_distance = None;
                    while let Some((cid, other)) = other_characters.next(&mut iter) {
                        let closest = other.pos.closest_point_on_line(pos, new_pos);
                        if vec2::distance(other.pos, closest) < SIZE {
                            let new_minimum;
                            let distance = vec2::distance(pos, other.pos);
                            if let Some(md) = min_distance {
                                new_minimum = distance < md;
                            } else {
                                new_minimum = true;
                            }
                            if new_minimum {
                                min_distance = Some(distance);
                                self.hook = Hook::Grabbed(cid, 0);
                            }
                        }
                    }
                }
                if let Hook::Flying(..) = self.hook {
                    self.hook = Hook::Flying(new_pos, dir);
                }
            }
            Hook::Retracting0(pos) => self.hook = Hook::Retracting1(pos),
            Hook::Retracting1(pos) => self.hook = Hook::Retracting2(pos),
            Hook::Retracting2(_) => self.hook = Hook::Retracted,
            Hook::Grabbed(..) => {} // See below.
            Hook::Attached(_) => {} // See below.
        }

        if let Hook::Attached(hook_pos) = self.hook {
            // Disable hook drag if we're too close.
            if vec2::distance(hook_pos, self.pos) > DISABLE_HOOK_DISTANCE {
                let mut hook_vel =
                    (hook_pos - self.pos).normalize() * tuning.hook_drag_accel.to_float();

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

        if let Hook::Grabbed(hooked, mut tick) = self.hook {
            // NOTE: Tick starts at 1, this is already executed in the tick the
            // hook grabs the player.
            tick += 1;
            if tick > MAX_HOOK_GRAB_TIME {
                self.hook = Hook::Retracted;
            } else {
                self.hook = Hook::Grabbed(hooked, tick);
            }
        }

        let mut iter = other_characters.iter();
        while let Some((cid, other)) = other_characters.next(&mut iter) {
            let distance = vec2::distance(self.pos, other.pos);

            if tuning.player_collision.to_float() != 0.0 && distance < SIZE * 1.25 && distance > 0.0
            {
                let dir_to_self = vec2::normalize(self.pos - other.pos);
                let arbitrary_factor = SIZE * 1.45 - distance;
                let additional_vel;
                if self.vel.length() > 0.0001 {
                    additional_vel =
                        1.0 - (vec2::dot(self.vel.normalize(), dir_to_self) + 1.0) / 2.0;
                } else {
                    additional_vel = 0.5;
                }
                self.vel += dir_to_self * arbitrary_factor * additional_vel * 0.75;
                self.vel *= 0.85;
            }

            if let Hook::Grabbed(hooked_cid, _) = self.hook {
                if tuning.player_hooking.to_float() != 0.0
                    && cid == hooked_cid
                    && distance > SIZE * 1.5
                {
                    let dir = vec2::normalize(self.pos - other.pos);
                    let accel = tuning.hook_drag_accel.to_float() * distance
                        / tuning.hook_length.to_float();
                    let drag_speed = tuning.hook_drag_speed.to_float();
                    other_characters.modify(cid, |other| {
                        other.vel.x = saturated_add(
                            -drag_speed,
                            drag_speed,
                            other.vel.x,
                            accel * dir.x * 1.5,
                        );
                        other.vel.y = saturated_add(
                            -drag_speed,
                            drag_speed,
                            other.vel.y,
                            accel * dir.y * 1.5,
                        );
                    });
                    self.vel.x =
                        saturated_add(-drag_speed, drag_speed, self.vel.x, -accel * dir.x * 0.25);
                    self.vel.y =
                        saturated_add(-drag_speed, drag_speed, self.vel.y, -accel * dir.y * 0.25);
                }
            }
        }

        // Clamp velocity.
        if self.vel.length() > MAX_VELOCITY {
            self.vel = self.vel.normalize() * MAX_VELOCITY;
        }
    }
    pub fn move_<C, OC>(
        &mut self,
        collision: &mut C,
        other_characters: &mut OC,
        tuning: &SvTuneParams,
    ) where
        C: Collision,
        OC: OtherCharacters,
    {
        let ramp_value = velocity_ramp(self.vel.length() * 50.0, tuning);
        self.vel.x *= ramp_value;
        let box_ = vec2::new(CHARACTER_SIZE, CHARACTER_SIZE);
        let (new_pos, new_vel) = collision.move_box(self.pos, self.vel, box_);
        self.vel = new_vel;
        self.vel.x *= 1.0 / ramp_value;

        if tuning.player_collision.to_float() != 0.0 {
            let distance = vec2::distance(self.pos, new_pos);
            let end = distance.trunc_to_i32() + 1;
            let mut last_pos = self.pos;
            for i in 0..end {
                let fraction = i as f32 / distance;
                let pos = vec2::mix(self.pos, new_pos, fraction);

                let mut iter = other_characters.iter();
                while let Some((_, other)) = other_characters.next(&mut iter) {
                    let d = vec2::distance(pos, other.pos);
                    if 0.0 < d && d < CHARACTER_SIZE {
                        if fraction > 0.0 {
                            self.pos = last_pos;
                        } else if vec2::distance(new_pos, other.pos) > d {
                            self.pos = new_pos;
                        }
                        return;
                    }
                }
                last_pos = pos;
            }
        }
        self.pos = new_pos;
    }
    fn net_jumped(&self) -> i32 {
        ((self.used_airjump as i32) << 1) | (self.jumped_already as i32)
    }
    fn used_airjump_from_net(jumped: i32) -> bool {
        // TODO: Check that jumped has valid values.
        (jumped & 2) != 0
    }
    fn jumped_already_from_net(jumped: i32) -> bool {
        (jumped & 1) != 0
    }
    pub fn to_net(&self) -> CharacterCore {
        let network_vel = self.vel * 256.0;
        let hook_pos = self.hook.net_pos();
        let hook_dir = self.hook.net_dir() * 256.0;
        CharacterCore {
            x: self.pos.x.round_to_i32(),
            y: self.pos.y.round_to_i32(),
            vel_x: network_vel.x.round_to_i32(),
            vel_y: network_vel.y.round_to_i32(),
            hook_state: self.hook.net_state(),
            hook_tick: self.hook.net_tick(),
            hook_x: hook_pos.x.round_to_i32(),
            hook_y: hook_pos.y.round_to_i32(),
            hook_dx: hook_dir.x.round_to_i32(),
            hook_dy: hook_dir.y.round_to_i32(),
            hooked_player: self.hook.net_hooked_player(),
            jumped: self.net_jumped(),
            direction: self.move_direction.as_int(),
            angle: self.angle.to_net(),
            tick: 0,
        }
    }
    pub fn from_net(core: &CharacterCore) -> Character {
        Character {
            pos: vec2::new(core.x as f32, core.y as f32),
            vel: vec2::new(core.vel_x as f32, core.vel_y as f32) / 256.0,
            hook: Hook::from_net(
                core.hook_state,
                vec2::new(core.hook_x as f32, core.hook_y as f32),
                vec2::new(core.hook_dx as f32, core.hook_dy as f32) / 256.0,
                core.hooked_player,
                core.hook_tick,
            ),
            used_airjump: Character::used_airjump_from_net(core.jumped),
            jumped_already: Character::jumped_already_from_net(core.jumped),
            angle: Angle::from_net(core.angle),
            move_direction: MoveDirection::from_int(core.direction),
        }
    }
    pub fn quantize(&mut self) {
        *self = Character::from_net(&self.to_net());
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
