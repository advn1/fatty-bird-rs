use ::rand::{thread_rng, Rng};
use macroquad::prelude::*;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::Path;

const GRAVITY: f32 = 2000.0;
const JUMP_STRENGTH: f32 = 700.0;
const FREE_SPACE: f32 = 150.0;

#[derive(PartialEq)]
enum GameState {
    Playing,
    GameOver,
    Paused,
    StartMenu,
}

#[derive(Debug)]
struct EventQueue {
    events: VecDeque<Event>,
}
#[expect(unused)]
impl EventQueue {
    fn add(&mut self, event: Event) {
        self.events.push_front(event);
    }
    fn delete(&mut self) -> Option<Event> {
        self.events.pop_front()
    }
    fn is_empty(&self) -> bool {
        return self.events.is_empty();
    }
}

#[derive(Debug)]
enum Event {
    PushCoin,
    PushPipe,
    DeleteCoin,
    DeletePipe,
    UpdateScore,
}

struct Stats {
    score: u32,
    high_score: u32,
    coins_amount: u32,
}

impl Stats {
    fn draw(&self) {
        draw_text(
            format!("SCORE: {}", self.score).as_str(),
            10.0,
            25.0,
            40.0,
            WHITE,
        );
        draw_text(
            format!("HIGH SCORE: {}", self.high_score).as_str(),
            10.0,
            50.0,
            40.0,
            WHITE,
        );
        draw_text(
            format!("COINS: {}", self.coins_amount).as_str(),
            screen_width() - 200.0,
            100.0,
            40.0,
            WHITE,
        );
    }

    fn update(&mut self) {
        self.score += 1;
        if self.score > self.high_score {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("high_score.txt")
                .unwrap();

            let mut writer: BufWriter<&mut File> = BufWriter::new(&mut file);

            match writer.write_all(format!("{}", self.score).as_bytes()) {
                Err(why) => panic!("couldn't write to high_score.txt: {}", why),
                Ok(_) => println!("successfully wrote"),
            }
            self.high_score = self.score
        }
    }

    fn reset(&mut self) {
        self.score = 0;
    }
}

#[derive(Debug)]
struct Pipe {
    x: f32,
    y: f32,
    height: f32,
}
#[derive(Debug)]
struct PipeGroup {
    pipe_up: Pipe,
    pipe_down: Pipe,
}

impl PipeGroup {
    fn from(up_height: f32, down_height: f32) -> PipeGroup {
        let pipe_up = Pipe {
            x: screen_width(),
            y: 0.0,
            height: up_height,
        };

        let pipe_down = Pipe {
            x: screen_width(),
            y: down_height,
            height: screen_height(),
        };

        PipeGroup { pipe_up, pipe_down }
    }
}

impl Default for PipeGroup {
    fn default() -> Self {
        let mut rng = thread_rng();
        let height = rng.gen_range(100.0..300.0);

        let pipe_up = Pipe {
            x: screen_width(),
            y: 0.0,
            height: height,
        };
        let pipe_down = Pipe {
            x: screen_width(),
            y: height + FREE_SPACE,
            height: screen_height(),
        };

        PipeGroup { pipe_up, pipe_down }
    }
}

struct Pipes {
    pipe_groups: Vec<PipeGroup>,
}

impl From<Vec<PipeGroup>> for Pipes {
    fn from(pipes: Vec<PipeGroup>) -> Self {
        Pipes { pipe_groups: pipes }
    }
}

impl Pipes {
    fn draw_pipes(&self, pipe: &Texture2D) {
        self.pipe_groups.iter().for_each(|pipes| {
            let pipe_up_rect = Rect::new(0.0, 0.0, 100.0, pipes.pipe_up.height);
            let pipe_down_rect = Rect::new(0.0, 0.0, 100.0, pipes.pipe_down.height);

            draw_pipe(&pipe, pipes.pipe_up.x, pipes.pipe_up.y, pipe_up_rect, 3.14);

            draw_pipe(
                &pipe,
                pipes.pipe_down.x,
                pipes.pipe_down.y,
                pipe_down_rect,
                0.0,
            );
        });
    }

