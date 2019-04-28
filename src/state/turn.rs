#[derive(Clone, Copy, Debug)]
pub enum GamePhase {
    NotStarted,
    Playing(Turn),
    GameOver,
}

impl GamePhase {
    pub fn turn(&self) -> Option<&Turn> {
        match self {
            GamePhase::Playing(turn) => Some(turn),
            _ => None,
        }
    }
}

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
        &self,
        num_players: usize,
        remaining_players: &mut dyn Iterator<Item = usize>,
    ) -> GamePhase {
        if num_players == 0 {
            return GamePhase::GameOver;
        }

        let next_turn = Turn::new((self.current_player + 1) % num_players);
        let next_phase = next_turn.skip_eliminated_players(num_players, remaining_players);
        match next_phase {
            GamePhase::Playing(Turn { current_player, .. }) => {
                if current_player == self.current_player {
                    GamePhase::GameOver
                } else {
                    next_phase
                }
            }
            _ => next_phase,
        }
    }

    pub fn skip_eliminated_players(
        &self,
        num_players: usize,
        remaining_players: &mut dyn Iterator<Item = usize>,
    ) -> GamePhase {
        if num_players == 0 {
            return GamePhase::GameOver;
        }

        let skip = remaining_players
            .filter(|p| *p < num_players) // Sanity check
            .map(|p| (p + num_players - self.current_player) % num_players) // Number of steps to this player
            .min();
        match skip {
            Some(skip) => {
                let mut next_turn = *self;
                next_turn.current_player = (next_turn.current_player + skip) % num_players;
                GamePhase::Playing(next_turn)
            }
            None => GamePhase::GameOver,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TurnState {
    Aiming,
    Firing,
}
