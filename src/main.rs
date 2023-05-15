use sealion_board::{Position, Square};
use sealion_maven::Maven;

fn main() {
    let start = Position::starting();
    let sq = Square::at(4, 5).unwrap();

    let result = Maven::bishop_moves(
        sq,
        start.board.get_color_bb(start.active_color),
        start.board.get_color_bb(start.active_color.opposite()),
    );

    assert_eq!(result, 0x88_50_00_50_88_00_00);
    println!("{result}");
}