    fn update(&mut self, event_queue: &mut EventQueue) {
        //move pipes
        self.pipe_groups.iter_mut().for_each(|pipes| {
            pipes.pipe_up.x -= 300.0 * get_frame_time();
            pipes.pipe_down.x -= 300.0 * get_frame_time();
        });
        //move coin
        // coins_array.iter_mut().for_each(|coin| {
        //     coin.x -= 300.0 * get_frame_time();
        // });

        //delete pipes
        if self.pipe_groups.first().unwrap().pipe_up.x <= -150.0 {
            println!("pop pipe");
            event_queue.add(Event::DeletePipe);
        }

        //add new pipes AND add coin
        if self.pipe_groups.get(0).unwrap().pipe_up.x <= 100.0 && self.pipe_groups.len() == 1 {
            event_queue.add(Event::PushPipe);
            event_queue.add(Event::PushCoin);
            event_queue.add(Event::UpdateScore);

            println!("push");
        }
    }

    fn push_pipes(&mut self) {
        let mut rng = thread_rng();
        let up_rand_height = rng.gen_range(100.0..300.0);
        let down_rand_height = up_rand_height + FREE_SPACE;

        //push pipe
        self.pipe_groups
            .push(PipeGroup::from(up_rand_height, down_rand_height));
    }

    fn delete_pipes(&mut self) {
        self.pipe_groups.remove(0);
    }
}

struct Bird {
    x: f32,
    y: f32,
    velocity: f32,
    space_pressed: bool,
}

impl Bird {
    fn draw(&self, texture: &Texture2D, sprite_rect: Rect) {
        draw_texture_ex(
            &texture,
            self.x,
            self.y,
            WHITE,
            DrawTextureParams {
                source: Some(sprite_rect),
                ..Default::default()
            },
        );
    }

    fn update(&mut self, pipe: &PipeGroup, game_state: &mut GameState) {
        //collide check
        if (self.y <= pipe.pipe_up.height - 5.0 || self.y >= pipe.pipe_down.y - 15.0)
            && pipe.pipe_down.x <= 230.0
            && pipe.pipe_down.x >= 230.0 - 130.0
        {
            *game_state = GameState::GameOver;
        }

        // //check out of bounds jump
        if self.y >= screen_height() + 10.0 || self.y <= -10.0 {
            self.velocity = 0.0;
            self.y = screen_width() / 2.0;
            *game_state = GameState::GameOver;
        }

        // // Space key check
        if is_key_down(KeyCode::Space) && !self.space_pressed {
            self.velocity = -JUMP_STRENGTH;
            self.space_pressed = true;
        }

        if is_key_released(KeyCode::Space) && self.space_pressed {
            self.space_pressed = false;
        }

        // Update bird position and velocity
        self.velocity += GRAVITY * get_frame_time();
        self.y += self.velocity * get_frame_time();
    }
}

impl Default for Bird {
    fn default() -> Self {
        Bird {
            x: 200.0,
            y: screen_height() / 2.0,
            velocity: 0.0,
            space_pressed: false,
        }
    }
}
#[derive(Debug)]
struct Coin {
    x: f32,
    y: f32,
}

impl Coin {
    fn from(space_between: f32) -> Coin {
        let rand_height = rand::gen_range(70.0, screen_height() - 70.0);
        println!("SPACE BETWEEN: {}", space_between);
        Coin {
            x: screen_width() + -space_between - rand::gen_range(400.0, -space_between - 200.0),
            y: rand_height,
        }
    }
}

struct Coins {
    coins: Vec<Coin>,
    is_not_touched_coin: bool,
}

impl Default for Coins {
    fn default() -> Self {
        Coins {
            coins: Vec::from([Coin::from(200.0)]),
            is_not_touched_coin: true,
        }
    }
}

impl Coins {
    fn draw_coins(&self, texture: &Texture2D) {
        self.coins.iter().for_each(|coin| {
            draw_texture(texture, coin.x, coin.y, WHITE);
        });
    }

    fn update(&mut self) {
        self.coins.iter_mut().for_each(|coin| {
            coin.x -= 300.0 * get_frame_time();
        });
    }

    fn push_coins(&mut self, space_between_pipe_groups: f32) {
        self.coins.push(Coin::from(space_between_pipe_groups));
        self.is_not_touched_coin = true;
    }

    fn delete_coin(&mut self) {
        self.coins.remove(0);
    }

