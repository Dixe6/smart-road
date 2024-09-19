use rand::Rng;
use sdl2::libc::dirent;
use sdl2::rect::{Point, Rect};
use std::collections::VecDeque;
use std::time::{Instant,Duration};

use crate::{ROAD_NUMBER, ROAD_WIDTH, SAFE_DISTANCE, SCREEN_HEIGHT, SCREEN_WIDTH, VEHICLE_HEIGHT, VEHICLE_WIDTH};
use crate::{Direction, Velocity, Sector};

#[derive(Clone, Debug, PartialEq,Copy)]
pub struct Stats{
    pub velocity: u32,  // Calculate after arrival
    pub distance: u32,  // increment on movement
    pub time: Duration, // increments until arrival
    pub close_call:u32,
    pub colision:u32,
}


#[derive(Clone, Debug, PartialEq,Copy)]
pub struct Hitbox{
    pub urgency_stop: Rect, // Front stop hitbox
    pub closer: Rect,
    pub slowdown_1: Rect,   // Hitbox to slow Fast to Normal
    pub slowdown_2: Rect,   // Hitbox to slow Normal to Slow
    pub left: Rect,         // Left hitbox
    pub right: Rect,        // Right hitbox
}
// Struct for vehicles, which includes position, direction of movement, route, and color
#[derive(Clone, Debug, PartialEq)]
pub struct Vehicle {
    // Element
    id: Instant,                    // id store an id, unique for each vehicle
    pub texture: String,            // path to vehicle texture (png)
    pub position:Point,             // x and y position of the center of vehicle
    pub direction: Direction,       // final destination direction
    pub route: Direction,           // actual route
    pub body: Rect,                 // global postion used by vehicle
    // Detection
    pub hitbox:Hitbox,  // all hitbox for obstacles detection
    // Movement
    pub velocity: i32,                  // velocity of vehicle
    pub speed: Velocity,                // actual speed objectiv
    pub last_acceleration: Instant,     // last acceleration
    pub acceleration_delay: Duration,   // acceleration
    // Stats
    pub stats: Stats,           // stats of vehicle
    // Check
    close: (bool,bool),    // limit the the close call

}
const SPEED_V:(u32,u32,u32) = (1,3,5);

