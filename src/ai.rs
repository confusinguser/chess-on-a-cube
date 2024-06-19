use crate::cell::*;
use crate::gamemanager::*;
use crate::movement::*;
use crate::units::*;

#[derive(Default)]
pub(crate) struct AICache {
    last_variation: Vec<GameMove>,
}

pub(crate) fn next_move(
    board: &Board,
    units: &Units,
    team: Team,
    depth: u32,
    ai_cache: &mut AICache,
) -> GameMove {
    next_move_internal(
        &mut board.clone(),
        &mut units.clone(),
        team,
        depth,
        ai_cache,
    )
}

fn next_move_internal(
    board: &mut Board,
    units: &mut Units,
    team: Team,
    depth: u32,
    ai_cache: &mut AICache,
) -> GameMove {
    let mut stats = (0, 0, 0);
    let mut variation = eval_recursive(
        board,
        units,
        team,
        depth,
        f32::MIN,
        f32::MAX,
        &mut stats,
        true,
        ai_cache,
    );

    variation.1.pop().unwrap()
}

#[allow(clippy::too_many_arguments)]
fn eval_recursive(
    board: &mut Board,
    units: &mut Units,
    team: Team,
    depth: u32,
    mut alpha: f32,
    beta: f32,
    stats: &mut (u32, u32, u32),
    og: bool,
    ai_cache: &mut AICache,
) -> (f32, Vec<GameMove>) {
    let (_, _, ref mut num_nodes) = stats;
    *num_nodes += 1;
    if depth == 0 {
        let eval = evaluation(board, units, ai_cache) * team.sign() as f32;
        return (eval, Vec::new());
    }

    let mut eval = f32::MIN;
    let mut best_variation: Vec<GameMove> = Vec::new();
    let possible_moves = get_possible_moves(board, units, team);
    let possible_moves = sort_moves(possible_moves, board, units, team, ai_cache);
    for game_move in possible_moves {
        let (made_move, captured_unit) = make_move(game_move.0, units);
        if !made_move {
            continue;
        }

        let (eval_next, best_variation_returned) = eval_recursive(
            board,
            units,
            team.opposite(),
            depth - 1,
            -beta,
            -alpha,
            stats,
            false,
            ai_cache,
        );
        unmake_move(game_move.0, units, captured_unit);

        if eval_next > eval {
            eval = eval_next;
            best_variation.clone_from(&best_variation_returned);
            if og {
                ai_cache.last_variation = best_variation_returned;
            }
            best_variation.push(game_move.0);
        }

        alpha = alpha.max(eval);
        if alpha >= beta {
            let (ref mut a, ref mut b, _) = stats;
            if team == Team::Black {
                *a += 1;
            } else {
                *b += 1;
            }

            // break;
        }
    }
    (eval, best_variation)
}

fn sort_moves(
    possible_moves: Vec<GameMove>,
    board: &Board,
    units: &mut Units,
    team: Team,
    ai_cache: &mut AICache,
) -> Vec<(GameMove, i32, f32)> {
    let mut output = Vec::new();
    for possible_move in possible_moves.into_iter() {
        let (move_made, captured_unit) = make_move(possible_move, units);
        if !move_made {
            continue;
        }

        let eval = evaluation(board, units, ai_cache) * team.sign() as f32;
        unmake_move(possible_move, units, captured_unit);

        if ai_cache
            .last_variation
            .last()
            .map_or(false, |&game_move| game_move == possible_move)
        {
            output.push((possible_move, 2, eval));
            continue;
        }

        let is_capture = units.is_unit_at(possible_move.to);
        if is_capture {
            output.push((possible_move, 1, eval));
            continue;
        }

        output.push((possible_move, 0, eval));
    }

    output.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    output.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Sorts list so largest is first
                                                           // output
                                                           //     .into_iter()
                                                           //     .map(|possible_move| possible_move.0)
                                                           //     .collect()
    output
}

fn get_possible_moves(board: &Board, units: &Units, team: Team) -> Vec<GameMove> {
    let mut output = Vec::new();
    for unit in units.all_units_iter() {
        if unit.team != team {
            continue;
        }
        for move_to in get_unit_moves(unit, board, units) {
            output.push(GameMove {
                from: unit.coords,
                to: move_to,
            })
        }
    }
    output
}

fn evaluation(_board: &Board, units: &Units, _ai_cache: &mut AICache) -> f32 {
    let mut white_material = 0.;
    let mut black_material = 0.;

    for unit in units.all_units_iter() {
        match unit.team {
            Team::Black => {
                black_material += unit.unit_type.material_value();
            }
            Team::White => {
                white_material += unit.unit_type.material_value();
            }
        }
    }

    white_material - black_material
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
