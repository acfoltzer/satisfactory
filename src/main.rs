use petgraph::dot::Dot;
use petgraph::prelude::{Bfs, Direction, Graph, NodeIndex};
use petgraph::visit::Reversed;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type Rate = f64;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Item {
    AILimiter,
    AlienCarapace,
    Any,
    Biofuel,
    Biomass,
    Cable,
    CateriumIngot,
    CateriumOre,
    CircuitBoard,
    Coal,
    ColorCartridge,
    Computer,
    Concrete,
    CopperIngot,
    CopperOre,
    CrudeOil,
    EncasedIndustrialBeam,
    Fabric,
    Filter,
    FlowerPetals,
    Fuel,
    GreenPowerSlug,
    HeavyModularFrame,
    HighSpeedConnector,
    IronIngot,
    IronOre,
    IronPlate,
    IronRod,
    Leaves,
    Limestone,
    ModularFrame,
    Motor,
    Mycelia,
    Plastic,
    PowerShard,
    PurplePowerSlug,
    Quickwire,
    ReinforcedIronPlate,
    Rotor,
    Rubber,
    Screw,
    SpikedRebar,
    Stator,
    SteelBeam,
    SteelIngot,
    SteelPipe,
    Supercomputer,
    Wire,
    Wood,
    YellowPowerSlug,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Machine {
    Assembler,
    CateriumMinerMk1,
    CateriumMinerMk2,
    Constructor,
    CopperMinerMk1,
    CopperMinerMk2,
    Foundry,
    IronMinerMk1,
    IronMinerMk2,
    Manufacturer,
    OilPump,
    OilRefinery,
    Smelter,
    Storage,
}

#[derive(Clone, Debug)]
struct Recipe {
    inputs: Vec<(Item, Rate)>,
    machine: Machine,
    output: Item,
    output_rate: Rate,
}

impl Eq for Recipe {}

impl PartialEq for Recipe {
    fn eq(&self, other: &Recipe) -> bool {
        self.inputs
            .iter()
            .zip(other.inputs.iter())
            .all(|(i1, i2)| i1.0 == i2.0 && (i1.1 as usize) == (i2.1 as usize))
            && self.machine == other.machine
            && self.output == other.output
            && (self.output_rate as usize) == (other.output_rate as usize)
    }
}

impl Hash for Recipe {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in self.inputs.iter() {
            i.0.hash(state);
            (i.1 as usize).hash(state)
        }
        self.machine.hash(state);
        self.output.hash(state);
        (self.output_rate as usize).hash(state);
    }
}

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

