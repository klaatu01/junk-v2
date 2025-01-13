use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

// Constants defining the grid size
const MAX_X: usize = 33;
const MAX_Y: usize = 33;

// Enum representing the different types of cells in the grid
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellType {
    Empty,
    Cockpit,
    Hull,
    Engine,
}

// Struct representing the cellular automata grid
pub struct Automata {
    cells: [[CellType; MAX_Y]; MAX_X],
    active: HashSet<(usize, usize)>, // Active cells to process
}

impl Automata {
    /// Creates a new Automata instance with all cells initialized to Empty,
    /// except the center cell which is set to Cockpit.
    pub fn new() -> Self {
        let mut cells = [[CellType::Empty; MAX_Y]; MAX_X];

        // Place the cockpit at the center of the grid
        let center_x = MAX_X / 2;
        let center_y = MAX_Y / 2;
        cells[center_x][center_y] = CellType::Cockpit;

        // Perform a weighted random walk to create initial hull cells
        let mut rng = thread_rng();
        let walk_length = 12; // Adjust based on desired ship size
        let mut x = center_x as isize;
        let mut y = center_y as isize;

        // Define directions: 0 = North, 1 = South, 2 = East, 3 = West
        let directions = ["North", "South", "East", "West"];
        // Assign weights: North less likely
        let weights = [1, 3, 6, 6]; // Adjust weights as desired

        // Create a WeightedIndex distribution
        let mut dist = WeightedIndex::new(&weights).unwrap();

        for _ in 0..walk_length {
            let reset = rng.gen_range(0..20);
            if reset == 0 {
                // Reset to center
                x = center_x as isize;
                y = center_y as isize;
            }
            //
            // Determine direction
            let direction = if cells[x as usize][y as usize] == CellType::Cockpit {
                // Force South direction from the cockpit
                1
            } else {
                // Weighted random selection for directions
                dist.sample(&mut rng)
            };

            // Update coordinates based on direction
            match direction {
                0 => y -= 1, // North
                1 => y += 1, // South
                2 => x += 1, // East
                3 => x -= 1, // West
                _ => {}
            }

            // Ensure the walk stays within bounds
            if x < 0 || x >= MAX_X as isize || y < 0 || y >= MAX_Y as isize {
                continue;
            }

            let (ux, uy) = (x as usize, y as usize);

            // Mark the cell as Hull if it's Empty and check cell below is not Cockpit
            if cells[ux][uy] == CellType::Empty && cells[ux][uy + 1] != CellType::Cockpit {
                cells[ux][uy] = CellType::Hull;
            }
        }

        // Initialize active list with hull cells and their neighbors
        let mut active = HashSet::new();
        for ux in 0..MAX_X {
            for uy in 0..MAX_Y {
                if cells[ux][uy] == CellType::Hull || cells[ux][uy] == CellType::Cockpit {
                    // Add all neighbors to active list
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = ux as isize + dx;
                            let ny = uy as isize + dy;
                            if nx >= 0 && nx < MAX_X as isize && ny >= 0 && ny < MAX_Y as isize {
                                active.insert((nx as usize, ny as usize));
                            }
                        }
                    }
                }
            }
        }

        Automata { cells, active }
    }

    /// Runs the cellular automata for a specified number of iterations.
    pub fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.step();
        }
        self.post_process();
        self.remove_disconnected_cells(); // Ensure connectivity
    }

    /// Performs a single iteration step of the cellular automata.
    fn step(&mut self) {
        let mut changes = Vec::new();
        let current_active: Vec<(usize, usize)> = self.active.iter().cloned().collect();
        self.active.clear();

        let mut rng = thread_rng();

        for (x, y) in current_active {
            let neighbors = self.count_neighbors(x, y);
            let current_cell = self.cells[x][y];

            match current_cell {
                CellType::Empty => {
                    // and check cell below is not Cockpit
                    if neighbors >= 3 && self.cells[x][y + 1] != CellType::Cockpit {
                        let chance = rng.gen_range(0..10);
                        if chance < 4 {
                            changes.push(((x, y), CellType::Hull));
                        }
                    }
                }
                CellType::Hull => {
                    if neighbors <= 1 {
                        let chance = rng.gen_range(0..10);
                        if chance < 4 {
                            changes.push(((x, y), CellType::Empty));
                        }
                    }
                }
                CellType::Cockpit => {
                    // Preserve cockpit
                }
                CellType::Engine => {
                    // Engines are handled in post-processing
                }
            }
        }

        // Apply changes and update the active list
        for ((x, y), new_state) in changes.iter() {
            self.cells[*x][*y] = *new_state;
            // Add neighbors of this cell to active list for next step
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = *x as isize + dx;
                    let ny = *y as isize + dy;
                    if nx >= 0 && nx < MAX_X as isize && ny >= 0 && ny < MAX_Y as isize {
                        self.active.insert((nx as usize, ny as usize));
                    }
                }
            }
        }
    }

    /// Counts the number of neighboring cells that are either Hull or Cockpit.
    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        let directions = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for &(dx, dy) in &directions {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && nx < MAX_X as isize && ny >= 0 && ny < MAX_Y as isize {
                let neighbor = self.cells[nx as usize][ny as usize];
                if neighbor == CellType::Hull || neighbor == CellType::Cockpit {
                    count += 1;
                }
            }
        }

        count
    }

    /// Post-processes the grid after CA iterations, such as placing engines.
    fn post_process(&mut self) {
        let bottom_y = MAX_Y - 1;
        for x in 0..MAX_X {
            if self.cells[x][bottom_y] == CellType::Hull {
                self.cells[x][bottom_y] = CellType::Engine;
            }
        }
    }

    /// Validates that all Hull and Engine cells are connected to the Cockpit.
    pub fn validate_connectivity(&self) -> bool {
        let mut visited = [[false; MAX_Y]; MAX_X];
        let mut queue = std::collections::VecDeque::new();

        // Find the Cockpit position
        let mut cockpit_found = false;
        for x in 0..MAX_X {
            for y in 0..MAX_Y {
                if self.cells[x][y] == CellType::Cockpit {
                    queue.push_back((x, y));
                    visited[x][y] = true;
                    cockpit_found = true;
                    break;
                }
            }
            if cockpit_found {
                break;
            }
        }

        if !cockpit_found {
            return false;
        }

        // Perform BFS to visit all connected Hull and Engine cells
        while let Some((x, y)) = queue.pop_front() {
            let directions = [
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ];

            for &(dx, dy) in &directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < MAX_X as isize && ny >= 0 && ny < MAX_Y as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if !visited[nx][ny]
                        && (self.cells[nx][ny] == CellType::Hull
                            || self.cells[nx][ny] == CellType::Engine)
                    {
                        visited[nx][ny] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }
        }

        // Check if any Hull or Engine cells are not visited
        for x in 0..MAX_X {
            for y in 0..MAX_Y {
                if (self.cells[x][y] == CellType::Hull || self.cells[x][y] == CellType::Engine)
                    && !visited[x][y]
                {
                    return false;
                }
            }
        }

        true
    }

    /// Removes disconnected Hull and Engine cells.
    pub fn remove_disconnected_cells(&mut self) {
        if self.validate_connectivity() {
            return; // All cells are connected; nothing to do
        }

        let mut visited = [[false; MAX_Y]; MAX_X];
        let mut queue = std::collections::VecDeque::new();

        // Find the Cockpit position
        let mut cockpit_found = false;
        for x in 0..MAX_X {
            for y in 0..MAX_Y {
                if self.cells[x][y] == CellType::Cockpit {
                    queue.push_back((x, y));
                    visited[x][y] = true;
                    cockpit_found = true;
                    break;
                }
            }
            if cockpit_found {
                break;
            }
        }

        if !cockpit_found {
            // If there's no Cockpit, clear all cells
            for x in 0..MAX_X {
                for y in 0..MAX_Y {
                    self.cells[x][y] = CellType::Empty;
                }
            }
            return;
        }

        // Perform BFS to mark connected cells
        while let Some((x, y)) = queue.pop_front() {
            let directions = [
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ];

            for &(dx, dy) in &directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < MAX_X as isize && ny >= 0 && ny < MAX_Y as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if !visited[nx][ny]
                        && (self.cells[nx][ny] == CellType::Hull
                            || self.cells[nx][ny] == CellType::Engine)
                    {
                        visited[nx][ny] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }
        }

        // Remove any cells that weren't visited (i.e., disconnected)
        for x in 0..MAX_X {
            for y in 0..MAX_Y {
                if (self.cells[x][y] == CellType::Hull || self.cells[x][y] == CellType::Engine)
                    && !visited[x][y]
                {
                    self.cells[x][y] = CellType::Empty;
                }
            }
        }
    }

    /// Displays the current state of the grid in the console for debugging.
    pub fn display(&self) {
        for y in 0..MAX_Y {
            for x in 0..MAX_X {
                let symbol = match self.cells[x][y] {
                    CellType::Empty => '.',
                    CellType::Cockpit => 'C',
                    CellType::Hull => 'H',
                    CellType::Engine => 'E',
                };
                print!("{}", symbol);
            }
            println!();
        }
    }

    pub fn get_non_empty(&self) -> HashMap<(usize, usize), CellType> {
        let mut non_empty = HashMap::new();
        for x in 0..MAX_X {
            for y in 0..MAX_Y {
                if self.cells[x][y] != CellType::Empty {
                    non_empty.insert((x, y), self.cells[x][y]);
                }
            }
        }
        non_empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automata_run() {
        let mut automata = Automata::new();
        automata.run(8);
        automata.display();
        assert!(automata.validate_connectivity());
    }

    #[test]
    fn batch_test() {
        for i in 0..100 {
            let mut automata = Automata::new();
            automata.run(i);
            automata.display();
            assert!(automata.validate_connectivity());
        }
    }
}
