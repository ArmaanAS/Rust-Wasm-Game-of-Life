use getrandom::getrandom;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cell_size: u32,
    cells: Vec<Cell>,
    previous_cells: Vec<Cell>,
    canvas: Vec<u32>,
    points: [(u32, u32); 8],
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32, cell_size: u32) -> Universe {
        let mut random = vec![0; (width * height / 8) as usize];
        getrandom(&mut random).unwrap_or_else(|err| println!("{:?}", err));
        let mut i = 0;
        // let mut counter = 0;

        let cells: Vec<Cell> = (0..width * height)
            .map(|_| {
                let random_bool = (random[i / 8] >> (i % 8)) & 1 == 1;
                i += 1;

                if random_bool {
                    // counter += 1;
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        // log!(
        //     "Randomness: {} / {} = {:.1}%",
        //     counter,
        //     width * height,
        //     (100 * counter) as f64 / (width * height) as f64
        // );

        let previous_cells = cells.clone();

        let mut canvas: Vec<u32> = (0..width * height * cell_size * cell_size)
            .map(|_| 0xFFCCCCCC)
            .collect();

        let size = cell_size;

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                let alive = cells[index as usize] == Cell::Alive;
                let col = if alive { 0xFF000000 } else { 0xFFFFFFFF };

                for j in 0..size - 1 {
                    for i in 0..size - 1 {
                        let idx = (y * size + j) * width * size + (x * size + i);
                        canvas[idx as usize] = col;
                    }
                }
            }
        }

        let points = [
            (width - 1, height - 1),
            (width - 1, 0),
            (width - 1, 1),
            (0, height - 1),
            (0, 1),
            (1, height - 1),
            (1, 0),
            (1, 1),
        ];

        Universe {
            width,
            height,
            cells,
            previous_cells,
            cell_size,
            canvas,
            points,
        }
    }

    fn alive_neighbour_count(&self, x: u32, y: u32) -> u8 {
        let mut count = 0;
        for (d_x, d_y) in self.points {
            let neighbor_y = (y + d_y) % self.height;
            let neighbor_x = (x + d_x) % self.width;
            let idx = self.index(neighbor_y, neighbor_x);
            if self.cells[idx] == Cell::Alive {
                count += 1;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.alive_neighbour_count(row, col);

                let next_cell = match (cell, live_neighbours) {
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, _) => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (_, _) => cell,
                };

                self.previous_cells[idx] = next_cell;
            }
        }

        std::mem::swap(&mut self.cells, &mut self.previous_cells);
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn canvas(&mut self) -> *const u32 {
        let size = self.cell_size;

        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.index(x, y);
                if self.cells[index] == self.previous_cells[index] {
                    continue;
                }

                let alive = self.cells[index] == Cell::Alive;
                let col = if alive { 0xFF000000 } else { 0xFFFFFFFF };

                for j in 0..size - 1 {
                    for i in 0..size - 1 {
                        let idx = (y * size + j) * self.width * size + (x * size + i);
                        self.canvas[idx as usize] = col;
                    }
                }
            }
        }

        self.canvas.as_ptr()
    }
}