fn recipes() -> Vec<Recipe> {
    vec![
        recipe!(Storage[(Any, 900.0)] => (Any, 900.0)),
        recipe!(IronMinerMk1[] => (IronOre, 60.0)),
        recipe!(CopperMinerMk1[] => (CopperOre, 60.0)),
        recipe!(CateriumMinerMk1[] => (CateriumOre, 60.0)),
        recipe!(IronMinerMk2[] => (IronOre, 120.0)),
        recipe!(CopperMinerMk2[] => (CopperOre, 120.0)),
        recipe!(CateriumMinerMk2[] => (CateriumOre, 120.0)),
        recipe!(Smelter[(IronOre, 30.0)] => (IronIngot, 30.0)),
        recipe!(Smelter[(CopperOre, 30.0)] => (CopperIngot, 30.0)),
        recipe!(Smelter[(CateriumOre, 60.0)] => (CateriumIngot, 15.0)),
        recipe!(Foundry[(Coal, 45.0), (IronOre, 45.0)] => (SteelIngot, 30.0)),
        recipe!(Constructor[(IronIngot, 30.0)] => (IronPlate, 15.0)),
        recipe!(Constructor[(IronIngot, 15.0)] => (IronRod, 15.0)),
        recipe!(Constructor[(CopperIngot, 15.0)] => (Wire, 45.0)),
        recipe!(Constructor[(Wire, 30.0)] => (Cable, 15.0)),
        recipe!(Constructor[(Leaves, 150.0)] => (Biomass, 90.0)),
        recipe!(Constructor[(Limestone, 45.0)] => (Concrete, 15.0)),
        recipe!(Constructor[(IronRod, 15.0)] => (Screw, 90.0)),
        recipe!(Constructor[(Wood, 75.0)] => (Biomass, 375.0)),
        recipe!(Constructor[(GreenPowerSlug, 6.0)] => (PowerShard, 6.0)),
        recipe!(Constructor[(Biomass, 60.0)] => (Biofuel, 30.0)),
        recipe!(Constructor[(SteelIngot, 30.0)] => (SteelBeam, 10.0)),
        recipe!(Constructor[(SteelIngot, 15.0)] => (SteelPipe, 15.0)),
        recipe!(Constructor[(Mycelia, 150.0)] => (Biomass, 150.0)),
        recipe!(Constructor[(FlowerPetals, 37.5)] => (ColorCartridge, 75.0)),
        recipe!(Constructor[(IronRod, 15.0)] => (SpikedRebar, 15.0)),
        recipe!(Constructor[(AlienCarapace, 15.0)] => (Biomass, 1500.0)),
        recipe!(Constructor[(YellowPowerSlug, 4.0)] => (PowerShard, 8.0)),
        recipe!(Constructor[(CateriumIngot, 15.0)] => (Quickwire, 60.0)),
        recipe!(Constructor[(PurplePowerSlug, 3.0)] => (PowerShard, 15.0)),
        recipe!(Assembler[(IronPlate, 20.0), (Screw, 120.0)] => (ReinforcedIronPlate, 5.0)),
        recipe!(Assembler[(IronRod, 18.0), (Screw, 132.0)] => (Rotor, 6.0)),
        recipe!(Assembler[(ReinforcedIronPlate, 12.0), (IronRod, 24.0)] => (ModularFrame, 4.0)),
        recipe!(Assembler[(SteelBeam, 16.0), (Concrete, 20.0)] => (EncasedIndustrialBeam, 4.0)),
        recipe!(Assembler[(SteelPipe, 18.0), (Wire, 60.0)] => (Stator, 6.0)),
        recipe!(Assembler[(Rotor, 10.0), (Stator, 10.0)] => (Motor, 5.0)),
        recipe!(Assembler[(Mycelia, 15.0), (Leaves, 30.0)] => (Fabric, 15.0)),
        recipe!(Assembler[(Wire, 60.0), (Plastic, 30.0)] => (CircuitBoard, 5.0)),
        recipe!(Assembler[(CircuitBoard, 5.0), (Quickwire, 90.0)] => (AILimiter, 5.0)),
    ]
}

fn recipes_to_make(outputs: &[(Item, Rate)]) -> (Graph<Recipe, (Item, Rate)>) {
    let mut spec: Graph<Recipe, (Item, Rate)> = Graph::new();
    let mut recipes_used = HashMap::new();
    for &(item, rate) in outputs {
        let output_storage = recipe!(Storage[(Any, 900.0)] => (Any, 900.0));
        let out = recipes_used
            .entry(output_storage.clone())
            .or_insert_with(|| spec.add_node(output_storage));
        let mut todo = vec![(*out, item, rate)];
        while let Some((out, item, rate)) = todo.pop() {
            for r in recipes().iter() {
                if r.output == item {
                    let cur = recipes_used
                        .entry(r.clone())
                        .or_insert_with(|| spec.add_node(r.clone()));
                    let total_rate = if let Some(e) = spec.find_edge(*cur, out) {
                        spec[e].1 + rate
                    } else {
                        rate
                    };
                    spec.update_edge(*cur, out, (item, total_rate));
                    for &(ii, ir) in r.inputs.iter() {
                        todo.push((*cur, ii, rate * (ir / r.output_rate)));
                    }
                    continue;
                }
            }
        }
    }
    spec
}

