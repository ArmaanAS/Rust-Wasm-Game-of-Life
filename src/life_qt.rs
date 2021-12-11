use getrandom::getrandom;
use quadtree_rs::{point::Point, Quadtree};
use std::{cmp::max, intrinsics::log2f64};
use wasm_bindgen::prelude::*;
use web_sys::console::log_2;

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
    cells: Quadtree<u32, Cell>,
    previous_cells: Quadtree<u32, Cell>,
    canvas: Vec<u32>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32, cell_size: u32) -> Universe {
        let mut random = vec![0; (width * height / 8) as usize];
        getrandom(&mut random).unwrap_or_else(|err| println!("{:?}", err));
        let mut i = 0;

        // let cells = Quadtree::<u32, Cell>::new(log2f64(max(width, height)) as u32);
        let cells = Quadtree::<u32, Cell>::new(4);
        for y in 0..height {
            for x in 0..width {
                let random_bool = (random[i / 8] >> (i % 8)) & 1 == 1;
                i += 1;

                if random_bool {
                    cells.insert_pt(Point { x, y }, Cell::Alive);
                }
            }
        }

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

        Universe {
            width,
            height,
            cells,
            previous_cells,
            cell_size,
            canvas,
        }
    }

    fn alive_neighbour_count(&self, x: u32, y: u32) -> u8 {
        let mut count = 0;
        for delta_y in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_y == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_y = (y + delta_y) % self.height;
                let neighbor_col = (x + delta_col) % self.width;
                let idx = self.get(neighbor_y, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get(row, col);
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

    fn get(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn canvas(&mut self) -> *const u32 {
        let size = self.cell_size;

        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.get(x, y);
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