    fn is_collide(
        &mut self,
        bird_x: f32,
        bird_y: f32,
        event_queue: &mut EventQueue,
        stats: &mut Stats,
    ) {
        if self.coins.len() < 1 {
            return;
        }
        if !self.is_not_touched_coin {
            return;
        }

        let coin = self.coins.first().unwrap();

        if coin.x < 0.0 - 40.0 {
            event_queue.add(Event::DeleteCoin);
        }
        // println!("{:?}", coins_array);
        let coin_rect = Rect::new(coin.x - 10.0, coin.y - 10.0, 60.0, 60.0);
        if coin_rect.contains(Vec2::from([bird_x, bird_y])) {
            stats.coins_amount += 1;
            self.is_not_touched_coin = false;
            event_queue.add(Event::DeleteCoin);
        }
    }
}

fn draw_start_menu() {
    draw_text(
        "Fatty Bird",
        screen_width() / 2.0 - 130.0,
        screen_height() / 2.0 - 30.0,
        60.0,
        YELLOW,
    );
    draw_rectangle(
        screen_width() / 2.0 - 100.0,
        screen_height() / 2.0 + 61.5,
        200.0,
        54.0,
        ORANGE,
    );
    draw_text(
        "PLAY",
        screen_width() / 2.0 - 50.0,
        screen_height() / 2.0 + 100.0,
        50.0,
        WHITE,
    );
}

fn draw_game_over() {
    draw_text(
        "GAY OVER",
        screen_width() / 2.0 - 90.0,
        screen_height() / 2.0 - 50.0,
        50.0,
        WHITE,
    );
    draw_rectangle(
        screen_width() / 2.0 - 100.0,
        screen_height() / 2.0 + 70.0,
        200.0,
        54.0,
        RED,
    );
    draw_text(
        "Play Again",
        screen_width() / 2.0 - 72.5,
        screen_height() / 2.0 + 100.0,
        35.0,
        BLACK,
    );
}

fn draw_pipe(
    texture: &Texture2D,
    pipe_pos_x: f32,
    pipe_pos_y: f32,
    pipe_rect: Rect,
    rotation: f32,
) {
    draw_texture_ex(
        &texture,
        pipe_pos_x,
        pipe_pos_y,
        WHITE,
        DrawTextureParams {
            source: Some(pipe_rect),
            rotation: rotation,
            ..Default::default()
        },
    );
}

