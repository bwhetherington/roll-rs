mod roll;
use rand::prelude::*;
use roll::{Keep, Roll};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MACROS: HashMap<String, Vec<Roll>> = {
        let mut map = HashMap::new();

        map.insert(
            String::from("adv"),
            vec![Roll::new(2, 20, None, Some(Keep::High(1)), None)],
        );
        map.insert(
            String::from("dis"),
            vec![Roll::new(2, 20, None, Some(Keep::Low(1)), None)],
        );
        map.insert(
            String::from("stats"),
            vec![
                Roll::new(4, 6, None, Some(Keep::High(3)), None),
                Roll::new(4, 6, None, Some(Keep::High(3)), None),
                Roll::new(4, 6, None, Some(Keep::High(3)), None),
                Roll::new(4, 6, None, Some(Keep::High(3)), None),
                Roll::new(4, 6, None, Some(Keep::High(3)), None),
                Roll::new(4, 6, None, Some(Keep::High(3)), None),
            ],
        );

        map
    };
}

struct Context {
    macros: HashMap<String, Vec<Roll>>,
}

impl Context {
    fn new() -> Context {
        Context {
            macros: HashMap::new(),
        }
    }

    fn load_macros(&mut self) {
        let macro_file = include_str!("../macros.txt");

        for line in macro_file.lines() {
            let mut iter = line.split_whitespace();
            let name = iter.next().unwrap();
            let rolls = iter.map(|roll| roll.to_string());
            let rolls = self.parse_rolls(rolls).expect("Parsing error.");
            self.macros.insert(name.to_string(), rolls);
        }
    }

    fn parse_rolls(&self, args: impl Iterator<Item = String>) -> Result<Vec<Roll>, &'static str> {
        let mut rolls: Vec<Roll> = vec![];
        for arg in args {
            // Look it up in macros
            if let Some(sub_rolls) = self.macros.get(&arg) {
                for roll in sub_rolls {
                    rolls.push(roll.clone());
                }
            } else {
                // Try to parse it
                let roll = arg.parse()?;
                rolls.push(roll);
            }
        }

        Ok(rolls)
    }

    fn process_rolls(&self, rolls: Vec<Roll>) {
        let mut rng = thread_rng();
        let mut total = 0;
        for roll in rolls.iter() {
            let outcome = roll.roll(&mut rng);
            total += outcome.total();
            println!(
                "{}: {} (Expected: {})",
                roll,
                outcome,
                roll.expected_total()
            );
        }
        if rolls.len() > 1 {
            println!("Total: {}", total);
        }
    }
}

fn main() {
    let mut context = Context::new();
    context.load_macros();
    match context.parse_rolls(env::args().skip(1)) {
        Ok(rolls) => context.process_rolls(rolls),
        Err(why) => println!("Error: {}", why),
    }
}
