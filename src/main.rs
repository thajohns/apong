mod a;
mod pong;
mod window;

use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc};

use nalgebra::vector;

use pong::{Paddle, World};
use a::{run_audio, AudioWorldState};

fn main() {
    let exit = Arc::new(AtomicBool::new(false));
    let (key_vec_send, key_vec_recv) = mpsc::sync_channel(8);
    let (audio_state_send, audio_state_recv) = mpsc::sync_channel(8);
    let psp = PSpaceTransform::from_points((-0.5, 100.0), (0.5, 200.0));
    println!("psp: {psp:?}");
    let e2 = exit.clone();
    let t1 = std::thread::spawn(move || {
        run_game(key_vec_recv, &psp, audio_state_send, e2);
    });
    let e2 = exit.clone();
    let t2 = std::thread::spawn(move || run_audio(audio_state_recv, e2));
    let (ev_loop, mut app) = window::init_window(key_vec_send, exit.clone());
    ev_loop
        .run_app(&mut app)
        .expect("could not initialize window event loop");
    std::mem::drop(app);

    t1.join().unwrap();
    t2.join().unwrap();
}

fn run_game(key_vec_recv: mpsc::Receiver<[bool; 2]>, psp: &PSpaceTransform, audio_state_channel: mpsc::SyncSender<AudioWorldState>, exit: Arc<AtomicBool>) {
    let mut key_state = [false; 2];

    let mut game_world = World {
        ball_pos: vector![0.5, 0.0],
        ball_vel: vector![0.1, 0.08],
        x_bounds: (0.0, 1.0),
        y_bounds: (-0.5, 0.5),
        paddles: [Paddle::new(0.02, 0.0, 0.1), Paddle::new(0.98, 0.0, 0.1)],
    };

    while !exit.load(atomic::Ordering::Relaxed) {
        // compute paddle velocity
        key_state = key_vec_recv.try_iter().last().unwrap_or(key_state);
        if let Err(mpsc::TryRecvError::Disconnected) = key_vec_recv.try_recv() {
            break;
        }
        game_world.paddles[0].yvel = match key_state {
            [true, false] => -0.1,
            [false, true] => 0.1,
            _ => 0.0,
        };

        #[allow(non_snake_case)]
        let Δt = 0.01f64;
        println!("{game_world:?}");
        game_world.do_physics(Δt);
        if game_world.game_over() {
            break;
        }
        audio_state_channel.send(game_world.to_audio_state(psp)).unwrap_or_else(|_| exit.store(true, atomic::Ordering::Relaxed)) ;
        std::thread::sleep(std::time::Duration::from_secs_f64(Δt));
    }

    exit.store(true, atomic::Ordering::Relaxed);
}

#[derive(Debug, Clone)]
struct PSpaceTransform(f64, f64);

impl PSpaceTransform {
    fn tf(&self, x: f64) -> f64 {
        (self.0 * x + self.1).exp()
    }

    fn from_points(p1: (f64, f64), p2: (f64, f64)) -> Self {
        let (l1, l2) = (p1.1.ln(), p2.1.ln());
        let m = (l2 - l1) / (p2.0 - p1.0);
        let b = l2 - p1.0 * m;
        Self(m, b)
    }
}
