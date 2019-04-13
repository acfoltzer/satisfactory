type Rate = f64;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Item {
    IronOre,
    IronIngot,
    IronRod,
    IronPlate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Machine {
    MinerMk1,
    Smelter,
    Constructor,
}

#[derive(Clone, Debug)]
struct Recipe {
    inputs: Vec<(Item, Rate)>,
    machine: Machine,
    output: Item,
    output_rate: Rate,
}

fn recipes() -> Vec<Recipe> {
    macro_rules! recipe {
        ( $m:ident [ $( ( $ii:ident, $ir:expr ) ),* ] => ( $oi:ident, $or:expr ) ) => {
            Recipe {
                inputs: vec![ $( (Item::$ii, $ir) ),* ],
                machine: Machine::$m,
                output: Item::$oi,
                output_rate: $or,
            }
        };
    }
    vec![
        recipe!(MinerMk1[] => (IronOre, 60.0)),
        recipe!(Smelter[(IronOre, 30.0)] => (IronIngot, 30.0)),
        recipe!(Constructor[(IronIngot, 15.0)] => (IronRod, 15.0)),
        recipe!(Constructor[(IronIngot, 30.0)] => (IronPlate, 15.0)),
    ]
}

fn recipes_to_make(item: Item, rate: Rate) -> Vec<(Recipe, Rate)> {
    let mut resources = vec![];
    let mut todo = vec![(item, rate)];
    while let Some((item, rate)) = todo.pop() {
        for r in recipes().iter() {
            if r.output == item {
                let repetitions = rate / r.output_rate;
                for &(ii, ir) in r.inputs.iter() {
                    todo.push((ii, ir * repetitions));
                }
                resources.push((r.clone(), rate));
                continue;
            }
        }
    }
    resources
}

fn assign_machines(spec: &[(Recipe, Rate)]) -> Vec<(Machine, Rate)> {
    let mut assignment = vec![];
    for &(ref r, rate) in spec {
        let machines_required = (rate / r.output_rate).ceil() as usize;
        let rate_per_machine = rate / machines_required as f64;
        for _ in 0..machines_required {
            assignment.push((r.machine, rate_per_machine));
        }
    }
    assignment
}

fn main() {
    println!(
        "In order to make 60 Iron Rods, you need: {:?}",
        assign_machines(&recipes_to_make(Item::IronRod, 60.0))
    );
    println!(
        "In order to make 60 Iron Plates, you need: {:?}",
        assign_machines(&recipes_to_make(Item::IronPlate, 60.0))
    );
}
