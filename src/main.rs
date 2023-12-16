use indicatif::ParallelProgressIterator;
use rand::distributions::{Standard, Uniform};
use rand::prelude::*;
use rayon::prelude::*;

#[derive(Debug)]
enum DieRoll {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
    Basket = 4,
    Bird = 5,
}

impl Distribution<DieRoll> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DieRoll {
        match Uniform::new(0, 6).sample(rng) {
            0 => DieRoll::Red,
            1 => DieRoll::Green,
            2 => DieRoll::Blue,
            3 => DieRoll::Yellow,
            4 => DieRoll::Basket,
            5 => DieRoll::Bird,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Game {
    /// Tracks how close the bird is to the orchard; at 0, the player(s) lose(s).
    ///
    /// There are five tiles included in the game box; to allow for 'difficulty' adjustment this
    /// value generally spans four to six:
    ///  * Starts on tile 1, lose when arriving at tile 5,
    ///  * Starts on tile 1, lose when moving 'off' tile 5,
    ///  * Starts on tile '0', lose when moving 'off' tile 5.
    bird_position: u8,

    /// Number of apples left in each orchard: [red, green, blue, yellow].
    orchards: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Outcome {
    Won,
    Lost,
}

impl Game {
    fn new(bird_position: u8) -> Game {
        Game {
            bird_position,
            orchards: [4, 4, 4, 4],
        }
    }

    fn apply(&mut self, roll: DieRoll) -> Option<Outcome> {
        match roll {
            DieRoll::Red | DieRoll::Green | DieRoll::Blue | DieRoll::Yellow => {
                let i = roll as usize;
                self.orchards[i] = self.orchards[i].saturating_sub(1);
            }
            DieRoll::Basket => {
                // Without further justification, we assume the optimal strategy is to decrement
                // the largest remaining orchard's count.
                //
                // Unwrap: we always have four orchards.
                let largest_pile = self.orchards.iter_mut().max().unwrap();
                *largest_pile = largest_pile.saturating_sub(1);
            }
            DieRoll::Bird => self.bird_position = self.bird_position.saturating_sub(1),
        }

        if self.bird_position == 0 {
            Some(Outcome::Lost)
        } else if self.orchards.iter().sum::<u8>() == 0 {
            Some(Outcome::Won)
        } else {
            None
        }
    }

    fn full_game(bird_position: u8, rng: &mut impl Rng) -> Outcome {
        let mut game = Game::new(bird_position);
        loop {
            let roll: DieRoll = rng.gen();
            if let Some(outcome) = game.apply(roll) {
                return outcome;
            }
        }
    }
}

fn estimate_win_rate(bird_position: u8) {
    let n = 1_000_000_000;
    let (won, lost) = (0..n)
        .into_par_iter()
        .progress_count(n)
        .map(|_| {
            let mut rng = rand::thread_rng();
            match Game::full_game(bird_position, &mut rng) {
                Outcome::Won => (1, 0),
                Outcome::Lost => (0, 1),
            }
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    let win_rate_percent = 100.0 * won as f64 / (won + lost) as f64;
    println!("Won {won}, lost {lost}, win rate {win_rate_percent:.2}%");
}

fn main() {
    println!("Estimating win rate for 'easy' mode (start pos = 6)...");
    estimate_win_rate(6);

    println!("Estimating win rate for 'normal' mode (start pos = 5)...");
    estimate_win_rate(5);

    println!("Estimating win rate for 'hard' mode (start pos = 4)...");
    estimate_win_rate(4);
}