#[macroquad::main(conf)]
async fn main() {
    let mut game_state = GameState::StartMenu;

    //background texture
    let bg = load_texture("./assets/bg4.png")
        .await
        .unwrap();
    bg.set_filter(FilterMode::Nearest);

    //pipe texture AND pipe vector
    let pipe = load_texture("./assets/mario_pipe_cut.png")
        .await
        .unwrap();
    pipe.set_filter(FilterMode::Nearest);
    let mut pipes: Pipes = Pipes::from(Vec::from([Default::default()]));

    //bird texture AND bird rect
    let bird = load_texture("./assets/pigeon.png")
        .await
        .unwrap();
    bird.set_filter(FilterMode::Nearest);

    let frame_width = bird.width() / 7.0;
    let frame_height = bird.height();
    let mut current_frame = 0;
    let frame_time = 0.1;
    let mut timer = 0.0;

    let mut starter_bird = Bird::default();
    let mut bird_rect = Rect::new(
        frame_width * current_frame as f32,
        0.0,
        frame_width,
        frame_height,
    );

    //pause texture AND pause rect
    let pause = load_texture("./assets/pause_icon.png")
        .await
        .unwrap();
    pause.set_filter(FilterMode::Nearest);
    let pause_rect = Rect::new(screen_width() - 75.0, 25.0, 50.0, 50.0);

    //coin texture AND coin vector
    let coin = load_texture("./assets/coin2.png")
        .await
        .unwrap();
    coin.set_filter(FilterMode::Nearest);
    let mut coins: Coins = Coins::default();

    //STATS
    let high_score = get_high_score("high_score.txt");

    let mut stats = Stats {
        score: 0,
        high_score,
        coins_amount: 0,
    };

    //left_bg_texture part
    let mut left_bg_texture: f32 = 0.0;
    println!("{}", bg.width());

    let mut event_queue = EventQueue {
        events: VecDeque::new(),
    };
    loop {
        //check click on pause
        if check_btn_click(&pause_rect) && game_state != GameState::StartMenu {
            game_state = if game_state == GameState::Paused {
                GameState::Playing
            } else {
                GameState::Paused
            }
        }

        //check out of bounds left_bg_texture
        if -left_bg_texture > screen_width() {
            left_bg_texture = 0.0;
        }

        //dynamic play again and play button rects
        let play_again_button_rect = Rect::new(
            screen_width() / 2.0 - 100.0,
            screen_height() / 2.0 + 70.0,
            200.0,
            54.0,
        );

        let play_button_rect = Rect::new(
            screen_width() / 2.0 - 100.0,
            screen_height() / 2.0 + 61.5,
            200.0,
            54.0,
        );

        clear_background(BLUE);

        //draw left_texture_bg
        draw_texture_ex(
            &bg,
            left_bg_texture,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::from([screen_width(), screen_height() + 20.0])),
                source: Some(Rect::new(0.0, 0.0, bg.width(), bg.height())),

                ..Default::default()
            },
        );
        //draw right_texture_bg
        draw_texture_ex(
            &bg,
            screen_width() - left_bg_texture.abs(),
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::from([screen_width(), screen_height() + 20.0])),
                source: Some(Rect::new(0.0, 0.0, bg.width(), bg.height())),
                ..Default::default()
            },
        );

        pipes.draw_pipes(&pipe);
        coins.draw_coins(&coin);

        //draw pause
        draw_texture(&pause, screen_width() - 75.0, 25.0, WHITE);
        stats.draw();

        while let Some(event) = event_queue.events.pop_back() {
            match event {
                Event::PushPipe => {
                    pipes.push_pipes();
                }
                Event::PushCoin => {
                    let space_between_pipe_groups = pipes.pipe_groups.first().unwrap().pipe_up.x
                        - pipes.pipe_groups.last().unwrap().pipe_up.x;

                    coins.push_coins(space_between_pipe_groups);
                }
                Event::DeleteCoin => coins.delete_coin(),
                Event::DeletePipe => {
                    pipes.delete_pipes();
                }
                Event::UpdateScore => {
                    stats.update();
                }
            }
        }
        match game_state {
            GameState::StartMenu => {
                left_bg_texture -= 150.0 * get_frame_time();
                draw_start_menu();
                if check_btn_click(&play_button_rect) {
                    game_state = GameState::Playing;
                    starter_bird = Bird::default();
                    pipes = Pipes::from(Vec::from([Default::default()]));
                    stats.reset();
                }
            }
            GameState::Playing => {
                left_bg_texture -= 150.0 * get_frame_time();
                timer += get_frame_time();

                if timer >= frame_time {
                    current_frame = (current_frame + 1) % 7;
                    timer = 0.0;
                }

                bird_rect = Rect::new(
                    frame_width * current_frame as f32,
                    0.0,
                    frame_width,
                    frame_height,
                );
                pipes.update(&mut event_queue);
                coins.update();

                starter_bird.draw(&bird, bird_rect);
                starter_bird.update(pipes.pipe_groups.first().unwrap(), &mut game_state);

                coins.is_collide(starter_bird.x, starter_bird.y, &mut event_queue, &mut stats);
            }
            GameState::GameOver => {
                draw_game_over();
                //check play again click
                if check_btn_click(&play_again_button_rect) {
                    game_state = GameState::Playing;
                    starter_bird = Bird::default();
                    pipes = Pipes::from(Vec::from([Default::default()]));
                    stats.reset();
                    if coins.coins.len() > 0 {
                        coins.delete_coin();
                    }
                }
            }
            GameState::Paused => {
                starter_bird.draw(&bird, bird_rect);
                draw_text(
                    "PAUSED",
                    screen_width() / 2.0 - 50.0,
                    screen_height() / 2.0,
                    50.0,
                    GRAY,
                );
            }
        }
        for touch in touches() {
            let (fill_color, size) = match touch.phase {
                TouchPhase::Started => (GREEN, 80.0),
                TouchPhase::Stationary => (WHITE, 60.0),
                TouchPhase::Moved => (YELLOW, 60.0),
                TouchPhase::Ended => (BLUE, 80.0),
                TouchPhase::Cancelled => (BLACK, 80.0),
            };
            draw_circle(touch.position.x, touch.position.y, size, fill_color);
        }

        draw_text("touch the screen!", 20.0, 20.0, 20.0, DARKGRAY);
        next_frame().await
    }
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("Fatty Bird"),
        window_height: 600,
        window_width: 1200,
        window_resizable: true,
        ..Default::default()
    }
}

fn check_btn_click(rect: &Rect) -> bool {
    if is_mouse_button_released(MouseButton::Left) {
        if rect.contains(Vec2::from(mouse_position())) {
            return true;
        }
    }
    false
}

fn get_high_score<P>(filename: P) -> u32
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let contents = contents.trim();

    let high_score: u32 = contents.parse().unwrap_or(0);

    return high_score;
}
