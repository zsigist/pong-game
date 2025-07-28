#![windows_subsystem = "windows"]
use macroquad::prelude::*;
use std::collections::HashMap;

const PADDLE_W: f32 = 20.0;
const PADDLE_H: f32 = 120.0;
const BALL_SIZE: f32 = 24.0;
const WIN_SCORE: i32 = 5;
const COUNTDOWN: f64 = 3.0;

#[derive(PartialEq)]
enum GameState {
    EnterName1,
    EnterName2,
    Countdown,
    InGame,
    GameOver,
}

struct Paddle {
    rect: Rect,
    speed: f32,
    color: Color,
}
impl Paddle {
    fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            rect: Rect::new(x, y, PADDLE_W, PADDLE_H),
            speed: 7.0,
            color,
        }
    }
    fn draw(&self) {
        draw_neon_rect(self.rect.x, self.rect.y, self.rect.w, self.rect.h, self.color);
    }
    fn move_up(&mut self) {
        self.rect.y = (self.rect.y - self.speed).max(0.0);
    }
    fn move_down(&mut self) {
        self.rect.y = (self.rect.y + self.speed).min(screen_height() - self.rect.h);
    }
    fn update_x(&mut self, x: f32) {
        self.rect.x = x;
    }
}

struct Ball {
    rect: Rect,
    velocity: Vec2,
    color: Color,
    active: bool,
    prev_positions: Vec<Vec2>,
}
impl Ball {
    fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            rect: Rect::new(x, y, BALL_SIZE, BALL_SIZE),
            velocity: vec2(0., 0.),
            color,
            active: false,
            prev_positions: vec![],
        }
    }
    fn draw(&self) {
        if !self.active { return; }
        // Trail
        for (i, pos) in self.prev_positions.iter().enumerate() {
            let alpha = 0.06 * (self.prev_positions.len() - i) as f32;
            draw_circle(
                pos.x + BALL_SIZE / 2.,
                pos.y + BALL_SIZE / 2.,
                BALL_SIZE / 2. + 3.,
                Color::new(1., 0.9, 0.4, alpha),
            );
        }
        // Drop shadow
        draw_circle(
            self.rect.x + self.rect.w / 2. + 5.,
            self.rect.y + self.rect.h / 2. + 8.,
            BALL_SIZE / 2. + 6.,
            Color::from_rgba(0, 0, 0, 40),
        );
        draw_neon_ball(self);
    }
    fn update(&mut self, p1: &Paddle, p2: &Paddle) -> (bool, bool) {
        if !self.active { return (false, false); }
        self.prev_positions.insert(0, vec2(self.rect.x, self.rect.y));
        if self.prev_positions.len() > 14 { self.prev_positions.pop(); }
        self.rect.x += self.velocity.x;
        self.rect.y += self.velocity.y;

        if self.rect.y <= 0. || self.rect.y + self.rect.h >= screen_height() {
            self.velocity.y = -self.velocity.y;
        }
        if self.rect.overlaps(&p1.rect) && self.velocity.x < 0. {
            self.velocity.x = -self.velocity.x;
            self.velocity.y += rand::gen_range(-1., 1.);
        }
        if self.rect.overlaps(&p2.rect) && self.velocity.x > 0. {
            self.velocity.x = -self.velocity.x;
            self.velocity.y += rand::gen_range(-1., 1.);
        }
        if self.velocity.y.abs() < 1. {
            self.velocity.y = self.velocity.y.signum();
        }
        let mut p1_scored = false;
        let mut p2_scored = false;
        if self.rect.x < 0. {
            p2_scored = true;
            self.active = false;
            self.prev_positions.clear();
        }
        if self.rect.x > screen_width() {
            p1_scored = true;
            self.active = false;
            self.prev_positions.clear();
        }
        (p1_scored, p2_scored)
    }
    fn reset(&mut self) {
        self.rect.x = screen_width() / 2. - BALL_SIZE / 2.;
        self.rect.y = screen_height() / 2. - BALL_SIZE / 2.;
        self.active = false;
        self.velocity = vec2(0., 0.);
        self.prev_positions.clear();
    }
    fn launch(&mut self) {
        let mut angle: f32;
        loop {
            angle = rand::gen_range(-0.35 * std::f32::consts::PI, 0.35 * std::f32::consts::PI);
            if angle.abs() > 0.15 * std::f32::consts::PI { break; }
        }
        let xsign = if rand::gen_range(0, 2) == 0 { 1. } else { -1. };
        let speed = 13.0;
        self.velocity = vec2(angle.cos() * speed * xsign, angle.sin() * speed);
        self.active = true;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Modern Pong".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::EnterName1;
    let mut player1_name = String::new();
    let mut player2_name = String::new();
    let mut input_name = String::new();
    let mut winner_msg = String::new();
    let mut winner_name = String::new();

    let mut score1 = 0;
    let mut score2 = 0;
    let mut leaderboard: HashMap<String, i32> = HashMap::new();
    let mut full_tab = true;

    let p1_color = BLUE;
    let p2_color = ORANGE;
    let ball_color = YELLOW;

    let mut paddle1 = Paddle::new(36., screen_height() / 2. - PADDLE_H / 2., p1_color);
    let mut paddle2 = Paddle::new(screen_width() - 56., screen_height() / 2. - PADDLE_H / 2., p2_color);
    let mut ball = Ball::new(screen_width() / 2. - BALL_SIZE / 2., screen_height() / 2. - BALL_SIZE / 2., ball_color);

    let mut countdown = 0.;
    let mut prev_screen_width = screen_width();
    let mut prev_screen_height = screen_height();

    loop {
        draw_dynamic_gradient_background(get_time() as f32);

        let current_width = screen_width();
        let current_height = screen_height();

        if (current_width != prev_screen_width) || (current_height != prev_screen_height) {
            let ratio_p1 = paddle1.rect.y / prev_screen_height;
            let ratio_p2 = paddle2.rect.y / prev_screen_height;
            let ball_ratio_x = (ball.rect.x + BALL_SIZE / 2.) / prev_screen_width;
            let ball_ratio_y = (ball.rect.y + BALL_SIZE / 2.) / prev_screen_height;
            paddle1.rect.y = ratio_p1 * current_height;
            paddle2.rect.y = ratio_p2 * current_height;
            paddle1.update_x(36.);
            paddle2.update_x(current_width - 56.);
            ball.rect.x = ball_ratio_x * current_width - BALL_SIZE / 2.;
            ball.rect.y = ball_ratio_y * current_height - BALL_SIZE / 2.;

            prev_screen_width = current_width;
            prev_screen_height = current_height;
        }

        if is_key_pressed(KeyCode::Tab) {
            full_tab = !full_tab;
        }

        match state {
            GameState::EnterName1 => {
                draw_animated_menu_title();
                draw_text("Player 1, enter your name:", current_width / 2. - 200., current_height / 2., 36., p1_color);
                draw_text(&input_name, current_width / 2. - 200., current_height / 2. + 50., 40., p1_color);
                handle_name_input(&mut input_name);
                if is_key_pressed(KeyCode::Enter) && !input_name.trim().is_empty() {
                    player1_name = input_name.trim().to_string();
                    input_name.clear();
                    state = GameState::EnterName2;
                }
            }
            GameState::EnterName2 => {
                draw_animated_menu_title();
                draw_text("Player 2, enter your name:", current_width / 2. - 200., current_height / 2., 36., p2_color);
                draw_text(&input_name, current_width / 2. - 200., current_height / 2. + 50., 40., p2_color);
                handle_name_input(&mut input_name);
                if is_key_pressed(KeyCode::Enter) && !input_name.trim().is_empty() {
                    player2_name = input_name.trim().to_string();
                    input_name.clear();
                    score1 = 0;
                    score2 = 0;
                    ball.reset();
                    countdown = 0.;
                    state = GameState::Countdown;
                }
            }
            GameState::Countdown => {
                paddle1.draw();
                paddle2.draw();
                draw_text(
                    &format!("Get ready... {}", (COUNTDOWN - countdown).ceil() as i32),
                    current_width / 2. - 150.,
                    current_height / 2.,
                    64.,
                    WHITE,
                );
                countdown += get_frame_time() as f64;
                if countdown >= COUNTDOWN {
                    countdown = 0.;
                    ball.reset();
                    ball.launch();
                    state = GameState::InGame;
                }
                draw_animated_net();
                draw_score_and_net(score1, score2, full_tab, &player1_name, &player2_name);
            }
            GameState::InGame => {
                if is_key_down(KeyCode::W)        { paddle1.move_up();   }
                if is_key_down(KeyCode::S)        { paddle1.move_down(); }
                if is_key_down(KeyCode::Up)       { paddle2.move_up();   }
                if is_key_down(KeyCode::Down)     { paddle2.move_down(); }
                let (p1_goal, p2_goal) = ball.update(&paddle1, &paddle2);

                if p1_goal { score1 += 1; ball.active = false; countdown = 0.; state = GameState::Countdown; }
                if p2_goal { score2 += 1; ball.active = false; countdown = 0.; state = GameState::Countdown; }

                if score1 >= WIN_SCORE {
                    winner_msg = format!("{} Wins!", player1_name);
                    winner_name = player1_name.clone();
                    *leaderboard.entry(winner_name.clone()).or_insert(0) += 1;
                    state = GameState::GameOver;
                } else if score2 >= WIN_SCORE {
                    winner_msg = format!("{} Wins!", player2_name);
                    winner_name = player2_name.clone();
                    *leaderboard.entry(winner_name.clone()).or_insert(0) += 1;
                    state = GameState::GameOver;
                }

                draw_animated_net();
                ball.draw();
                paddle1.draw();
                paddle2.draw();
                draw_score_and_net(score1, score2, full_tab, &player1_name, &player2_name);
            }
            GameState::GameOver => {
                draw_text(&winner_msg, current_width / 2. - 150., current_height / 2. - 120., 56., GREEN);
                draw_text(
                    &format!("Winner: {}", winner_name),
                    current_width / 2. - 150.,
                    current_height / 2. - 60.,
                    40.,
                    YELLOW,
                );
                draw_text(
                    "Leaderboard - Top Winners",
                    current_width / 2. - 170.,
                    current_height / 2. - 10.,
                    35.,
                    WHITE,
                );
                let mut lb: Vec<_> = leaderboard.iter().collect();
                lb.sort_by(|a, b| b.1.cmp(a.1));
                for (i, (name, wins)) in lb.iter().take(5).enumerate() {
                    let y_pos = current_height / 2. + 30. + i as f32 * 30.;
                    draw_text(
                        &format!("{}. {} - {} wins", i + 1, name, wins),
                        current_width / 2. - 170.,
                        y_pos,
                        28.,
                        WHITE,
                    );
                }
                draw_text(
                    "Press [Enter] to play again",
                    current_width / 2. - 180.,
                    current_height - 80.,
                    28.,
                    GRAY,
                );
                draw_text(
                    "Press [Esc] to Quit",
                    current_width / 2. - 140.,
                    current_height - 40.,
                    28.,
                    GRAY,
                );
                if is_key_pressed(KeyCode::Enter) {
                    player1_name.clear();
                    player2_name.clear();
                    input_name.clear();
                    score1 = 0;
                    score2 = 0;
                    ball.reset();
                    winner_msg.clear();
                    winner_name.clear();
                    state = GameState::EnterName1;
                }
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
            }
        }

        let mode_txt = if full_tab {
            "Mode: Full Tab (Press Tab to Minimize)"
        } else {
            "Mode: Min Tab (Press Tab to Maximize)"
        };
        let mode_dim = measure_text(mode_txt, None, 22, 1.);
        draw_text(
            mode_txt,
            current_width - mode_dim.width - 20.,
            current_height - 40.,
            22.,
            GRAY,
        );
        if is_key_pressed(KeyCode::Escape) && state != GameState::GameOver {
            break;
        }
        next_frame().await;
    }
}