impl Vehicle {
    pub fn new(route: Direction) -> Self {

        // Random direction
        let directions = [Direction::Left, Direction::Right, Direction::Forward];
        let random_direction = directions[rand::thread_rng().gen_range(0..directions.len())];

        // Displacement based on random direction
        let displacement = match random_direction {
            Direction::Left => (ROAD_WIDTH / (ROAD_NUMBER * 2)) as i32 * 2,
            Direction::Forward => (ROAD_WIDTH / (ROAD_NUMBER * 2)) as i32,
            Direction::Right | _ => 0,
        };

        // Calculate common values
        let half_vehicle_height = VEHICLE_HEIGHT as i32 / 2;
        // let half_vehicle_width = VEHICLE_WIDTH as i32 / 2;

        // Position
        let position= match route {
            Direction::North => {
                Point::new(
                    ((SCREEN_WIDTH - ROAD_WIDTH) / 2 + ROAD_WIDTH / (ROAD_NUMBER * 2) / 2) as i32 + displacement,
                    -half_vehicle_height,
                    // half_vehicle_height,
                )
            }
            Direction::South => {
                Point::new(
                    ((SCREEN_WIDTH + ROAD_WIDTH) / 2 - ROAD_WIDTH / (ROAD_NUMBER * 2) / 2) as i32 - displacement,
                    SCREEN_HEIGHT as i32 + half_vehicle_height,
                )
            }
            Direction::East => {
                Point::new(
                    SCREEN_WIDTH as i32 + half_vehicle_height,
                    ((SCREEN_HEIGHT - ROAD_WIDTH) / 2 + ROAD_WIDTH / (ROAD_NUMBER * 2) / 2) as i32 + displacement,
                )
            }
            Direction::West => {
                Point::new(
                    -half_vehicle_height,
                    ((SCREEN_HEIGHT + ROAD_WIDTH) / 2 - ROAD_WIDTH / (ROAD_NUMBER * 2) / 2) as i32 - displacement,
                )
            }
            _ => todo!(),
        };

        // calculate body and hitboxes
        let (body,hitbox) = cal_hitboxes(position,route);

        // Set random image path
        let images = ["red", "blue", "green", "yellow", "orange", "black", "white"];
        let image_path = format!("assets/{}.png", images[rand::thread_rng().gen_range(0..images.len())]);

        // Create the vehicle
        Self {
            id : Instant::now(),
            texture: image_path,
            position,
            body,
            hitbox,
            direction: random_direction,
            route,
            velocity: SPEED_V.2 as i32,
            speed: Velocity::Fast,
            acceleration_delay: Duration::from_millis(150),
            last_acceleration: Instant::now(),
            stats: Stats {
                velocity: 0,
                time: Duration::from_secs(0),
                distance: 0,
                close_call: 0,
                colision: 0,
            },
            close: (false,false),
        }
    }
    // ToDo: optimizing reaction detection
    pub fn is_overlapping(&mut self,sector:Sector, vehicles: VecDeque<Vehicle>) -> u8 {
    let mut nbt_av_intersects = 0;
    let mut nb_av_intersects = 0;
    for vehicle in vehicles.clone().iter_mut() {
        if vehicle.body.intersection(sector.in_intersect).is_some()
        // && vehicle.id < self.id
        {
            if vehicle.body.intersection(sector.turn_east.0).is_some()
            || vehicle.body.intersection(sector.turn_west.0).is_some()
            || vehicle.body.intersection(sector.turn_north.0).is_some()
            || vehicle.body.intersection(sector.turn_south.0).is_some()
            || vehicle.direction == Direction::Left{
                nbt_av_intersects += 1;
                nb_av_intersects += 1;
            }else{
                nb_av_intersects += 1;
            }
        }
            // Skip the current vehicle
            if vehicle.id == self.id {
                continue;
            }
            if self.hitbox.closer.intersection(vehicle.body).is_some()
            {
                if !self.close.0{
                    self.stats.close_call += 1;
                    self.close.0 = true;
                }
            }else{
                self.close.0 = false;
            }
        // Check if we have a collision to stop cars who collide
        if self.body.intersection(vehicle.body).is_some(){
            if !self.close.1{
                self.stats.colision += 1;
                self.close.1 = true;
            }
            return 1;
        }else{
            self.close.1 = false;
        }

          // lock the number of vehicle in the intersection
            if self.hitbox.closer.intersection(sector.in_intersect).is_some()
            && !self.body.intersection(sector.in_intersect).is_some()
            && (nb_av_intersects >= 5 || nbt_av_intersects >= 2){
                return 1
            }
       
        // Check collision between the front hitbox and the body rectangle of another vehicle
        if self.hitbox.urgency_stop.intersection(vehicle.body).is_some()
        // Av turn right don't take in charge the other ways
        {
            return 1;
        }
            
            // Check priority based on direction (Left-hand priority)
            if self.hitbox.left.intersection(vehicle.hitbox.urgency_stop).is_some() 
            && !self.body.intersection(vehicle.hitbox.urgency_stop).is_some()
            && vehicle.speed != Velocity::Stop
            && self.route != vehicle.route
            && self.direction != Direction::Right
            && !(
                (self.route == Direction::North && vehicle.route == Direction::South)
                || (self.route == Direction::South && vehicle.route == Direction::North)
                || (self.route == Direction::East && vehicle.route == Direction::West)
                || (self.route == Direction::West && vehicle.route == Direction::East)
            ){
                return 1;
            }
            
            // let right-hand priority if vehicle turns left and doesn't have place to turn
        if self.hitbox.right.intersection(vehicle.body).is_some()
        && (self.direction == Direction::Left || self.direction == Direction::Right)
        && self.body.intersection(sector.in_intersect).is_some()
        && !(
            (self.route == Direction::North && vehicle.route == Direction::South)
            || (self.route == Direction::South && vehicle.route == Direction::North)
            || (self.route == Direction::East && vehicle.route == Direction::West)
            || (self.route == Direction::West && vehicle.route == Direction::East)
        ){
            return 1;
        }

        // Check collision between the front deceleration box and the body rectangle
        if self.hitbox.slowdown_1.intersection(vehicle.body).is_some(){
            return 3;
        }
        // Check collision between the front deceleration box and the body rectangle
        if self.hitbox.slowdown_2.intersection(vehicle.body).is_some(){
            return 2;
        }
    }
    // No collision detected
    0
}

