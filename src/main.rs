use csv::Writer;
use std::collections::HashMap;
use std::fs;

use automata::{
    automaton::InfiniteWordAutomaton,
    hoa::output::WriteHoa,
    prelude::*,
    random::{generate_random_dba, generate_random_dpa, generate_random_omega_words},
};
use math::set::IndexSet;

fn main() {
    // set parameters
    let num_symbols = 2;
    let num_prios = 5;
    let automata_sizes = vec![4, 8];
    let automata_per_size = 2;
    let train_sizes = vec![100, 1000];
    let test_size = 1000;
    let num_sets = 2;
    let lambda = 0.95;
    fs::create_dir_all("data/automata").unwrap();
    fs::create_dir_all("data/sets").unwrap();

    // generate DBAs
    println!("generating DBAs");
    let mut dbas = HashMap::new();
    for &size in automata_sizes.iter() {
        let mut auts = vec![];
        for i in 0..automata_per_size {
            let dba = generate_dba(num_symbols, size, lambda);
            export_automaton(aut_name(size, i, "dba".to_string()), &dba);
            auts.push(dba);
        }
        dbas.insert(size, auts);
    }

    // generate DPAs
    println!("generating DPAs");
    let mut dpas = HashMap::new();
    for &size in automata_sizes.iter() {
        let mut auts = vec![];
        for i in 0..automata_per_size {
            let dpa = generate_dpa(num_symbols, size, num_prios, lambda);
            export_automaton(aut_name(size, i, "dpa".to_string()), &dpa);
            auts.push(dpa);
        }
        dpas.insert(size, auts);
    }

    // generate train and test sets
    println!("generating word sets");
    let mut sets = HashMap::new();
    for &aut_size in automata_sizes.iter() {
        for &train_size in train_sizes.iter() {
            let mut sets_of_size = vec![];
            for i in 0..num_sets {
                let len_spoke = 2 * ((aut_size as f64).log2().ceil() as usize) - 1;
                let len_cycle = (2 * aut_size - len_spoke) * len_spoke;
                let (train, test) =
                    generate_set(num_symbols, len_spoke, len_cycle, train_size, test_size);
                export_set(set_name(aut_size, train_size, i, true), &train);
                export_set(set_name(aut_size, train_size, i, false), &test);
                sets_of_size.push((train, test));
            }
            sets.insert((aut_size, train_size), sets_of_size);
        }
    }

    // label dba sets
    println!("labelling dba sets");
    for &aut_size in automata_sizes.iter() {
        for (aut_index, dba) in dbas[&aut_size].iter().enumerate() {
            for &train_size in train_sizes.iter() {
                for (set_index, &(ref tr, ref te)) in
                    sets[&(aut_size, train_size)].iter().enumerate()
                {
                    let train = label_set(dba, tr);
                    let test = label_set(dba, te);
                    // export as learning task
                    export_task(
                        task_name(
                            aut_size,
                            train_size,
                            aut_index,
                            set_index,
                            "dba".to_string(),
                        ),
                        dba,
                        &train,
                        &test,
                    );
                }
            }
        }
    }

    // label dpa sets
    println!("labelling dpa sets");
    for &aut_size in automata_sizes.iter() {
        for (aut_index, dpa) in dpas[&aut_size].iter().enumerate() {
            for &train_size in train_sizes.iter() {
                for (set_index, &(ref tr, ref te)) in
                    sets[&(aut_size, train_size)].iter().enumerate()
                {
                    let train = label_set(dpa, tr);
                    let test = label_set(dpa, te);
                    // export as learning task
                    export_task(
                        task_name(
                            aut_size,
                            train_size,
                            aut_index,
                            set_index,
                            "dpa".to_string(),
                        ),
                        dpa,
                        &train,
                        &test,
                    );
                }
            }
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
pub fn export_automaton<AUT: WriteHoa>(file: String, aut: &AUT) {
    fs::write(file, aut.to_hoa()).expect("Unable to write file");
}

/// Give filename for an omega automaton
pub fn aut_name(aut_size: usize, aut_index: usize, acc_type: String) -> String {
    format!("data/automata/{acc_type}__aut_size={aut_size}__{aut_index:0>2}.hoa")
}

/// Write the given set to the given `path` as csv
pub fn export_set(file: String, set: &IndexSet<ReducedOmegaWord<char>>) {
    let mut wtr = Writer::from_path(file).expect("creating file failed");
    for w in set.iter() {
        wtr.write_record(&[
            w.spoke().iter().collect::<String>(),
            w.cycle().iter().collect(),
        ])
        .unwrap();
    }
    wtr.flush().unwrap();
}

/// Give filename for a set of omega words
pub fn set_name(aut_size: usize, set_size: usize, set_index: usize, train: bool) -> String {
    let class = if train { "train" } else { "test" };
    format!("data/sets/word_set__aut_size={aut_size}__sample_size={set_size}__{set_index:0>2}_{class}.csv")
}

pub fn export_labelled_set(file: String, set: &Vec<(ReducedOmegaWord<char>, bool)>) {
    let mut wtr = Writer::from_path(file).expect("creating file failed");
    for (w, r) in set.iter() {
        wtr.write_record(&[
            w.spoke().iter().collect(),
            w.cycle().iter().collect(),
            format!("{r:?}"),
        ])
        .unwrap();
    }
    wtr.flush().unwrap();
}

/// Write the given omega automata learning task to the given `path` in HOA format
pub fn export_task<AUT: WriteHoa>(
    name: String,
    aut: &AUT,
    train: &Vec<(ReducedOmegaWord<char>, bool)>,
    test: &Vec<(ReducedOmegaWord<char>, bool)>,
) {
    fs::create_dir_all(format!("data/tasks/{name}")).unwrap();
    export_automaton(format!("data/tasks/{name}/aut.hoa"), aut);
    export_labelled_set(format!("data/tasks/{name}/train.csv"), train);
    export_labelled_set(format!("data/tasks/{name}/test.csv"), test);
}

pub fn task_name(
    aut_size: usize,
    set_size: usize,
    aut_index: usize,
    set_index: usize,
    acc_type: String,
) -> String {
    format!("{acc_type}_task__aut_size={aut_size}__sample_size={set_size}__{acc_type}{aut_index:0>2}__sample{set_index:0>2}")
}
