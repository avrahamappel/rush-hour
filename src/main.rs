use std::collections::VecDeque;
use std::fmt::Display;
use std::iter;
use std::process::exit;

use itertools::Itertools;

fn id<T>(x: T) -> T {
    x
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum CarType {
    Player,
    Other,
}

type Id = char;

#[derive(Debug, PartialEq, Clone)]
struct Car {
    id: Id,
    car_type: CarType,
    x: usize,
    y: usize,
    length: usize,
    horizontal: bool,
}

impl Car {
    fn new(id: Id, ps: Vec<(usize, usize)>) -> Self {
        let car_type = if id == 'X' {
            CarType::Player
        } else {
            CarType::Other
        };

        let length = ps.len();

        let (xs, ys): (Vec<_>, Vec<_>) = ps.into_iter().unzip();

        let horizontal = ys.iter().all_equal();

        let x = xs
            .into_iter()
            .min()
            .expect("shouldn't get this far with an empty array");
        let y = ys
            .into_iter()
            .min()
            .expect("shouldn't get this far with an empty array");

        Self {
            id,
            car_type,
            x,
            y,
            length,
            horizontal,
        }
    }

    fn is_player(&self) -> bool {
        matches!(self.car_type, CarType::Player)
    }

    fn head(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    fn body(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        iter::successors(Some(self.head()), |(px, py)| {
            if self.horizontal {
                if *px == self.x + self.length - 1 {
                    None
                } else {
                    Some((px + 1, *py))
                }
            } else {
                #[allow(clippy::collapsible_if)]
                if *py == self.y + self.length - 1 {
                    None
                } else {
                    Some((*px, py + 1))
                }
            }
        })
    }

    fn tail(&self) -> (usize, usize) {
        self.body().last().expect("shouldn't be an empty body")
    }

    fn includes(&self, other: &Self) -> bool {
        if self.id == other.id {
            return false;
        }

        for pos in self.body() {
            for opos in other.body() {
                if pos == opos {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, Copy)]
enum Dir {
    Forward,
    Backward,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    exit: (usize, usize),
    cars: Vec<Car>,
}

impl Grid {
    fn parse(input: &str) -> Option<Self> {
        let puzzle: Vec<Vec<_>> = input
            .trim()
            .lines()
            .map(|l| l.trim().chars().collect())
            .collect();

        let exit = {
            let (mut x, mut y) = puzzle.iter().enumerate().find_map(|(y, line)| {
                line.iter()
                    .enumerate()
                    .find_map(|(x, c)| c.eq(&'x').then_some((x, y)))
            })?;

            // Account for borders
            if x != 0 {
                x -= 1
            }
            if y != 0 {
                y -= 1
            }

            (x, y)
        };

        let width = puzzle[0].len() - 2;
        let height = puzzle.len() - 2;

        let cars = puzzle
            .into_iter()
            .enumerate()
            .fold(vec![], |cars: Vec<(_, Vec<_>)>, (y, line)| {
                line.into_iter()
                    .enumerate()
                    .fold(cars, |mut cars, (x, c)| match c {
                        '.' | ' ' | '+' | '-' | '|' | 'x' => cars,
                        a => {
                            if let Some((_, ps)) = cars.iter_mut().find(|(id, _)| *id == a) {
                                ps.push((x - 1, y - 1))
                            } else {
                                cars.push((a, vec![(x - 1, y - 1)]))
                            }

                            cars
                        }
                    })
            })
            .into_iter()
            .map(|(id, ps)| Car::new(id, ps))
            .collect();

        Some(Self {
            width,
            height,
            exit,
            cars,
        })
    }

    fn is_solved(&self) -> bool {
        self.cars
            .iter()
            .filter(|c| c.is_player())
            .any(|c| c.head() == self.exit)
    }

    fn car_fits(&self, car: &Car) -> bool {
        // dbg!(car.head(), car.tail());

        if car.is_player() && car.head() == self.exit {
            return true;
        }

        car.tail().0 < self.width
            && car.tail().1 < self.height
            && !self.cars.iter().any(|c| c.includes(car))
    }

    fn move_car(mut self, id: Id, dir: Dir) -> Option<Self> {
        self.cars
            .iter()
            .cloned()
            .find_position(|c| c.id == id)
            .and_then(|(i, mut c)| {
                // dbg!(i, &c, dir);

                let try_move = |car| self.car_fits(&car).then_some(car);

                let new_car = match dir {
                    Dir::Backward => {
                        if c.horizontal {
                            c.x += 1;
                        } else {
                            c.y += 1;
                        }
                        try_move(c)
                    }
                    Dir::Forward => {
                        if c.horizontal {
                            c.x.checked_sub(1).and_then(|x| {
                                c.x = x;
                                try_move(c)
                            })
                        } else {
                            c.y.checked_sub(1).and_then(|y| {
                                c.y = y;
                                try_move(c)
                            })
                        }
                    }
                };

                // dbg!(&new_car);

                new_car.map(|car| {
                    self.cars[i] = car;
                    self
                })
            })
    }

    fn next_moves(&self) -> Vec<Grid> {
        self.cars
            .iter()
            .flat_map(|car| {
                [
                    self.clone().move_car(car.id, Dir::Forward),
                    self.clone().move_car(car.id, Dir::Backward),
                ]
            })
            .filter_map(id)
            .collect()
    }

    fn diff(&self, other: &Grid) -> Option<(Id, &'static str)> {
        self.cars
            .iter()
            .filter_map(|car| {
                other
                    .cars
                    .iter()
                    .find(|ocar| ocar.id == car.id)
                    .map(|ocar| (car, ocar))
            })
            .find(|(c, oc)| c.head() != oc.head())
            .map(|(c, oc)| {
                let dir = if c.x > oc.x {
                    "right"
                } else if c.x < oc.x {
                    "left"
                } else if c.y > oc.y {
                    "down"
                } else {
                    "up"
                };

                (c.id, dir)
            })
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut grid: Vec<Vec<_>> = (0..self.height)
            .map(|_| (0..self.width).map(|_| '.').collect())
            .collect();

        for car in &self.cars {
            for (x, y) in car.body() {
                grid[y][x] = car.id;
            }
        }

        for line in grid {
            writeln!(f)?;
            for char in line {
                write!(f, "{char}")?;
            }
        }

        writeln!(f)
    }
}

type Step = (Id, &'static str, usize);

fn get_history(visited: &[(Grid, Option<Grid>)], grid: Grid) -> Vec<Step> {
    #[allow(clippy::needless_collect)]
    let path: Vec<_> = iter::successors(Some(&grid), |prev| {
        visited
            .iter()
            .find_map(|(g, p)| (g == *prev).then_some(p.as_ref()).flatten())
    })
    .collect();

    path.into_iter()
        .rev()
        .tuple_windows()
        .filter_map(|(prev, grid)| grid.diff(prev))
        // .inspect(|x| {
        //     dbg!(x);
        // })
        .fold(Vec::new(), |mut acc, diff| {
            // dbg!(&acc, diff);

            match acc.last_mut() {
                Some((car, dir, n)) if (*car, *dir) == diff => *n += 1,
                _ => {
                    acc.push((diff.0, diff.1, 1));
                }
            }

            acc
        })
}

fn solve(grid: Grid) -> Option<Vec<Step>> {
    let mut queue = VecDeque::new();
    let mut visited = Vec::new();

    visited.push((grid.clone(), None));
    queue.push_back(grid);

    while let Some(grid) = queue.pop_front() {
        eprint!(".");

        // eprintln!("Step {}", visited.len());
        // eprintln!("Grid: {grid}");

        if Grid::is_solved(&grid) {
            return Some(get_history(&visited, grid));
        }

        for m in grid.next_moves() {
            if !visited.iter().any(|(g, _)| *g == m) {
                visited.push((m.clone(), Some(grid.clone())));
                queue.push_back(m);
            }
        }
    }

    None
}
fn main() {
    // Create a new Puzzle instance and solve it.
    let grid = Grid::parse(
        "
        +--x---+
        |...LLL|
        |......|
        |..BBBR|
        |...G.R|
        |..XGUU|
        |..X...|
        +------+
        ",
    )
    .unwrap();

    // dbg!(&grid);

    if let Some(solution) = solve(grid) {
        println!();
        println!("Solution found!");

        for (car, dir, count) in solution {
            println!("{} {:?} {}", car, dir, count);
        }
    } else {
        println!("No solution found.");
        exit(1);
    }
}