    pub fn turn(&mut self, sector: Sector)-> i32 {
        // Define a map for route changes based on direction and sector centers
        let turn_map = match self.route {
            Direction::North => [
                (self.direction == Direction::Left,sector.turn_north.0.center(),sector.turn_north.0.center().y - (self.position.y + self.velocity), Direction::West),
                (self.direction == Direction::Right,sector.turn_north.1.center(),sector.turn_north.1.center().y - (self.position.y + self.velocity), Direction::East),
            ],
            Direction::South => [
                (self.direction == Direction::Left,sector.turn_south.0.center(),(self.position.y - self.velocity) - sector.turn_south.0.center().y, Direction::East),
                (self.direction == Direction::Right,sector.turn_south.1.center(),(self.position.y - self.velocity )- sector.turn_south.1.center().y, Direction::West),
            ],
            Direction::East => [
                (self.direction == Direction::Left,sector.turn_east.0.center(),(self.position.x - self.velocity )- sector.turn_east.0.center().x, Direction::North),
                (self.direction == Direction::Right,sector.turn_east.1.center(),(self.position.x - self.velocity) - sector.turn_east.1.center().x, Direction::South),
            ],
            Direction::West => [
                (self.direction == Direction::Left,sector.turn_west.0.center(),sector.turn_west.0.center().x - (self.position.x + self.velocity), Direction::South),
                (self.direction == Direction::Right,sector.turn_west.1.center(),sector.turn_west.1.center().x - (self.position.x + self.velocity), Direction::North),
            ],
            _ => return 0,
        };
    
        // Check for turning conditions and update the route
        for &(condition,center,distance, new_route) in &turn_map {
            if condition && distance < 0 && !(self.body.center() == center)  {
                return self.velocity + distance
            } 
            if condition && self.body.center() == center {
                self.route = new_route;
                self.direction = Direction::Forward;
                break;
            }
        }
        return -1
    }
    pub fn forward(&mut self,vehicles:VecDeque<Vehicle>, sector:Sector, turn_velocity:i32){
        // if a car are front of the vehicle
        match self.is_overlapping(sector.clone(),vehicles.clone()){
            1 => self.speed = Velocity::Stop,
            2 => self.speed = Velocity::Slow,
            3 => self.speed = Velocity::Normal,
            _ => {
                if self.body.intersection(sector.entry_intersect).is_some(){
                    self.speed = match self.speed {
                        Velocity::Stop => {
                            if self.direction == Direction::Forward{
                                Velocity::Normal
                            }else{
                                Velocity::Slow
                            }
                        },
                        Velocity::Slow => Velocity::Normal,
                        Velocity::Normal => self.speed,
                        Velocity::Fast => Velocity::Normal,
                    }
                }else{
                    self.speed = match self.speed {
                        Velocity::Stop => Velocity::Slow,
                        Velocity::Slow => Velocity::Normal,
                        Velocity::Normal => Velocity:: Fast,
                        Velocity::Fast => self.speed
                    }
                }
                // Velocity::Slow
            }
        }
        // update actual velocity
        let acctual_speed = match self.speed{
            Velocity::Stop => 0,
            Velocity::Slow => SPEED_V.0,
            Velocity::Normal => SPEED_V.1,
            Velocity::Fast => SPEED_V.2,
        };
        // Velocity:: with acceleration
        if acctual_speed > self.velocity as u32{
            if Instant::now().duration_since(self.last_acceleration) >= self.acceleration_delay{
                self.velocity += 1;
                self.last_acceleration = Instant::now();
            }
        }else{
            self.velocity = acctual_speed as i32;
        }
        // check velocity to turn 
        if turn_velocity >= 0{
            self.velocity = turn_velocity;
        }
        // update position with the velocity
        self.position = match self.route{ 
            Direction::North => Point::new(self.position.x(), self.position.y + self.velocity),
            Direction::East => Point::new(self.position.x-self.velocity,self.position.y),
            Direction::South => Point::new(self.position.x(), self.position.y - self.velocity),
            Direction::West => Point::new(self.position.x+self.velocity, self.position.y),
            _ => Point::new(self.position.x, self.position.y),
        };

        // recalculate hitboxes
        let hitboxes = cal_hitboxes(self.position, self.route);
        self.body = hitboxes.0;
        self.hitbox = hitboxes.1;

        // Update stats
        self.stats.time = Instant::now() - self.id;
        self.stats.distance += self.velocity as u32;

    }
    pub fn arrival(&mut self){
        // calculate the medium velocity from distance and time passed
        // Store the stats
        self.stats.velocity = (self.stats.distance as f64 / self.stats.time.as_secs_f64()).round() as u32;
        // // print the stats
        // println!("time: {}s\ndistance: {}px\nvelocity: {}px/s\n", self.stats.time.as_secs_f64(), self.stats.distance, self.stats.velocity);
    }
}
// ToDo: Optimize and add hitbox for some cases like:
// - stop av before to avoid stopping all traffics
// - side detection more exported on external side
fn cal_hitboxes(position: Point, direction: Direction) -> (Rect,Hitbox) {
    let min_deceleration = (
        SPEED_V.0 + SPEED_V.1,
        SPEED_V.0 + SPEED_V.1 + SPEED_V.2,
    );
    let width = (VEHICLE_WIDTH+2,SAFE_DISTANCE,VEHICLE_WIDTH*3/2,VEHICLE_WIDTH+2);
    let height = (SAFE_DISTANCE+VEHICLE_WIDTH/4,min_deceleration.0*3,min_deceleration.1*3,SAFE_DISTANCE/2);
    let displace = ((VEHICLE_HEIGHT/2 + 2 + height.0/2)as i32,(VEHICLE_HEIGHT/2 + height.0/2) as i32,(VEHICLE_HEIGHT/2 + 2 + height.3/2)as i32);
    match direction {
        Direction::North => {
            let body = Rect::from_center(position, width.0, VEHICLE_HEIGHT+4);
            let hitbox_f = Rect::from_center(
                Point::new(position.x, position.y + displace.0),
                width.3,
                height.0,
            );
            let hitbox_c = Rect::from_center(
                Point::new(position.x, position.y + displace.2),
                width.3,
                height.3,
            );
            let hitbox_sr = Rect::from_center(
                Point::new(position.x - (width.1/2 + VEHICLE_WIDTH/2 + 1)as i32, position.y + displace.1),
                width.1,
                width.2,
            );
            let hitbox_sl = Rect::from_center(
                Point::new(position.x + (width.2/2 + VEHICLE_WIDTH/2 + 2)as i32, position.y + displace.1),
                width.2,
                width.2,
            );
            let hitbox_fd_s = Rect::from_center(
                Point::new(position.x, position.y + (displace.0 as u32 +1+ height.0/2 + height.1 / 2) as i32),
                width.3,
                height.1,
            );
            let hitbox_fd_n = Rect::from_center(
                Point::new(position.x, position.y + (displace.0 as u32 +1+ height.0/2 + height.1 + height.2 / 2) as i32),
                width.3,
                height.2,
            );
             (body,
             Hitbox {
                urgency_stop: hitbox_f,
                closer: hitbox_c,
                slowdown_1: hitbox_fd_n,
                slowdown_2: hitbox_fd_s,
                left: hitbox_sl,
                right: hitbox_sr,
            })
        }
        Direction::South => {
            let body = Rect::from_center(position, width.0, VEHICLE_HEIGHT+4);
            let hitbox_f = Rect::from_center(
                Point::new(position.x, position.y - displace.0),
                width.3,
                height.0,
            );
            let hitbox_c = Rect::from_center(
                Point::new(position.x, position.y - displace.2),
                width.3,
                height.3,
            );
            let hitbox_sr = Rect::from_center(
                Point::new(position.x + (width.1/2 + VEHICLE_WIDTH/2 + 1)as i32, position.y - displace.1),
                width.1,
                width.2,
            );
            let hitbox_sl = Rect::from_center(
                Point::new(position.x - (width.2/2 + VEHICLE_WIDTH/2 + 2)as i32, position.y - displace.1),
                width.2,
                width.2,
            );
            let hitbox_fd_s = Rect::from_center(
                Point::new(position.x, position.y - (displace.0 as u32 +1+ height.0/2 + height.1 / 2) as i32),
                width.3,
                height.1,
            );
            let hitbox_fd_n = Rect::from_center(
                Point::new(position.x, position.y - (displace.0 as u32 +1+ height.0/2 + height.1 + height.2 / 2) as i32),
                width.3,
                height.2,
            ); 
            (body,
             Hitbox {
                urgency_stop: hitbox_f,
                closer: hitbox_c,
                slowdown_1: hitbox_fd_n,
                slowdown_2: hitbox_fd_s,
                left: hitbox_sl,
                right: hitbox_sr,
            })
        }
        Direction::East => {
            let body = Rect::from_center(position, VEHICLE_HEIGHT+4, width.0);
            let hitbox_f = Rect::from_center(
                Point::new(position.x - displace.0, position.y),
                height.0,
                width.3,
            );
            let hitbox_c = Rect::from_center(
                Point::new(position.x - displace.2, position.y ),
                height.3,
                width.3,
            );
            let hitbox_sr = Rect::from_center(
                Point::new(position.x - displace.1, position.y - (width.1/2 + VEHICLE_WIDTH/2 + 1)as i32),
                width.2,
                width.1,
            );
            let hitbox_sl = Rect::from_center(
                Point::new(position.x - displace.1, position.y + (width.2/2 + VEHICLE_WIDTH/2 + 2)as i32),
                width.2,
                width.2,
            );
            let hitbox_fd_s = Rect::from_center(
                Point::new(position.x - (displace.0 as u32 +1+ height.0/2 + height.1 / 2) as i32, position.y),
                height.1,
                width.3,
            );
            let hitbox_fd_n = Rect::from_center(
                Point::new(position.x - (displace.0 as u32 +1+ height.0/2 + height.1 + height.2 / 2) as i32, position.y),
                height.2,
                width.3,
            );
            (body,
             Hitbox {
                urgency_stop: hitbox_f,
                closer: hitbox_c,
                slowdown_1: hitbox_fd_n,
                slowdown_2: hitbox_fd_s,
                left: hitbox_sl,
                right: hitbox_sr,
            })
        }
        Direction::West => {
            let body = Rect::from_center(position, VEHICLE_HEIGHT+4, width.0);
            let hitbox_f = Rect::from_center(
                Point::new(position.x + displace.0, position.y),
                height.0,
                width.3,
            );
            let hitbox_c = Rect::from_center(
                Point::new(position.x + displace.2, position.y ),
                height.3,
                width.3,
            );
            let hitbox_sr = Rect::from_center(
                Point::new(position.x + displace.1, position.y + (width.1/2 + VEHICLE_WIDTH/2 + 1)as i32),
                width.2,
                width.1,
            );
            let hitbox_sl = Rect::from_center(
                Point::new(position.x + displace.1, position.y - (width.2/2 + VEHICLE_WIDTH/2 + 2)as i32),
                width.2,
                width.2,
            );
            let hitbox_fd_s = Rect::from_center(
                Point::new(position.x + (displace.0 as u32 +1+ height.0/2 + height.1 / 2) as i32, position.y),
                height.1,
                width.3,
            );
            let hitbox_fd_n = Rect::from_center(
                Point::new(position.x + (displace.0 as u32 +1+ height.0/2 + height.1 + height.2 / 2) as i32, position.y),
                height.2,
                width.3,
            );
            (body,
             Hitbox {
                urgency_stop: hitbox_f,
                closer: hitbox_c,
                slowdown_1: hitbox_fd_n,
                slowdown_2: hitbox_fd_s,
                left: hitbox_sl,
                right: hitbox_sr,
            })
        }
        _ => todo!(),
    }
}
