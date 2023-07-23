use crate::gamemanager::*;
use crate::movement::*;
use crate::units::*;
use crate::{cell::*, movement};

pub(crate) fn next_move(board: &Board, units: &Units, team: Team, depth: u32) -> GameMove {
    next_move_internal(&mut board.clone(), &mut units.clone(), team, depth)
}

fn next_move_internal(board: &mut Board, units: &mut Units, team: Team, depth: u32) -> GameMove {
    eval_recursive(board, units, team, depth, f32::MIN, f32::MAX)
        .1
        .unwrap()
}

fn eval_recursive(
    board: &mut Board,
    units: &mut Units,
    team: Team,
    depth: u32,
    mut alpha: f32,
    mut beta: f32,
) -> (f32, Option<GameMove>) {
    if depth == 0 {
        let eval = eval(board, units);
        return (eval, None);
    }

    let mut eval = if team == Team::White {
        f32::MIN
    } else {
        f32::MAX
    };
    let mut best_move: Option<GameMove> = None;
    for game_move in get_possible_moves(board, units, team) {
        let (made_move, captured_unit) = make_move(game_move, units);
        if !made_move {
            continue;
        }

        let (eval_next, _) = eval_recursive(board, units, team.opposite(), depth - 1, alpha, beta);
        if (team == Team::White && eval_next > eval) || (team == Team::Black && eval_next < eval) {
            dbg!(best_move);
            eval = eval_next;
            best_move = Some(game_move);
        }

        if team == Team::White {
            if eval > beta {
                break;
            }
            dbg!(eval);
            alpha = alpha.max(eval);
        } else {
            if eval < alpha {
                println!("brk alph");
                dbg!(beta, alpha, eval);
                break;
            }
            beta = beta.min(eval);
        }

        unmake_move(game_move, units, captured_unit)
    }
    (eval, best_move)
}

fn get_possible_moves(board: &Board, units: &Units, team: Team) -> Vec<GameMove> {
    let mut output = Vec::new();
    for unit in units.all_units_iter() {
        if unit.team != team {
            continue;
        }
        for move_to in movement::get_unit_moves(unit, board, units) {
            output.push(GameMove {
                from: unit.coords,
                to: move_to,
            })
        }
    }
    output
}

fn eval(board: &Board, units: &Units) -> f32 {
    let mut white_material = 0;
    let mut black_material = 0;

    for unit in units.all_units_iter() {
        let material = 1;
        match unit.team {
            Team::Black => {
                black_material += material;
            }
            Team::White => {
                white_material += material;
            }
        }
    }

    (white_material - black_material) as f32
}

fn make_move(game_move: GameMove, units: &mut Units) -> (bool, Option<Unit>) {
    let captured_unit = units.remove_unit(game_move.to);
    let Some(unit) = units.get_unit_mut(game_move.from) else {
        return (false, None);
    };
    unit.move_unit_to(game_move.to);
    (true, captured_unit)
}

fn unmake_move(game_move: GameMove, units: &mut Units, captured_unit: Option<Unit>) {
    let Some(unit) = units.get_unit_mut(game_move.to) else {
        panic!("Couldn't undo move: {:?}, units: {:?}", game_move, units);
    };
    unit.move_unit_to(game_move.from);
    if let Some(captured_unit) = captured_unit {
        units.add_unit(captured_unit);
    }
}
