pub fn backtrack(
    initial_board: u64,
    placements: &[u64],
    placement_indices: &[usize],
) -> Option<Vec<usize>> {
    let mut selected_placements = Box::<[usize]>::from(placement_indices);
    let mut board_states = vec![initial_board; placement_indices.len() - 1];
    let mut i = 0;
    let mut board = initial_board;

    loop {
        let mut selected_placement = selected_placements[i];
        let bound = selected_placements[i + 1];

        // Find the next valid move
        let placement = loop {
            let placement = placements[selected_placement];
            if placement & board == 0 {
                break placement;
            }

            selected_placement += 1;
            if selected_placement == bound {
                break 0;
            }
        };

        // We ran out of possible moves, so backtrack
        if placement == 0 {
            // The idea here is to walk back our selected placements until we find a piece with a
            // configuration we haven't tried. If no such pieces exist, then we're done and we
            // couldn't find a solution.

            selected_placements[i] = placement_indices[i];

            loop {
                if i == 0 {
                    // We've tried everything
                    return None;
                }

                let bound = placement_indices[i];
                i -= 1;

                // We've tried all configurations for this piece, so check the previous one on the
                // next iteration
                let selected_placement = selected_placements[i];
                if selected_placement == bound {
                    // Reset the selected placement to the first option
                    selected_placements[i] = placement_indices[i];
                }
                // We haven't tried all possibilities for piece i
                else {
                    board = board_states[i];
                    break;
                }
            }

            continue;
        }

        // Store the next configuration we want to check if we backtrack
        selected_placements[i] = selected_placement + 1;

        // Store the current board state so we can back-track
        board_states[i] = board;

        // We'll check the next piece on the next iteration
        i += 1;

        // We're done and we've found a solution
        if i == placement_indices.len() - 1 {
            break;
        }

        // Add the placement to the board
        board |= placement;
    }

    Some(
        selected_placements
            .iter()
            .take(selected_placements.len() - 1)
            .copied()
            .map(|index| index - 1)
            .collect(),
    )
}
