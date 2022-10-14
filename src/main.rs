use std::collections::HashSet;

use crate::entity::{DecodingBoard, Point};

mod backtrack;
mod entity;

macro_rules! aabb {
    (($x0:literal, $y0:literal) -> ($x1:literal, $y1:literal)) => {
        $crate::entity::AABB::new(
            $crate::entity::Point { x: $x0, y: $y0 },
            $crate::entity::Point { x: $x1, y: $y1 },
        )
    };
}

macro_rules! enc_board {
    ($( $aabb:expr, )+) => {
        $crate::entity::EncodingBoard::new(vec![$( $aabb ),+])
    };
}

macro_rules! tile {
    ($( ($x:literal, $y:literal), )+) => {
        $crate::entity::Tile::new(vec![$( $crate::entity::Point { x: $x, y: $y } ),+])
    };
}

#[repr(i32)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

fn main() {
    let encoding_board = enc_board!(
        aabb!((0,0) -> (2,0)),
        aabb!((0,1) -> (6,4)),
        aabb!((0,5) -> (5,6)),
    );

    #[rustfmt::skip]
    let tiles = vec![
        tile! {
            (0,2),
            (0,1),
            (0,0),(1,0),(2,0),
        },
        tile! {
            (0,3),
            (0,2),
            (0,1),
            (0,0),(1,0),
        },
        tile! {
                  (1,3),
            (0,2),(1,2),
            (0,1),
            (0,0),
        },
        tile! {
            (0,2),(1,2),
            (0,1),(1,1),
            (0,0),(1,0),
        },
        tile! {
            (0,2),
            (0,1),(1,1),
            (0,0),(1,0),
        },
        tile! {
            (0,2),(1,2),
            (0,1),
            (0,0),(1,0),
        },
        tile! {
            (0,3),
            (0,2),
            (0,1),(1,1),
            (0,0),
        },
        tile! {
                        (2,2),
            (0,1),(1,1),(2,1),
            (0,0),
        }
    ];

    let configurations = tiles
        .into_iter()
        .map(|mut tile| {
            let mut tile_configs = HashSet::new();

            for _ in 0..2 {
                for _ in 0..4 {
                    for offset in encoding_board.points() {
                        if let Some(enc) = encoding_board.encode(tile.offset_points(offset)) {
                            tile_configs.insert(enc);
                        }
                    }

                    tile.rotate_ccw_90();
                }

                tile.reflect_over_vert();
            }

            tile_configs
        })
        .collect::<Vec<_>>();

    const MONTH: Month = Month::October;
    const DAY: i32 = 13;

    let month_num = MONTH as i32;
    let month_point = Point {
        x: month_num % 6,
        y: 6 - month_num / 6,
    };

    let day_point = Point {
        x: (DAY - 1) % 7,
        y: 4 - (DAY - 1) / 7,
    };

    let initial_board = encoding_board
        .encode([month_point, day_point].into_iter())
        .expect("Month and day should lie within the board");

    let mut placements = Vec::new();
    let mut placement_indices = Vec::with_capacity(configurations.len() + 1);

    for configs in configurations {
        placement_indices.push(placements.len());

        for enc in configs.into_iter().filter(|&enc| enc & initial_board == 0) {
            placements.push(enc);
        }
    }

    placement_indices.push(placements.len());

    let solution = backtrack::backtrack(initial_board, &placements, &placement_indices);

    if let Some(solution) = solution {
        let mut decoding_board = DecodingBoard::from(encoding_board);
        solution
            .iter()
            .enumerate()
            .map(|(id, &index)| (id as u8, placements[index]))
            .for_each(|(id, enc)| decoding_board.decode(enc, Some(id)));
        decoding_board.decode(initial_board, None);
        decoding_board.print();
    } else {
        println!("No solution found :(");
    }
}
