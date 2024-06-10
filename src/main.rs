use automata::{
    prelude::*,
    random::{generate_random_dba, generate_random_omega_words},
};
use math::set::IndexSet;

fn main() {
    // set parameters
    let num_symbols = 2;
    let automata_sizes = vec![4];
    let automata_per_size = 2;
    let train_sizes = vec![100];
    let test_size = 1000;
    let num_sets = 2;
    let lambda = 0.95;

    // generate DBAs
    for size in automata_sizes.iter() {
        for i in 0..automata_per_size {
            let dba = generate_dba(num_symbols, *size, lambda);
            export_automaton();
        }
    }

    // generate train and test sets
    for aut_size in automata_sizes {
        for train_size in train_sizes.iter() {
            for i in 0..num_sets {
                let len_spoke = 2 * ((aut_size as f64).log2().ceil() as usize) - 1;
                let len_cycle = (2 * aut_size - len_spoke) * len_spoke;
                let (train, test) =
                    generate_set(num_symbols, len_spoke, len_cycle, *train_size, test_size);
                export_set();
            }
        }
    }
    // draw sample sets
    // export sample sets
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

/// Write the given automaton to the given `path` in HOA format
pub fn export_automaton() {
    todo!();
}

/// Write the given automaton to the given `path` in HOA format
pub fn export_set() {
    todo!();
}
