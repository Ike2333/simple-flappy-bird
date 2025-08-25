use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;
const PLAYER_SCREEN_X: i32 = 10;
const DIST_BETWEEN: i32 = 50;

struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    obstacles: Vec<Obstacle>,
    score: i32,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(0, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,
            obstacles: vec![Obstacle::new(SCREEN_WIDTH, 0)],
            score: 0,
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flippy Bird");
        ctx.print_centered(8, "(P) play");
        ctx.print_centered(9, "(Q) quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead");
        ctx.print_centered(8, "(P) play again");
        ctx.print_centered(9, "(Q) quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);
        ctx.print(0, 0, "Press space to flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        for obs in self.obstacles.iter_mut() {
            obs.render(ctx, self.player.x);

            if self.player.x > obs.x && !obs.passed {
                obs.passed = true;
                self.score += 1;
            }

            if obs.hit_obstacle(&self.player) {
                self.mode = GameMode::End;
            }
        }

        // 创建下一个障碍物, 移除第一个障碍物
        if let Some(last) = self.obstacles.last() {
            if self.player.x > last.x - DIST_BETWEEN && self.obstacles.len() < 2 {
                self.obstacles
                    .push(Obstacle::new(last.x + DIST_BETWEEN, self.score));
            }
        }
        if let Some(first) = self.obstacles.first() {
            let screen_x = first.x - self.player.x + PLAYER_SCREEN_X;
            if screen_x < 0 {
                self.obstacles.remove(0);
            }
        }

        if self.player.y > SCREEN_HEIGHT {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(0, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.obstacles = vec![Obstacle::new(SCREEN_WIDTH, 0)];
        self.score = 0;
    }
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(PLAYER_SCREEN_X, self.y, YELLOW, BLACK, to_cp437('@'))
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity as i32;
        self.x += 1;

        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
    passed: bool,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
            passed: false,
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        // 增加障碍物x轴偏移量
        let screen_x = self.x - player_x + PLAYER_SCREEN_X;
        let half_size = self.size / 2;
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_blow_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_blow_gap)
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flippy Bird")
        .build()?;
    main_loop(context, State::new())
}
