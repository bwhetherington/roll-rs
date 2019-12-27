use rand::prelude::*;
use regex::Regex;
use std::{fmt, str};

pub const REGEX_STR: &'static str =
    r"(?P<num>[0-9]*)d(?P<die>[0-9]+)(r(?P<reroll>[0-9]+))?((?P<high_or_low>[hl])(?P<keep>[0-9]+))?(?P<modifier>[\+\-][0-9]+)?";

lazy_static! {
    static ref REGEX: Regex = Regex::new(REGEX_STR).unwrap();
}

#[derive(Clone, Debug)]
pub enum Keep {
    High(usize),
    Low(usize),
}

#[derive(Clone, Debug)]
pub struct Outcome {
    rolls: Vec<DieRoll>,
    modifier: i32,
    keep: Option<Keep>,
}

#[derive(Clone, Debug)]
pub enum DieRoll {
    Kept(u32),
    Rerolled(u32, u32),
}

impl fmt::Display for DieRoll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DieRoll::Kept(n) => write!(f, "{}", n),
            DieRoll::Rerolled(old, new) => write!(f, "{}=>{}", old, new),
        }
    }
}

impl DieRoll {
    pub fn value(&self) -> u32 {
        match self {
            DieRoll::Kept(n) => *n,
            DieRoll::Rerolled(_, n) => *n,
        }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.total())?;
        let rolls: Vec<_> = self.rolls.iter().map(|roll| roll.to_string()).collect();
        let rolls = rolls.join(", ");
        write!(f, "({})", rolls)?;
        if self.modifier > 0 {
            write!(f, " + {}", self.modifier)
        } else if self.modifier < 0 {
            write!(f, " - {}", -self.modifier)
        } else {
            Ok(())
        }
    }
}

impl Outcome {
    pub fn new(mut rolls: Vec<DieRoll>, keep: Option<Keep>, modifier: i32) -> Outcome {
        rolls.sort_by(|a, b| a.value().cmp(&b.value()));
        Outcome {
            rolls,
            keep,
            modifier,
        }
    }

    /// Computes the total value of the roll outcome.
    pub fn total(&self) -> i32 {
        let range = match &self.keep {
            Some(Keep::High(n)) => &self.rolls[self.rolls.len() - n..],
            Some(Keep::Low(n)) => &self.rolls[..*n],
            None => &self.rolls[..],
        };
        range.into_iter().map(|roll| roll.value()).sum::<u32>() as i32 + self.modifier
    }
}

#[derive(Clone, Debug)]
pub struct Roll {
    num: u32,
    die: u32,
    reroll: Option<u32>,
    modifier: Option<i32>,
    keep: Option<Keep>,
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.num > 1 {
            write!(f, "{}", self.num)?;
        }

        write!(f, "d{}", self.die)?;

        if let Some(n) = self.reroll {
            write!(f, "r{}", n)?;
        }

        if let Some(keep) = &self.keep {
            match keep {
                Keep::High(n) => {
                    write!(f, "h{}", n)?;
                }
                Keep::Low(n) => {
                    write!(f, "l{}", n)?;
                }
            }
        }

        if let Some(modifier) = self.modifier {
            if modifier != 0 {
                write!(f, "{:+}", modifier)?;
            }
        }

        Ok(())
    }
}

impl Default for Roll {
    fn default() -> Roll {
        Roll {
            num: 1,
            die: 0,
            reroll: None,
            modifier: None,
            keep: None,
        }
    }
}

