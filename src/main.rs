mod roll;
use rand::prelude::*;
use roll::{Keep, Roll};
use std::{collections::HashMap, env};

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

fn parse_rolls(args: impl Iterator<Item = String>) -> Result<Vec<Roll>, &'static str> {
    let mut rolls: Vec<Roll> = vec![];
    for arg in args.skip(1) {
        // Look it up in macros
        if let Some(sub_rolls) = MACROS.get(&arg) {
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

fn process_rolls(rolls: Vec<Roll>) {
    let mut rng = thread_rng();
    for roll in rolls {
        println!("{}", roll.roll(&mut rng));
    }
}

fn main() {
    match parse_rolls(env::args()) {
        Ok(rolls) => process_rolls(rolls),
        Err(why) => println!("Error: {}", why),
    }
}
