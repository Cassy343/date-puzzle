fn main() {
    println!("Hello, world!");
}

fn backtrack(
    initial_board: u64,
    placements: &[u64],
    placement_indices: &[usize],
) -> Option<Box<[usize]>> {
    let mut selected_placements = Box::from(placement_indices);
    let mut i = 0;
    let mut board = initial_board;

    loop {
        let placement = placements[selected_placements[i]];
        selected_placements[i] += 1;
    }

    None
}