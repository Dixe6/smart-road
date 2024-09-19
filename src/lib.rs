extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::image::{LoadTexture};
use sdl2::ttf::Font;
use sdl2::render::TextureQuery;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;

use rand::Rng;
use std::collections::VecDeque;
use std::thread::spawn;
use std::time::{Duration, Instant};

pub mod vehicle;
pub mod sector;
use vehicle::*;
use sector::*;


// Constants for the simulation window, road dimensions, vehicle size, and safe distance between vehicles
const SCREEN_WIDTH: u32 = 1000;
const SCREEN_HEIGHT: u32 = 1000;
const ROAD_WIDTH: u32 = 400;
const ROAD_NUMBER: u32 = 3; 

const VEHICLE_WIDTH: u32 = 25;
const VEHICLE_HEIGHT: u32 = 50;
const SAFE_DISTANCE: u32 = 20; 
const NUMBER_AV:usize = 24;

// Enum to represent the direction of vehicle movement
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Left,
    Right,
    Forward,
}
// Enum to represent the direction of vehicle movement
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Velocity {
    Stop,
    Slow,
    Normal,
    Fast,
}
// Main simulation struct, which contains SDL canvas, event pump, a queue of vehicles, traffic lights, and random number generator
pub struct Simulation {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    vehicles: VecDeque<Vehicle>,
    sector: Sector,
    stats: VecDeque<Stats>,
    rng: rand::rngs::ThreadRng,
    // to limit the spawn of the vehicles
    last_spawn_time: Instant,
    cooldown: Duration,
    refresh_time: Duration,
    speed_boost: u32,
    visibility: (bool,bool),
    paused: bool,
    stat_showing: bool,
    spawn_loop: (Instant,Duration),
}
impl Simulation {
    // Initialize a new simulation
    pub fn new() -> Self {
        // Initialize SDL context and video subsystem
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        // Create a centered window with specified width and height
        let window = video_subsystem
            .window("Traffic Simulation", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        // Create a canvas for rendering
        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let rng = rand::thread_rng();

        let map = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let sector = Sector::new(map);

        // Return an instance of the Simulation struct
        Simulation {
            canvas,
            event_pump,
            vehicles: VecDeque::new(),
            sector,
            stats: VecDeque::new(),
            rng,
            last_spawn_time: Instant::now(),
            cooldown: Duration::from_millis(300),
            refresh_time: Duration::from_millis(10),
            speed_boost: 0,
            visibility: (false,false),
            paused: false,
            stat_showing: false,
            spawn_loop: (Instant::now(),Duration::from_secs_f32(0.0)),
        }
    }

    // Main simulation loop that handles events, updates the state, and renders the simulation
    pub fn run(&mut self) {
        let mut running = true;
        while running {
            self.handle_events(&mut running);  // This will need to be async as well
        
            if !self.stat_showing {
                if !self.paused{
                    self.update();
                }
                self.render();
                match self.speed_boost{
                    0 => {::std::thread::sleep(self.refresh_time)}      // normal refresh
                    1 => {::std::thread::sleep(self.refresh_time/2)}    // boosted refresh
                    2 => {::std::thread::sleep(self.refresh_time*3)}    // Slow refresh
                    _ => self.speed_boost -= 1,
                }
            }else{
                self.render_stat();
            }
        }
    }
     // Update the state of vehicles and traffic lights
     fn update(&mut self) {
        if self.spawn_loop.1.as_secs_f32()>0.0
        && self.spawn_loop.0 + self.cooldown < Instant::now() 
        {
            let directions = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ];
            let random_direction = directions[self.rng.gen_range(0..4)];
            // self.spawn_vehicle(random_direction);
            let mut vehicle = Vehicle::new(random_direction);
            let is_overlapping = vehicle.is_overlapping(self.sector.clone(),self.vehicles.clone());
                if is_overlapping == 0 
                && self.vehicles.len()< NUMBER_AV{
                    self.vehicles.push_back(vehicle);
                }
            self.last_spawn_time = Instant::now(); // Update the last spawn time
            self.spawn_loop.1 -= self.cooldown;
            self.spawn_loop.0 = Instant::now();
        }
        for i in 0..self.vehicles.len() {
            let vehicles = self.vehicles.clone();
            // turn if vehicle need
            let velocity =self.vehicles[i].turn(self.sector.clone());
            // move forward if vehicle can
            self.vehicles[i].forward(vehicles.clone(),self.sector.clone(),velocity);
        }
        // Retain only the vehicles that have not yet arrived
        self.vehicles.retain(|vehicle| {
            let arrived = 
            !vehicle.body.intersection(self.sector.map).is_some()
            && !vehicle.hitbox.urgency_stop.intersection(self.sector.map).is_some();
            
            if arrived {
                let mut t_av = vehicle.clone();
                t_av.arrival();
                self.stats.push_back(t_av.stats);
            }
            
            !arrived
        });
        // println!("{}",self.vehicles.len());
    }

