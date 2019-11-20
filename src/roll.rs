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
pub struct Roll {
    num: u32,
    die: u32,
    reroll: Option<u32>,
    modifier: Option<i32>,
    keep: Option<Keep>,
}

#[derive(Clone, Debug)]
pub struct Outcome {
    rolls: Vec<u32>,
    modifier: i32,
    keep: Option<Keep>,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} ", &self.rolls)?;
        if self.modifier > 0 {
            write!(f, "+ {} ", self.modifier)?;
        } else if self.modifier < 0 {
            write!(f, "- {} ", -self.modifier)?;
        }
        write!(f, "= {}", self.total())
    }
}

impl Outcome {
    pub fn new(mut rolls: Vec<u32>, keep: Option<Keep>, modifier: i32) -> Outcome {
        rolls.sort();
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
        range.into_iter().sum::<u32>() as i32 + self.modifier
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
            Err("Something went wrong.")
        }
    }
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

    pub fn roll(&self, mut rng: impl Rng) -> Outcome {
        let mut rolls = Vec::with_capacity(self.num as usize);

        // Roll the dice
        for _ in 0..self.num {
            let mut roll = self.base_roll(&mut rng);

            // Check if we need to reroll
            if let Some(reroll) = self.reroll {
                if roll <= reroll {
                    roll = self.base_roll(&mut rng);
                }
            }

            // Add the roll
            rolls.push(roll);
        }

        Outcome::new(rolls, self.keep.clone(), self.modifier.unwrap_or(0))
    }
}
