
use sdl2::rect::{Point, Rect};
use crate::{ROAD_WIDTH,ROAD_NUMBER};
#[derive(Clone, Debug, PartialEq)]
pub struct Sector {
    pub map: Rect,
    pub entry_intersect: Rect,
    pub in_intersect: Rect,
    pub turn_north: (Rect,Rect),
    pub turn_south: (Rect,Rect),
    pub turn_east: (Rect,Rect),
    pub turn_west: (Rect,Rect),
}
impl Sector {
    pub fn new(map: Rect) -> Sector {
        let entry_intersect = Rect::new(
            ((map.width() - ROAD_WIDTH )/2-ROAD_WIDTH/4) as i32,
            ((map.height() - ROAD_WIDTH )/2-ROAD_WIDTH/4)as i32,
            ROAD_WIDTH+ROAD_WIDTH/2,
            ROAD_WIDTH+ROAD_WIDTH/2
        );
        let in_intersect = Rect::new(
            ((map.width() - ROAD_WIDTH )/2 + ROAD_WIDTH / (ROAD_NUMBER * 2))as i32,
            ((map.height() - ROAD_WIDTH )/2  + ROAD_WIDTH / (ROAD_NUMBER * 2))as i32,
            ROAD_WIDTH - ROAD_WIDTH / ROAD_NUMBER,
            ROAD_WIDTH - ROAD_WIDTH / ROAD_NUMBER
        );
        let cubesize= ROAD_WIDTH/(ROAD_NUMBER*2);
        let turn_north = (
            Rect::from_center(
                Point::new(
                    ((map.width() - ROAD_WIDTH) /2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32 + ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32,
                    ((map.height() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32 - ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32,
                ),
                cubesize,
                cubesize
            ),
            Rect::from_center(
                Point::new(
                    ((map.width() - ROAD_WIDTH) /2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32,
                    ((map.height() - ROAD_WIDTH) / 2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32
                ),
                cubesize,
                cubesize
            )
        );
        let turn_south = (
            Rect::from_center(
                Point::new(
                    ((map.width() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32 - ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32,
                    ((map.height() - ROAD_WIDTH) / 2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32 + ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32
                ),
                cubesize,
                cubesize
            ),
            Rect::from_center(
                Point::new(
                    ((map.width() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32, 
                    ((map.height() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32
                ),
                cubesize,
                cubesize
            ),
        );
        let turn_east = (
            Rect::from_center(
                Point::new(
                    ((map.width() - ROAD_WIDTH) /2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32 + ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32,
                    ((map.height() - ROAD_WIDTH) / 2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32 + ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32
                ),
                cubesize,
                cubesize
            ),
            Rect::from_center(
                Point::new(
                    ((map.width() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32,
                    ((map.height() - ROAD_WIDTH) / 2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32
                ),
                cubesize,
                cubesize
            ),
        );
        let turn_west = (
            Rect::from_center(
                Point::new(
                    ((map.width() + ROAD_WIDTH) /2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32 - ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32,
                    ((map.height() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32 - ((ROAD_WIDTH/(ROAD_NUMBER*2))*2) as i32
                ),
                cubesize,
                cubesize
            ),
            Rect::from_center(
                Point::new(
                    ((map.width() - ROAD_WIDTH) /2 + (ROAD_WIDTH/(ROAD_NUMBER*2))/2)as i32,
                    ((map.height() + ROAD_WIDTH) / 2 - (ROAD_WIDTH/(ROAD_NUMBER*2))/2) as i32
                ),
                cubesize,
                cubesize
            ),
        );
        Sector{
            map,
            entry_intersect,
            in_intersect,
            turn_north,
            turn_south,
            turn_east,
            turn_west,
        }
    }
}
// );