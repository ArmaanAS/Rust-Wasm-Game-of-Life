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
    tick_count: u32,
    canvas: Vec<u32>,
    colour_canvas: Vec<u32>,
    points: [(u32, u32); 8],
    cell_neighbour_count: Vec<u8>,
    previous_cell_neighbour_count: Vec<u8>,
}

#[wasm_bindgen]
impl Universe {
    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn increment_neighbour_counts(&mut self, x: u32, y: u32) {
        for (d_x, d_y) in self.points {
            let neighbour_x = (x + d_x) % self.width;
            let neighbour_y = (y + d_y) % self.height;
            let idx = self.index(neighbour_x, neighbour_y);
            self.cell_neighbour_count[idx] += 1;
        }
    }

    fn decrement_neighbour_counts(&mut self, x: u32, y: u32) {
        for (d_x, d_y) in self.points {
            let neighbour_x = (x + d_x) % self.width;
            let neighbour_y = (y + d_y) % self.height;
            let idx = self.index(neighbour_x, neighbour_y);
            self.cell_neighbour_count[idx] -= 1;
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, cell: Cell) {
        let idx = self.index(x, y);
        if self.cells[idx] != cell {
            self.cells[idx] = cell;
            if cell == Cell::Alive {
                self.increment_neighbour_counts(x, y);
            } else {
                self.decrement_neighbour_counts(x, y);
            }
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32, cell_size: u32) -> Universe {
        let cells = vec![Cell::Dead; (width * height) as usize];
        let previous_cells = vec![Cell::Dead; (width * height) as usize];
        let canvas = vec![0xFFCCCCCC; (width * height * cell_size * cell_size) as usize];
        let colour_canvas = vec![0xFFCCCCCC; (width * height) as usize];

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

        let cell_neighbour_count = vec![0u8; (width * height) as usize];
        let previous_cell_neighbour_count = vec![0u8; (width * height) as usize];

        let mut universe = Universe {
            width,
            height,
            cells,
            previous_cells,
            tick_count: 0,
            cell_size,
            canvas,
            colour_canvas,
            points,
            cell_neighbour_count,
            previous_cell_neighbour_count,
        };

        universe.init();

        universe
    }

    pub(self) fn init(&mut self) {
        // Get random bytes
        let mut random = vec![0; (self.width * self.height / 8) as usize];
        getrandom(&mut random).unwrap_or_else(|err| println!("{:?}", err));

        std::mem::swap(
            &mut self.cell_neighbour_count,
            &mut self.previous_cell_neighbour_count,
        );

        // Initialise cells randomly and update counter
        for i in 0..self.height * self.width {
            let random_bool = (random[(i / 8) as usize] >> (i % 8)) & 1 == 0;
            if random_bool {
                self.cells[i as usize] = Cell::Alive;
                self.increment_neighbour_counts(i % self.width, i / self.width);
            }
        }

        // Initialise canvas and colour_canvas
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);

                let red = y * 0xFF / self.height;
                let green = ((self.width - x) * 0xB0 / self.width) << 8;
                let blue = (x * 0xFF / self.width) << 16;
                let colour = 0xFF000000 | red | green | blue;
                self.colour_canvas[idx] = colour;

                let alive = self.cells[idx as usize] == Cell::Alive;
                // let cell_colour = if alive { colour } else { 0xFFFFFFFF };
                let cell_colour = if alive { colour } else { 0xFF000000 };

                let canvas_width = self.width * self.cell_size;
                // for j in 0..self.cell_size - 1 {
                for j in 0..self.cell_size {
                    let canvas_y = y * self.cell_size + j;
                    // for i in 0..self.cell_size - 1 {
                    for i in 0..self.cell_size {
                        let canvas_x = x * self.cell_size + i;
                        let idx = canvas_y * canvas_width + canvas_x;

                        self.canvas[idx as usize] = cell_colour;
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        std::mem::swap(&mut self.cells, &mut self.previous_cells);
        std::mem::swap(
            &mut self.cell_neighbour_count,
            &mut self.previous_cell_neighbour_count,
        );

        for i in 0..self.cell_neighbour_count.len() {
            self.cell_neighbour_count[i] = 0;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let cell = self.previous_cells[idx];
                let live_neighbours = self.previous_cell_neighbour_count[idx];

                let next_cell = match (cell, live_neighbours) {
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, _) => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (_, _) => cell,
                };

                self.cells[idx] = next_cell;
                if next_cell == Cell::Alive {
                    self.increment_neighbour_counts(x, y);
                }
            }
        }

        self.tick_count += 1;
    }

    pub fn canvas(&mut self) -> *const u32 {
        let canvas_width = self.width * self.cell_size;
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.index(x, y);
                let alive = self.cells[index] == Cell::Alive;
                let colour = if alive {
                    self.colour_canvas[index]
                } else {
                    // 0xFFFFFFFF
                    0xFF000000
                };

                let mut canvas_y = y * self.cell_size;
                let mut canvas_x = x * self.cell_size;
                let mut idx = canvas_y * canvas_width + canvas_x;
                if colour == self.canvas[idx as usize] {
                    continue;
                }

                // for j in 0..self.cell_size - 1 {
                for j in 0..self.cell_size {
                    canvas_y = y * self.cell_size + j;
                    // for i in 0..self.cell_size - 1 {
                    for i in 0..self.cell_size {
                        canvas_x = x * self.cell_size + i;
                        idx = canvas_y * canvas_width + canvas_x;

                        self.canvas[idx as usize] = colour;
                    }
                }
            }
        }

        self.canvas.as_ptr()
    }
}
