use std::collections::HashMap;

use automata::{
    automaton::InfiniteWordAutomaton,
    prelude::*,
    random::{generate_random_dba, generate_random_dpa, generate_random_omega_words},
};
use math::set::IndexSet;

fn main() {
    // set parameters
    let num_symbols = 2;
    let num_prios = 5;
    let automata_sizes = vec![4];
    let automata_per_size = 2;
    let train_sizes = vec![100];
    let test_size = 1000;
    let num_sets = 2;
    let lambda = 0.95;

    // generate DBAs
    println!("generating DBAs");
    let mut dbas = HashMap::new();
    for size in automata_sizes.iter() {
        let mut auts = vec![];
        for _ in 0..automata_per_size {
            let dba = generate_dba(num_symbols, *size, lambda);
            auts.push(dba);
            export_automaton();
        }
        dbas.insert(*size, auts);
    }

    // generate DPAs
    println!("generating DPAs");
    let mut dpas: Vec<DPA> = vec![];
    for size in automata_sizes.iter() {
        for _ in 0..automata_per_size {
            let dpa = generate_dpa(num_symbols, *size, num_prios, lambda);
            dpas.push(dpa);
            export_automaton();
        }
    }

    // generate train and test sets
    println!("generating word sets");
    let mut sets = HashMap::new();
    for aut_size in automata_sizes.iter() {
        for train_size in train_sizes.iter() {
            let mut sets_of_size = vec![];
            for _ in 0..num_sets {
                let len_spoke = 2 * ((*aut_size as f64).log2().ceil() as usize) - 1;
                let len_cycle = (2 * aut_size - len_spoke) * len_spoke;
                let (train, test) =
                    generate_set(num_symbols, len_spoke, len_cycle, *train_size, test_size);
                sets_of_size.push((train, test));
                export_set();
            }
            sets.insert((*aut_size, *train_size), sets_of_size);
        }
    }

    // label sets
    println!("labelling sets");
    for aut_size in automata_sizes.iter() {
        for train_size in train_sizes.iter() {
            let dba = &dbas[aut_size][0];
            let (tr, te) = &sets[&(*aut_size, *train_size)][0];
            let train = label_set(dba, tr);
            let test = label_set(dba, te);
            export_task();
        }
    }
}

/// Generate a random DBA with `aut_size` states on an alphabet of size `num_symbols`.
/// The parameter `lambda` is used to draw the acceptance condition from a continuous Bernoulli distribution.
/// The produced automaton has an informative right congruence.
pub fn generate_dba(num_symbols: usize, aut_size: usize, lambda: f64) -> DBA {
    let gen_size = aut_size + (aut_size as f32).log2().round() as usize - 1;
    let mut dba: DBA;
    loop {
        dba = generate_random_dba(num_symbols, gen_size, lambda).streamlined();
        // check if dba has the correct size
        if dba.size() == aut_size {
            let equiv_dpa = dba
                .clone()
                .map_edge_colors(|c| if c { 0 } else { 1 })
                .collect_dpa();
            // check if automaton has informative right congruence
            if equiv_dpa.is_informative_right_congruent() {
                break;
            }
        }
    }
    dba
}

/// Generate a random DPA with `aut_size` states on an alphabet of size `num_symbols`.
/// The parameter `lambda` is used to draw the acceptance condition from a continuous Bernoulli distribution.
/// The produced automaton has an informative right congruence.
pub fn generate_dpa(num_symbols: usize, aut_size: usize, num_prios: u8, lambda: f64) -> DPA {
    let gen_size = aut_size + (aut_size as f32).log2().round() as usize - 1;
    let mut dpa: DPA;
    loop {
        dpa = generate_random_dpa(num_symbols, gen_size, num_prios, lambda)
            .streamlined()
            .collect_dpa();
        // check if dpa has the correct size and has informative right congruence
        if dpa.size() == aut_size && dpa.is_informative_right_congruent() {
            break;
        }
    }
    dpa
}

/// Generate a training set, test set pair of random ultimately periodic words.
/// The length of spoke and cycle are drawn uniformly and the used alphabet is of size `num_symbols`.
pub fn generate_set(
    num_symbols: usize,
    len_spoke: usize,
    len_cycle: usize,
    train_size: usize,
    test_size: usize,
) -> (
    IndexSet<ReducedOmegaWord<char>>,
    IndexSet<ReducedOmegaWord<char>>,
) {
    let alphabet = CharAlphabet::of_size(num_symbols);
    let mut training_set = generate_random_omega_words(
        &alphabet,
        0,
        len_spoke,
        1,
        len_cycle,
        train_size + test_size,
    );
    let test_set = training_set.split_off(train_size);
    (training_set, test_set)
}

/// Label a `set` of [`ReducedOmegaWord`]s with the result of the given automaton.
pub fn label_set<Z, C>(
    aut: &InfiniteWordAutomaton<CharAlphabet, Z, Void, C, true>,
    set: &IndexSet<ReducedOmegaWord<char>>,
) -> Vec<(ReducedOmegaWord<char>, bool)>
where
    Z: OmegaSemantics<Void, C, Output = bool>,
    C: Color,
{
    set.into_iter()
        .map(|w| (w.clone(), aut.accepts(w)))
        .collect()
}

/// Write the given automaton to the given `path` in HOA format
pub fn export_automaton() {}

/// Write the given set to the given `path` as csv
pub fn export_set() {}

/// Write the given omega automata learning task to the given `path` in HOA format
pub fn export_task() {}
