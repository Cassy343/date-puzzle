use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
    io::{stdout, Write},
    ops::Add,
};

// Invariants: always contains a point centered at the origin
#[derive(Clone)]
pub struct Tile {
    points: Vec<Point>,
}

impl Tile {
    pub fn new(points: Vec<Point>) -> Self {
        if !points.iter().any(|point| point.x == 0 && point.y == 0) {
            panic!("A valid tile must contain a point at the origin.");
        }

        Self { points }
    }

    pub fn rotate_ccw_90(&mut self) {
        self.points
            .iter_mut()
            .for_each(|point| *point = point.rotated_ccw_90())
    }

    pub fn reflect_over_vert(&mut self) {
        self.points
            .iter_mut()
            .for_each(|point| *point = point.reflected_over_vert())
    }

    pub fn offset_points(&self, offset: Point) -> impl Iterator<Item = Point> + '_ {
        self.points.iter().map(move |&point| point + offset)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Point {
    // positive y
    // ^
    // |
    // |
    // +------> positive x
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn rotated_ccw_90(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn reflected_over_vert(&self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
        }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Invariants: min.x <= max.x and min.y <= max.y
#[derive(Clone, Copy)]
pub struct AABB {
    // Inclusive
    min: Point,
    // Inclusive
    max: Point,
}

impl AABB {
    pub fn new(min: Point, max: Point) -> Self {
        assert!(min.x <= max.x && min.y <= max.y);

        Self { min, max }
    }

    pub fn points(&self) -> impl Iterator<Item = Point> {
        let min = self.min;
        let max = self.max;

        (min.x..=max.x).flat_map(move |x| (min.y..=max.y).map(move |y| Point { x, y }))
    }
}

// Invariants: has no more than 64 squares, constituent AABBs do not overlap
#[derive(Clone)]
pub struct EncodingBoard {
    aabbs: Vec<AABB>,
    encoding: HashMap<Point, u64>,
}

impl EncodingBoard {
    pub fn new(aabbs: Vec<AABB>) -> Self {
        let mut encoding = HashMap::new();
        let mut enc = 1u64;

        for aabb in &aabbs {
            for point in aabb.points() {
                if enc == 0 {
                    panic!("More than 64 tiles in Board");
                }

                match encoding.entry(point) {
                    Entry::Vacant(entry) => {
                        entry.insert(enc);
                    }
                    Entry::Occupied(..) => panic!("Overlapping AABBs in Board"),
                }

                enc = enc.overflowing_shl(1).0;
            }
        }

        Self { aabbs, encoding }
    }

    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        self.aabbs.iter().flat_map(|aabb| aabb.points())
    }

    pub fn encode(&self, points: impl Iterator<Item = Point>) -> Option<u64> {
        points
            .map(|point| self.encoding.get(&point).copied())
            .reduce(|a, b| match (a, b) {
                (Some(a), Some(b)) => Some(a | b),
                _ => None,
            })
            .flatten()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Square {
    Covered { tile_id: u8 },
    Uncovered,
    Unknown,
}

pub struct DecodingBoard {
    decoding: HashMap<u64, (usize, usize)>,
    rows: Vec<Vec<Square>>,
}

impl DecodingBoard {
    pub fn decode(&mut self, enc: u64, tile_id: Option<u8>) {
        let mut mask = 1u64;
        while mask != 0 {
            if enc & mask != 0 {
                let &(col, row) = self
                    .decoding
                    .get(&mask)
                    .expect("Encoding not compatible with this board");
                self.rows[col][row] = if let Some(id) = tile_id {
                    Square::Covered { tile_id: id }
                } else {
                    Square::Uncovered
                };
            }

            mask = mask.overflowing_shl(1).0;
        }
    }

    pub fn print(&self) {
        let mut stdout = stdout().lock();

        for (col_idx, row) in self.rows.iter().enumerate() {
            for (row_idx, &square) in row.iter().enumerate() {
                if col_idx == 0
                    || row_idx >= self.rows[col_idx - 1].len()
                    || self.rows[col_idx - 1][row_idx] != square
                {
                    stdout.write(b"+---").unwrap();
                } else {
                    stdout.write(b"+   ").unwrap();
                }
            }

            if col_idx > 0 {
                for _ in row.len()..self.rows[col_idx - 1].len() {
                    stdout.write(b"+---").unwrap();
                }
            }

            stdout.write(b"+\n").unwrap();

            for (row_idx, &square) in row.iter().enumerate() {
                let left = if row_idx == 0 || row[row_idx - 1] != square {
                    '|' as u8
                } else {
                    32
                };

                let center = match square {
                    Square::Covered { .. } => 32,
                    Square::Uncovered => '#' as u8,
                    Square::Unknown => '?' as u8,
                };

                stdout.write(&[left, 32, center, 32]).unwrap();
            }
            stdout.write(b"|\n").unwrap();

            if col_idx == self.rows.len() - 1 {
                for _ in 0..row.len() {
                    stdout.write(b"+---").unwrap();
                }
                stdout.write(b"+\n").unwrap();
            }
        }
    }
}

impl From<EncodingBoard> for DecodingBoard {
    fn from(board: EncodingBoard) -> Self {
        fn rev_i32_order(x: i32) -> i32 {
            if x == i32::MIN {
                i32::MAX
            } else {
                1 - x
            }
        }

        let mut points = board.points().collect::<Vec<_>>();
        points.sort_by(|lhs, rhs| {
            let y_lhs = rev_i32_order(lhs.y);
            let y_rhs = rev_i32_order(rhs.y);

            let y_cmp = y_lhs.cmp(&y_rhs);
            if y_cmp == Ordering::Equal {
                let x_lhs = lhs.x;
                let x_rhs = rhs.x;

                x_lhs.cmp(&x_rhs)
            } else {
                y_cmp
            }
        });

        let mut prev_y = None;
        let mut rows = Vec::new();
        let mut row = Vec::new();
        let mut point_map = HashMap::new();

        for point in points {
            match (point.y, prev_y) {
                (y, Some(py)) if y != py => {
                    rows.push(row);
                    row = Vec::new();
                    prev_y = Some(y);
                }
                (y, None) => {
                    prev_y = Some(y);
                }
                _ => (),
            }

            point_map.insert(point, (rows.len(), row.len()));
            row.push(Square::Unknown);
        }
        rows.push(row);

        Self {
            decoding: board
                .encoding
                .into_iter()
                .map(|(point, enc)| (enc, *point_map.get(&point).unwrap()))
                .collect(),
            rows,
        }
    }
}