fn legalize(spec: &mut Graph<Recipe, (Item, Rate)>) {
    for out in spec
        .externals(Direction::Outgoing)
        .collect::<Vec<NodeIndex<u32>>>()
    {
        let mut bfs = Bfs::new(Reversed(&*spec), out);
        while let Some(nr_orig) = bfs.next(Reversed(&*spec)) {
            let r_orig = dbg!(spec[nr_orig].clone());
            let total_out = spec
                .edges_directed(nr_orig, Direction::Outgoing)
                .fold(0.0, |acc, e| acc + e.weight().1);
            if dbg!(total_out) > r_orig.output_rate {
                let repetitions = (total_out / r_orig.output_rate).ceil();
                // create the necessary new nodes and add/update the scaled down edges to/from the inputs and outputs
                let nrs = (0..dbg!(repetitions as usize) - 1)
                    .into_iter()
                    .map(|_| spec.add_node(r_orig.clone()))
                    .chain(std::iter::once(nr_orig))
                    .collect::<Vec<NodeIndex<u32>>>();
                let mut outs = spec
                    .neighbors_directed(nr_orig, Direction::Outgoing)
                    .detach();
                let mut todo_outs = vec![];
                while let Some((out_e, out_n)) = outs.next(&*spec) {
                    let (oi, mut or) = spec[out_e];
                    while or > r_orig.output_rate {
                        or -= r_orig.output_rate;
                        todo_outs.push(((oi, r_orig.output_rate), out_n));
                    }
                    todo_outs.push(((oi, or), out_n));
                    spec.remove_edge(out_e);
                }
                todo_outs.sort_unstable_by(|x, y| (x.0).1.partial_cmp(&(y.0).1).unwrap());
                dbg!(&todo_outs);
                for nr in nrs {
                    let mut remaining_output = r_orig.output_rate;
                    loop {
                        if dbg!(remaining_output) == 0.0 {
                            break;
                        }
                        if let Some(((oi, or), on)) = todo_outs.pop() {
                            if or > remaining_output {
                                spec.add_edge(nr, on, (oi, remaining_output));
                                todo_outs.push(((oi, or - remaining_output), on));
                                remaining_output = 0.0;
                                break;
                            } else {
                                remaining_output -= or;
                                spec.add_edge(nr, on, (oi, or));
                            }
                        } else {
                            break;
                        }
                    }
                    let input_scale = (r_orig.output_rate - remaining_output) / r_orig.output_rate;
                    let mut ins = spec
                        .neighbors_directed(nr_orig, Direction::Incoming)
                        .detach();
                    while let Some((in_e, in_n)) = ins.next(&*spec) {
                        let (ii, _ir) = spec[in_e];
                        let ir = r_orig
                            .inputs
                            .iter()
                            .find_map(|&(rii, ir)| if rii == ii { Some(ir) } else { None })
                            .expect("input edge provides an item used in the recipe");
                        let scaled_e = (ii, (ir * dbg!(input_scale)));
                        if nr == nr_orig {
                            // update any edges to the original node
                            spec[in_e] = dbg!(scaled_e);
                        } else {
                            spec.add_edge(dbg!(in_n), dbg!(nr), dbg!(scaled_e));
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let desired_outputs = &[
        (Item::IronRod, 60.0),
        (Item::IronPlate, 60.0),
        (Item::Wire, 90.0),
        (Item::Cable, 45.0),
    ];
    let mut spec = recipes_to_make(desired_outputs);
    legalize(&mut spec);
    println!("{:?}", Dot::new(&spec));
    verify_spec(&spec);
    /* println!(
        "In order to make 60 Iron Plates, you need: {:?}",
        recipes_to_make(Item::IronPlate, 60.0)
    ); */
}

fn verify_spec(spec: &Graph<Recipe, (Item, Rate)>) {
    for out in spec
        .externals(Direction::Incoming)
        .collect::<Vec<NodeIndex<u32>>>()
    {
        let mut bfs = Bfs::new(spec, out);
        while let Some(n) = bfs.next(spec) {
            if spec[n].machine == Machine::Storage {
                continue;
            }

            let mut overall_rate = 1.0;
            let mut total_inputs = HashMap::new();

            for in_e in spec.edges_directed(n, Direction::Incoming) {
                total_inputs
                    .entry(in_e.weight().0)
                    .and_modify(|r| *r += in_e.weight().1)
                    .or_insert(in_e.weight().1);
            }

            for (item, rate) in total_inputs {
                let max_rate = spec[n]
                    .inputs
                    .iter()
                    .find_map(|&(ii, ir)| if ii == item { Some(ir) } else { None })
                    .expect("input to recipe exists");
                let effective_rate = rate / max_rate;
                assert!(
                    overall_rate == 1.0
                        || (overall_rate - effective_rate).abs() <= std::f64::EPSILON
                );
                overall_rate = effective_rate;
            }

            let total_output =
                spec.edges_directed(n, Direction::Outgoing)
                    .fold(0.0, |acc, out_e| {
                        assert!(out_e.weight().0 == spec[n].output);
                        acc + out_e.weight().1
                    });

            assert!(total_output <= spec[n].output_rate * overall_rate);
        }
    }
}