fn handle_name_input(input_name: &mut String) {
    while let Some(c) = get_char_pressed() {
        if (c.is_alphanumeric() || c == ' ') && input_name.len() < 20 {
            input_name.push(c);
        }
    }
    if is_key_pressed(KeyCode::Backspace) && !input_name.is_empty() {
        input_name.pop();
    }
}

fn draw_dynamic_gradient_background(time: f32) {
    let top = Color::from_rgba(
        (45.0 + 25.0 * (time * 0.2).sin()) as u8,
        (59.0 + 30.0 * (time * 0.15).cos()) as u8,
        108,
        255,
    );
    let bottom = Color::from_rgba(
        (183.0 + 20.0 * (time * 0.24).cos()) as u8,
        (203.0 + 15.0 * (time * 0.19).sin()) as u8,
        213,
        255,
    );
    for i in 0..screen_height() as i32 {
        let t = i as f32 / screen_height();
        let col = Color {
            r: top.r + (bottom.r - top.r) * t,
            g: top.g + (bottom.g - top.g) * t,
            b: top.b + (bottom.b - top.b) * t,
            a: 1.0,
        };
        draw_line(0., i as f32, screen_width(), i as f32, 1., col);
    }
}

fn draw_neon_rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    for i in (1..12).rev() {
        let alpha = 0.04 * i as f32;
        let bright = Color::new(color.r, color.g, color.b, alpha);
        draw_rectangle(
            x - i as f32,
            y - i as f32,
            w + i as f32 * 2.,
            h + i as f32 * 2.,
            bright,
        );
    }
    draw_rectangle(x, y, w, h, color);
}

