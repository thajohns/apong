use crate::PSpaceTransform;
use crate::a::AudioWorldState;
pub type V2f = nalgebra::Vector2<f64>;

#[derive(Debug, Clone)]
pub struct World {
    pub ball_pos: V2f,
    pub ball_vel: V2f,
    pub y_bounds: (f64, f64),
    pub x_bounds: (f64, f64),
    pub paddles: [Paddle; 2],
}

#[derive(Debug, Clone)]
pub struct Paddle {
    pub x: f64,
    pub ys: (f64, f64), // lower and upper end of paddle
    pub yvel: f64,
}

impl World {
    #[allow(non_snake_case)]
    pub fn do_physics(&mut self, Δt: f64) {
        self.ball_pos += self.ball_vel * Δt;
        // bounce off of paddles
        if self.ball_pos.x < self.paddles[0].x && self.paddles[0].intersects_y(self.ball_pos) && self.ball_vel.x < 0.0 {
            self.ball_vel.x = -self.ball_vel.x;
        }
        if self.ball_pos.x > self.paddles[1].x && self.paddles[0].intersects_y(self.ball_pos) && self.ball_vel.x > 0.0 {
            self.ball_vel.x = -self.ball_vel.x;
        }
        // bounce off of vertical boundaries of arena
        if (self.ball_pos.y < self.y_bounds.0 && self.ball_vel.y < 0.0) || (self.ball_pos.y > self.y_bounds.1 && self.ball_vel.y > 0.0) {
            self.ball_vel.y = -self.ball_vel.y;
        }
        // move paddles
        self.paddles[0].do_physics(Δt, self.y_bounds);
        self.paddles[1].do_physics(Δt, self.y_bounds);
    }

    pub fn game_over(&self) -> bool {
        self.ball_pos.x < self.x_bounds.0 || self.ball_pos.x > self.x_bounds.1
    }

    pub fn to_audio_state(&self, psp: &PSpaceTransform) -> AudioWorldState {
        let ys = [self.ball_pos.y, self.paddles[0].midpoint(), self.paddles[1].midpoint()];
        let fs = [psp.tf(ys[0]), psp.tf(ys[1]), psp.tf(ys[2])];
        AudioWorldState {
            fs,
            dc: self.ball_pos.x,
        }
    }
}

impl Paddle {
    fn midpoint(&self) -> f64 {
        (self.ys.0 + self.ys.1) / 2.0
    }

    fn intersects_y(&self, pos: V2f) -> bool {
        self.ys.0 <= pos.y && pos.y < self.ys.1
    }

    pub fn new(x: f64, y: f64, width: f64) -> Self {
        Self {
            x,
            ys: (y - width * 0.5, y + width * 0.5),
            yvel: 0.0,
        }
    }

    #[allow(non_snake_case)]
    pub fn do_physics(&mut self, Δt: f64, y_bounds: (f64, f64)) {
        if self.ys.1 > y_bounds.1 {
            self.yvel = f64::min(0.0, self.yvel)
        }
        if self.ys.0 < y_bounds.0 {
            self.yvel = f64::max(0.0, self.yvel)
        }
        let d = self.yvel * Δt;
        self.ys.0 += d;
        self.ys.1 += d;
    }
}