impl str::FromStr for Roll {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Roll, Self::Err> {
        if let Some(cap) = REGEX.captures(input) {
            let mut roll = Roll::default();
            if let Some(num) = cap.name("num") {
                let num_str = &input[num.start()..num.end()];
                if num_str.len() > 0 {
                    let num_parsed = num_str
                        .parse::<u32>()
                        .map_err(|_| "Failed to parse number of dice.")?;
                    roll.num = num_parsed;
                }
            }
            if let Some(die) = cap.name("die") {
                let die_str = &input[die.start()..die.end()];
                let die_parsed = die_str
                    .parse::<u32>()
                    .map_err(|_| "Failed to parse die size.")?;
                roll.die = die_parsed;
            } else {
                return Err("No die specified.");
            }
            if let Some(reroll) = cap.name("reroll") {
                let reroll_str = &input[reroll.start()..reroll.end()];
                let reroll_parsed = reroll_str
                    .parse::<u32>()
                    .map_err(|_| "Failed to parse reroll.")?;
                roll.reroll = Some(reroll_parsed);
            }
            if let Some(modifier) = cap.name("modifier") {
                let mod_str = &input[modifier.start()..modifier.end()];
                let mod_parsed = mod_str
                    .parse::<i32>()
                    .map_err(|_| "Failed to parse modifier.")?;
                roll.modifier = Some(mod_parsed);
            }
            if let Some(high_or_low) = cap.name("high_or_low") {
                let hol_str = &input[high_or_low.start()..high_or_low.end()];
                let is_high = match hol_str {
                    "h" => true,
                    "l" => false,
                    _ => {
                        return Err("Error parsing high or low.");
                    }
                };
                if let Some(keep_amount) = cap.name("keep") {
                    let keep_str = &input[keep_amount.start()..keep_amount.end()];
                    let keep_parsed = keep_str
                        .parse::<usize>()
                        .map_err(|_| "Error parsing number or dice to keep.")?;
                    let keep = if is_high {
                        Keep::High(keep_parsed)
                    } else {
                        Keep::Low(keep_parsed)
                    };
                    roll.keep = Some(keep);
                }
            }
            Ok(roll)
        } else {
            println!("{}", input);
            Err("Something went wrong.")
        }
    }
}

fn expected_roll(die: u32, reroll: Option<u32>) -> f64 {
    let reroll = reroll.unwrap_or(die + 1);
    let avg = (die as f64 / 2.0) + 0.5;
    let total = (1..=die)
        .map(|n| if n <= reroll { avg } else { n as f64 })
        // .map(|n| {
        //     println!("out: {} ({})", n, avg);
        //     n
        // })
        .sum::<f64>();
    total / (die as f64)
}

impl Roll {
    pub fn new(
        num: u32,
        die: u32,
        reroll: Option<u32>,
        keep: Option<Keep>,
        modifier: Option<i32>,
    ) -> Roll {
        Roll {
            num,
            die,
            reroll,
            keep,
            modifier,
        }
    }

    fn base_roll(&self, mut rng: impl Rng) -> u32 {
        rng.gen_range(0, self.die) + 1
    }

    pub fn expected_total(&self) -> f64 {
        let num_dice = self
            .keep
            .as_ref()
            .map(|keep| match keep {
                Keep::High(n) => *n,
                Keep::Low(n) => *n,
            })
            .unwrap_or(self.num as usize) as f64;
        expected_roll(self.die, self.reroll) * num_dice + (self.modifier.unwrap_or(0) as f64)
    }

    pub fn roll(&self, mut rng: impl Rng) -> Outcome {
        let mut rolls = Vec::with_capacity(self.num as usize);

        // Roll the dice
        for _ in 0..self.num {
            // Check if we need to reroll
            let original_roll = self.base_roll(&mut rng);
            let roll = self
                .reroll
                .map(|reroll| {
                    if original_roll <= reroll {
                        DieRoll::Rerolled(original_roll, self.base_roll(&mut rng))
                    } else {
                        DieRoll::Kept(original_roll)
                    }
                })
                .unwrap_or_else(|| DieRoll::Kept(original_roll));

            // Add the roll
            rolls.push(roll);
        }

        Outcome::new(rolls, self.keep.clone(), self.modifier.unwrap_or(0))
    }
}
