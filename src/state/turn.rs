#[derive(Clone, Copy, Debug)]
pub struct Turn {
    pub current_player: usize,
    pub state: TurnState,
}

impl Turn {
    pub fn new(current_player: usize) -> Turn {
        Turn {
            current_player,
            state: TurnState::Aiming,
        }
    }

    pub fn next_player(
        turn: &Option<Turn>,
        num_players: usize,
        remaining_players: &mut dyn Iterator<Item = usize>,
    ) -> Option<Turn> {
        if num_players == 0 {
            return None;
        }

        let next_player = match turn {
            Some(Turn { current_player, .. }) => (*current_player + 1) % num_players,
            None => 0,
        };
        let next_player = remaining_players
            .filter(|p| *p < num_players) // Sanity check
            .map(|p| (p + num_players - next_player) % num_players) // Number of steps to this player
            .min()
            .map(|offset| next_player + offset);
        Some(Turn::new(next_player?))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TurnState {
    Aiming,
    Firing,
}