    // Handle input events such as quitting, spawning vehicles in different directions, and random vehicle spawning
    fn handle_events(&mut self, running: &mut bool) {
        let events: Vec<Event> = self.event_pump.poll_iter().collect();
        let now = Instant::now();
        if now.duration_since(self.last_spawn_time) < self.cooldown {
            return; // If cooldown period has not passed, return early
        }

        for event in events {
            match event {
                Event::Quit { 
                    .. 
                } => {
                    *running = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    if self.stat_showing{
                        *running = false;
                    }else{
                        self.stat_showing = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // self.spawn_vehicle(Direction::North);
                    let mut vehicle = Vehicle::new(Direction::North);
                    let is_overlapping = vehicle.is_overlapping(self.sector.clone(),self.vehicles.clone());
                    if is_overlapping == 0 
                    && self.vehicles.len()< NUMBER_AV{
                        self.vehicles.push_back(vehicle);
                    }
                    self.last_spawn_time = now; // Update the last spawn time
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // self.spawn_vehicle(Direction::South);
                    let mut vehicle = Vehicle::new(Direction::South);
                    let is_overlapping = vehicle.is_overlapping(self.sector.clone(),self.vehicles.clone());
                    if  is_overlapping == 0 
                    && self.vehicles.len()< NUMBER_AV{
                        self.vehicles.push_back(vehicle);
                    }
                    self.last_spawn_time = now; // Update the last spawn time
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // self.spawn_vehicle(Direction::East);
                    let mut vehicle = Vehicle::new(Direction::East);
                    let is_overlapping = vehicle.is_overlapping(self.sector.clone(),self.vehicles.clone());
                    if is_overlapping == 0 
                    && self.vehicles.len()< NUMBER_AV{
                        self.vehicles.push_back(vehicle);
                    }
                    self.last_spawn_time = now; // Update the last spawn time
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // self.spawn_vehicle(Direction::West);
                    let mut vehicle = Vehicle::new(Direction::West);
                    let is_overlapping = vehicle.is_overlapping(self.sector.clone(),self.vehicles.clone());
                    if is_overlapping == 0 
                    && self.vehicles.len()< NUMBER_AV{
                        self.vehicles.push_back(vehicle);
                    }
                    self.last_spawn_time = now; // Update the last spawn time
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    let directions = [
                        Direction::North,
                        Direction::South,
                        Direction::East,
                        Direction::West,
                    ];
                    let random_direction = directions[self.rng.gen_range(0..4)];
                    // self.spawn_vehicle(random_direction);
                    let mut vehicle = Vehicle::new(random_direction);
                    let is_overlapping = vehicle.is_overlapping(self.sector.clone(),self.vehicles.clone());
                    if is_overlapping == 0 
                    && self.vehicles.len()< NUMBER_AV{
                        self.vehicles.push_back(vehicle);
                    }
                    self.last_spawn_time = now; // Update the last spawn time
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.spawn_loop = (Instant::now(),Duration::from_secs_f32(60.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.speed_boost = 1;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.speed_boost = 0;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::I),
                    ..
                } => {
                    self.visibility.1 = !self.visibility.1;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::O),
                    ..
                } => {
                    self.visibility.0 = !self.visibility.0;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::T),
                    ..
                } => {
                    if self.speed_boost != 2 {
                        self.speed_boost = 2;
                    }else {
                        self.speed_boost = 0;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    self.paused = !self.paused;
                }
                _ => {}
            }
        }
    }

    // Render the simulation, including roads, and vehicles
    fn render(&mut self) {
        // Draw the road
        draw_road(&mut self.canvas);
        if self.visibility.1{
            // Render zone hitbox
            self.canvas.set_draw_color(Color::RGBA(75, 75, 0,85));
            self.canvas.fill_rect(self.sector.entry_intersect).unwrap();
            self.canvas.set_draw_color(Color::RGBA(125, 42, 42,85));
            self.canvas.fill_rect(self.sector.in_intersect).unwrap();
            // Left turn point
            self.canvas.set_draw_color(Color::RGB(110, 0, 0));
            self.canvas.fill_rect(self.sector.turn_north.0).unwrap();
            self.canvas.fill_rect(self.sector.turn_north.1).unwrap();
            self.canvas.set_draw_color(Color::RGB(0, 110, 0));
            self.canvas.fill_rect(self.sector.turn_east.1).unwrap();
            self.canvas.fill_rect(self.sector.turn_east.0).unwrap();
            self.canvas.set_draw_color(Color::RGB(0, 0, 110));
            self.canvas.fill_rect(self.sector.turn_south.0).unwrap();
            self.canvas.fill_rect(self.sector.turn_south.1).unwrap();
            self.canvas.set_draw_color(Color::RGB(110, 110, 0));
            self.canvas.fill_rect(self.sector.turn_west.0).unwrap();
            self.canvas.fill_rect(self.sector.turn_west.1).unwrap();
            // Right turn point
        }
        // Set the position and size of the image on the screen
        let width = VEHICLE_WIDTH;      // Width of the image
        let height = VEHICLE_HEIGHT;     // Height of the image

        // Render vehicles
        for vehicle in &self.vehicles {
            if self.visibility.0{
                // Render the body and hitbox
                self.canvas.set_draw_color(Color::RGBA(225, 225, 90,125));
                self.canvas.fill_rect(vehicle.hitbox.slowdown_2).unwrap();
                self.canvas.set_draw_color(Color::RGBA(90, 90, 255,125));
                self.canvas.fill_rect(vehicle.hitbox.left).unwrap();
                self.canvas.fill_rect(vehicle.hitbox.right).unwrap();
                self.canvas.set_draw_color(Color::RGBA(125, 125, 0,125));
                self.canvas.fill_rect(vehicle.hitbox.slowdown_1).unwrap();
                self.canvas.set_draw_color(Color::RGBA(125, 0, 0,125));
                self.canvas.fill_rect(vehicle.hitbox.closer).unwrap();
                self.canvas.fill_rect(vehicle.hitbox.urgency_stop).unwrap();
                self.canvas.set_draw_color(Color::RGBA(0, 125, 0,125));
                self.canvas.fill_rect(vehicle.body).unwrap();
            }

            let rotation_angle = match vehicle.route {
                Direction::North => 180.0,
                Direction::East => 270.0,
                Direction::South => 0.0,
                Direction::West => 90.0,
                _ => todo!(),
            };
            let position_x = vehicle.position.x as i32 - (VEHICLE_WIDTH as i32)/2;
            let position_y = vehicle.position.y as i32 - (VEHICLE_HEIGHT as i32)/2;
            let destination_rect = Rect::new(position_x, position_y, width, height);

            let texture_creator = self.canvas.texture_creator();
    
            match texture_creator.load_texture(&vehicle.texture) {
                Ok(texture) => {
                    self.canvas.copy_ex(
                        &texture,
                        None,
                        Some(destination_rect),
                        rotation_angle,
                        None,
                        false,
                        false,
                    )
                    .unwrap();
                }
                Err(e) => eprintln!("Failed to load texture: {}", e),
            };
        }
        self.canvas.present(); // Present the updated canvas to the screen
    }
    // The render_stat function
    fn render_stat(&mut self) {
        // Initialize TTF context if not done already
        let ttf_context = sdl2::ttf::init().expect("Failed to initialize TTF context");

        let font_path = "font/RubikGlitch-Regular.ttf";
        let font = ttf_context.load_font(font_path, 45).expect("Failed to load font");

        // Set the background color
        self.canvas.set_draw_color(Color::RGB(25, 25, 25));
        self.canvas.clear();

        // Calculate the stats
        let total_av = self.stats.len();
        let max_velocity = self.stats.iter().map(|s| s.velocity).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_velocity = self.stats.iter().map(|s| s.velocity).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_time = self.stats.iter().map(|s| s.time).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_time = self.stats.iter().map(|s| s.time).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let close_call:u32 = self.stats.iter().map(|s| s.close_call).sum();
        let colision:u32 = self.stats.iter().map(|s| s.colision).sum();


        // Define the text for display
        let text_data = [
            format!("Total Entries: {}", total_av),
            format!("Max Velocity: {:.2}", max_velocity),
            format!("Min Velocity: {:.2}", min_velocity),
            format!("Max Time: {:.2}", max_time.as_secs_f64()),
            format!("Min Time: {:.2}", min_time.as_secs_f64()),
            format!("Colision: {}", colision),
            format!("Close Call: {}", close_call)
        ];

        // Set the text color
        let text_color = Color::RGB(225, 225, 255);

        // Create a surface for each text and render it to the canvas
        for (i, text) in text_data.iter().enumerate() {
            let surface = font
                .render(text)
                .blended(text_color)
                .expect("Could not create surface");
            let texture_creator = self.canvas.texture_creator();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .expect("Could not create texture");

            // Get the texture size
            let TextureQuery { width, height, .. } = texture.query();

            // Set the position for the text rendering
            let target = Rect::from_center(
                Point::new(
                    SCREEN_WIDTH as i32/2,
                    (SCREEN_HEIGHT-(text_data.len()as u32*height))as i32/2 + (height*i as u32)as i32
                    ),
                  width, 
                  height
                );

            // Copy the texture to the canvas
            self.canvas.copy(&texture, None, Some(target)).expect("Render failed");
        }

        // Present the updated canvas to the screen
        self.canvas.present();
    }
}
fn draw_road(canvas: &mut Canvas<Window>) {
    let (screen_width, screen_height) = (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
    let half_road_width = ROAD_WIDTH as i32 / 2;
    let road_center_x = screen_width / 2;
    let road_center_y = screen_height / 2;
    let road_segment = ROAD_WIDTH / (ROAD_NUMBER * 2) as u32;

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // Clear canvas with background color
    canvas.set_draw_color(Color::RGB(86, 125, 70));
    canvas.clear();

    // Draw vertical and horizontal roads
    canvas.set_draw_color(Color::RGB(45, 45, 45));
    canvas
        .fill_rect(Rect::new(road_center_x - half_road_width, 0, ROAD_WIDTH, SCREEN_HEIGHT))
        .unwrap();
    canvas
        .fill_rect(Rect::new(0, road_center_y - half_road_width, SCREEN_WIDTH, ROAD_WIDTH))
        .unwrap();

    // Draw lines on roads
    canvas.set_draw_color(Color::RGB(175, 175, 175));
    for i in 0..=ROAD_NUMBER {
        let offset = (road_segment * i) as i32;
        let line_width = if i != 0 && i < ROAD_NUMBER { 1 } else { 2 };
        let displacement = if i == 0 {0}else{ROAD_WIDTH/(ROAD_NUMBER*2)};
        let road_length = (screen_height as u32 - ROAD_WIDTH) / 2 + displacement;

        // Vertical road lines
        for &x_offset in &[road_center_x - half_road_width + offset, road_center_x + half_road_width - offset] {
            // substract 2 at left part under the half_road_width
            let mut offset = x_offset;
            if line_width == 2 && x_offset < SCREEN_WIDTH as i32/2{
                offset -= 2;
            }else if line_width == 2 && x_offset == SCREEN_WIDTH as i32/2{
                offset -= 1;
            }
            canvas
                .fill_rect(Rect::new(offset, 0, line_width, road_length))
                .unwrap();
            canvas
                .fill_rect(Rect::new(offset, road_center_y + half_road_width - displacement as i32, line_width, road_length))
                .unwrap();
        }

        // Horizontal road lines
        for &y_offset in &[road_center_y - half_road_width + offset, road_center_y + half_road_width - offset] {
            let mut offset = y_offset;
            if line_width == 2 && y_offset < SCREEN_HEIGHT as i32/2{
                offset -= 2;
            }else if line_width == 2 && y_offset == SCREEN_HEIGHT as i32/2{
                offset -= 1;
            }
            canvas
                .fill_rect(Rect::new(0, offset, road_length, line_width))
                .unwrap();
            canvas
                .fill_rect(Rect::new(road_center_x + half_road_width - displacement as i32, offset, road_length, line_width))
                .unwrap();
        }
    }
}
