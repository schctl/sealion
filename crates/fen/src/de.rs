//! Fen parser implementation.
//!
//! <https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation>

use std::str::FromStr;

use nom::bytes::complete::{is_not, take_till};
use nom::character::complete::{digit1, one_of, space0, space1};
use nom::character::{is_digit, is_space};
use nom::combinator::map_res;
use nom::multi::{many0, many1};
use nom::sequence::Tuple;
use nom::IResult;

use sealion_board::{Board, CastlingRights, Color, Piece, Position, Square};

fn parse_board(mut input: &str) -> IResult<&str, Board> {
    let mut board = Board::default();
    let mut square = Square::at(7, 0).unwrap();

    for _ in 0..8 {
        let (new_input, rank) = take_till(|c| c == '/' || is_space(c as u8))(input)?;
        let (new_input, _) = many0(one_of("/"))(new_input)?;

        for char in rank.chars() {
            // FIXME: add some checks to increment square index
            if is_digit(char as u8) {
                *square.raw_index_mut() += (char as u8).saturating_sub(b'0');
                continue;
            }

            board.set(square, Piece::from_char(char));
            *square.raw_index_mut() += 1;
        }

        input = new_input;
        *square.raw_index_mut() = square.raw_index().saturating_sub(16);
    }

    Ok((input, board))
}

fn parse_active_color(input: &str) -> IResult<&str, Color> {
    let (input, active_color) = one_of("wbWB")(input)?;

    let active_color = match active_color {
        'w' | 'W' => Color::White,
        'b' | 'B' => Color::Black,
        _ => unreachable!(),
    };

    Ok((input, active_color))
}

fn parse_castling_rights(input: &str) -> IResult<&str, CastlingRights> {
    let (input, castle_str) = many1(one_of("KQkq-"))(input)?;
    let mut castling_rights = CastlingRights::empty();

    if castle_str.contains(&'K') {
        castling_rights |= CastlingRights::WHITE_OO;
    }
    if castle_str.contains(&'Q') {
        castling_rights |= CastlingRights::WHITE_OOO;
    }
    if castle_str.contains(&'k') {
        castling_rights |= CastlingRights::BLACK_OO;
    }
    if castle_str.contains(&'q') {
        castling_rights |= CastlingRights::BLACK_OOO;
    }

    Ok((input, castling_rights))
}

fn parse_ep_target(input: &str) -> IResult<&str, Option<Square>> {
    let (input, ep_target) = is_not(" \t\r\n")(input)?;

    let ep_target = if ep_target == "-" {
        None
    } else {
        Square::from_str(ep_target).ok()
    };

    Ok((input, ep_target))
}

fn parse_u8(input: &str) -> IResult<&str, u8> {
    map_res(digit1, str::parse)(input)
}

/// Parse a chessboard state from the provided FEN string.
pub fn parse(input: &str) -> IResult<&str, Position> {
    let (
        input,
        (
            _,
            board,
            _,
            active_color,
            _,
            castling,
            _,
            ep_target,
            _,
            halfmove_clock,
            _,
            fullmove_counter,
        ),
    ) = (
        space0,
        parse_board,
        space1,
        parse_active_color,
        space1,
        parse_castling_rights,
        space1,
        parse_ep_target,
        space1,
        parse_u8,
        space1,
        parse_u8,
    )
        .parse(input)?;

    Ok((
        input,
        Position {
            board,
            active_color,
            castling,
            ep_target,
            halfmove_clock,
            fullmove_counter,
        },
    ))
}
