use rand::distributions::Uniform;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

#[derive(Clone, Copy, Debug)]
pub struct Point2D {
    pub x: isize,
    pub y: isize,
}

/// Generate points using Poisson Disk Sampling with a seeded RNG,
/// returning integer coordinates.
///
/// - `width`, `height`: bounding rectangle in integer coordinates.
/// - `min_dist`: minimum spacing (in float). If, for example, you want
///               at least 5 units of distance between points, set `min_dist = 5.0`.
/// - `k`: number of attempts around each active point (often 30).
/// - `seed`: seed for reproducible randomness.
///
/// Returns a list of points (`Point2D`) within the `[0..width, 0..height]` range.
pub fn sample(width: isize, height: isize, min_dist: f32, k: usize, seed: u64) -> Vec<Point2D> {
    // We'll do our distance math in float, but store final points as isize.
    let w_f = width as f32;
    let h_f = height as f32;

    // Initialize pseudo-random generator with a fixed seed.
    let mut rng = StdRng::seed_from_u64(seed);

    // Cell size is typically min_dist / sqrt(2).
    // This helps skip checking far-away cells.
    let cell_size = min_dist / (2.0_f32).sqrt();

    // Number of columns/rows in our grid (used for neighbor checks).
    let cols = (w_f / cell_size).ceil() as isize;
    let rows = (h_f / cell_size).ceil() as isize;

    // This grid will hold the indices of points in `points` or -1 if empty.
    // We’ll make it a single vector. Index = col + row * cols.
    let mut grid = vec![-1; (cols * rows) as usize];

    // The final list of accepted points.
    let mut points = Vec::new();

    // The "active list" of points we’re trying to expand.
    let mut active_list = Vec::new();

    // Helper function to map (col, row) -> index in `grid`.
    let grid_index = |col: isize, row: isize| -> usize { (col + row * cols) as usize };

    //
    // 1) Start by placing one initial point randomly inside the bounding box.
    //
    let x0_f = rng.gen_range(0.0..w_f);
    let y0_f = rng.gen_range(0.0..h_f);

    let p0 = Point2D {
        x: x0_f as isize,
        y: y0_f as isize,
    };
    points.push(p0);

    // Determine the cell of this initial point (in float, then as isize).
    let c0 = (x0_f / cell_size) as isize;
    let r0 = (y0_f / cell_size) as isize;

    // Mark this cell in the grid with the index of the point (0).
    grid[grid_index(c0, r0)] = 0;

    // Add it to the active list
    active_list.push(0);

    //
    // 2) Loop while we have active points
    //
    while !active_list.is_empty() {
        // Randomly pick a point from the active list
        let rand_i = rng.gen_range(0..active_list.len());
        let p_idx = active_list[rand_i];
        let point = points[p_idx];

        let mut found_new_point = false;

        //
        // 3) Try `k` times to place a new point around the selected point
        //
        for _ in 0..k {
            // Random radius between [min_dist, 2 * min_dist]
            let r1 = rng.gen_range(min_dist..2.0 * min_dist);
            // Random angle in [0, 2π)
            let theta = rng.gen_range(0.0..std::f32::consts::TAU);

            // We'll do the offset in float
            let x_f = point.x as f32 + r1 * theta.cos();
            let y_f = point.y as f32 + r1 * theta.sin();

            // Candidate in integer coordinates
            let candidate_x = x_f as isize;
            let candidate_y = y_f as isize;

            // Check if candidate is inside the bounding rectangle
            if candidate_x < 0 || candidate_x >= width || candidate_y < 0 || candidate_y >= height {
                continue; // skip out-of-bounds
            }

            // Compute the candidate’s cell coords in float, then convert to isize
            let col = (x_f / cell_size) as isize;
            let row = (y_f / cell_size) as isize;

            // We'll check neighboring cells to ensure min distance
            let search_radius = 2; // check neighbors within ±2 cells
            let mut too_close = false;

            for dx in -search_radius..=search_radius {
                for dy in -search_radius..=search_radius {
                    let nx = col + dx;
                    let ny = row + dy;

                    // Skip out-of-bounds cells
                    if nx < 0 || ny < 0 || nx >= cols || ny >= rows {
                        continue;
                    }

                    let neighbor_idx = grid[grid_index(nx, ny)];
                    if neighbor_idx != -1 {
                        // We have a point in this neighbor cell, so check distance
                        let existing_point = points[neighbor_idx as usize];

                        // Convert existing point to float for distance
                        let ex_f = existing_point.x as f32;
                        let ey_f = existing_point.y as f32;

                        let dist_sq = (ex_f - x_f).powi(2) + (ey_f - y_f).powi(2);
                        if dist_sq < min_dist.powi(2) {
                            too_close = true;
                            break;
                        }
                    }
                }
                if too_close {
                    break;
                }
            }

            // If it's sufficiently far from all neighbors, accept it
            if !too_close {
                let new_point = Point2D {
                    x: candidate_x,
                    y: candidate_y,
                };
                points.push(new_point);
                let new_index = points.len() - 1;

                // Mark the grid cell for this new point
                grid[grid_index(col, row)] = new_index as isize;

                // Add to active list
                active_list.push(new_index);
                found_new_point = true;
                break;
            }
        }

        // If we didn't find a new point in k attempts, remove this from active list
        if !found_new_point {
            active_list.remove(rand_i);
        }
    }

    points
}
