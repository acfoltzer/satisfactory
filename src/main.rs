use petgraph::dot::{Config, Dot};
use petgraph::prelude::{Bfs, Direction, Graph, NodeIndex};
use petgraph::visit::Reversed;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type Rate = f64;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Item {
    Any,
    IronOre,
    CopperOre,
    IronIngot,
    CopperIngot,
    IronRod,
    IronPlate,
    Wire,
    Cable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Machine {
    IronMinerMk1,
    CopperMinerMk1,
    Smelter,
    Constructor,
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
        self.inputs.iter().map(|i| {
            i.0.hash(state);
            (i.1 as usize).hash(state)
        });
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
        recipe!(Smelter[(IronOre, 30.0)] => (IronIngot, 30.0)),
        recipe!(Smelter[(CopperOre, 30.0)] => (CopperIngot, 30.0)),
        recipe!(Constructor[(IronIngot, 15.0)] => (IronRod, 15.0)),
        recipe!(Constructor[(IronIngot, 30.0)] => (IronPlate, 15.0)),
        recipe!(Constructor[(CopperIngot, 15.0)] => (Wire, 45.0)),
        recipe!(Constructor[(Wire, 30.0)] => (Cable, 15.0)),
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
        /*         (Item::IronRod, 60.0),
                (Item::IronPlate, 60.0), */
        /*         (Item::Wire, 90.0), */
        (Item::Cable, 30.0),
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
    fn outputs_at(spec: &Graph<Recipe, (Item, Rate)>, node: NodeIndex<u32>) -> (Item, Rate) {
        assert!(spec[node].machine != Machine::Storage);
        let mut inputs = spec.neighbors_directed(node, Direction::Incoming).detach();
        let mut rate = 1.0;
        while let Some((in_e, in_n)) = inputs.next(spec) {
            let effective_rate = spec[in_e].1 / spec[in_n].output_rate;
            assert!(rate == 1.0 || rate == effective_rate);
            rate = effective_rate;

            let (oi, or) = outputs_at(spec, in_n);
            let (ei, er) = spec[in_e];
            assert_eq!(oi, ei);
            assert!(or >= er);
        }
        (spec[node].output, spec[node].output_rate * rate)
    }

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