fn draw_neon_ball(ball: &Ball) {
    let base = ball.rect.x + ball.rect.w / 2.;
    let base_y = ball.rect.y + ball.rect.h / 2.;
    let pulse = 1. + 0.06 * (get_time() * 3.0).sin() as f32;
    for i in (1..9).rev() {
        draw_circle(
            base, base_y,
            (BALL_SIZE / 2.) * (pulse + i as f32 * 0.10),
            Color::new(1.0, 1.0, 0.54, 0.04 * i as f32 + 0.02),
        );
    }
    draw_circle(base, base_y, BALL_SIZE / 2., ball.color);
}

fn draw_animated_net() {
    let time = get_time() as f32;
    let w = screen_width() / 2. - 4.;
    for y in (0..screen_height() as i32).step_by(28) {
        let alpha = 80. + 60. * (time * 1.4 + y as f32 * 0.10).sin();
        draw_rectangle(
            w,
            y as f32,
            8.,
            18.,
            Color::from_rgba(180, 180, 180, alpha as u8),
        );
    }
}

fn draw_animated_menu_title() {
    let pulse = 1.06 + 0.04 * (get_time() as f32 * 2.2).sin();
    draw_text("PONG",
        screen_width()/2.-130.*pulse, screen_height()/2.-150.*pulse,
        110. * pulse, Color::from_rgba(250, 240, 40, 245));
}

fn draw_score_and_net(s1: i32, s2: i32, full: bool, name1: &str, name2: &str) {
    draw_animated_net();
    if full {
        let text = format!("{}: {}      {}: {}", name1, s1, name2, s2);
        let dm = measure_text(&text, None, 48, 1.);
        let x = screen_width() / 2. - dm.width / 2.;
        let y = 70.;
        draw_rectangle(x - 10., y - 40., dm.width + 20., 60., Color::from_rgba(0, 0, 0, 120));
        draw_text(&text, x, y, 48., WHITE);
    } else {
        let text = format!("{}   :   {}", s1, s2);
        let dm = measure_text(&text, None, 54, 1.);
        let x = screen_width() / 2. - dm.width / 2.;
        let y = 70.;
        draw_text(&text, x, y, 54., WHITE);
    }
}
