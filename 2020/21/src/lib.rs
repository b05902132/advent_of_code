use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;

pub type Ingredient = String;
pub type Allergen = String;

pub struct Food {
    pub ingredients: HashSet<Ingredient>,
    pub allergens: HashSet<Allergen>,
}

impl std::str::FromStr for Food {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref REGEX: Regex =
                Regex::new(r"^(?P<ingredients>.*).*\(contains (?P<allergens>.*)\)\s*$").unwrap();
            static ref ALLERGEN_PARSE: Regex = Regex::new(r"(\w+)").unwrap();
        }
        let caps = REGEX.captures(s).unwrap();
        let ingredients = caps.name("ingredients").unwrap().as_str();
        let allergens = caps.name("allergens").unwrap().as_str();
        let ingredients = ingredients.split_whitespace().map(str::to_string).collect();
        let allergens = ALLERGEN_PARSE
            .captures_iter(allergens)
            .map(|c| c.get(0).unwrap().as_str().to_string())
            .collect();
        Ok(Self {
            ingredients,
            allergens,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_food() {
        let s = "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)";
        let food = s.parse::<Food>().unwrap();
        assert_eq!(
            food.ingredients,
            "mxmxvkd kfcds sqjhc nhms"
                .split_whitespace()
                .map(str::to_string)
                .collect()
        );
        assert_eq!(
            food.allergens,
            ["dairy", "fish"]
                .into_iter()
                .map(|s| s.to_string())
                .collect()
        )
    }
}
