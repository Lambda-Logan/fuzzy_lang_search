#![allow(warnings)]
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::str;
use std::time::{Duration, Instant};
use unidecode::unidecode;

mod feat;
//use feat::{
//    book_ends, n_gram, skipgram, AnonFtzr, BookEndsFtzr, CanGram, DefaultAscii, DefaultUnicode,
//    Doc, DocFtzr, EmptyFtzr, FeatEntry, Featurizer, FuzzyEntry, MultiFtzr, SkipScheme,
//};

mod dualiter;
use dualiter::*;

mod ftzrs;
use ftzrs::{book_ends, n_gram, skipgram, CanGram, EmptyFtzr, MultiFtzr};

mod fuzzyindex;
use fuzzyindex::{FuzzyIndex, ScratchPad, SearchParams};

mod fuzzypoint;
use fuzzypoint::{
    Cosine, Counted, FuzzyPoint, Hamming, Jaccard, Labeled, Metric, SimHash, SimplePoint, Total,
};

mod hasfeatures;
use hasfeatures::{HasFeatures, HasName};

mod utils;
use utils::{get_entry, open_lexicon, rec_rev_str, shuffle, Entry};
//fn test_index<Ftzr: CanGram, Point: FuzzyPoint>(lookup: &FuzzyIndex<&String, Ftzr, Point>) {
mod lang_info;
use lang_info::*;

fn induce_typo(word: &str) -> String {
    let mut messed_up: String = "A".to_owned();
    messed_up.push_str(&unidecode(&word));

    if word.len() > 4 {
        messed_up.insert_str(4, "E");
    };
    messed_up
}

#[macro_export]
macro_rules! featurizers {
    () => {
        (EmptyFtzr)
    };
    ($a:expr $(, $tail:expr)*) => {{
        MultiFtzr {
            a: $a,
            b: featurizers!($($tail), *),
        }
    }};
}

//returns the user input
fn input() -> String {
    let mut s = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

fn main() {
    //here are some other potential featurizers:
    //    let ftzr = n_gram(2);
    //    let ftzr = skipgram(2, (0, 1), 2);
    //    let ftzr = book_ends((2, 2), n_gram(1))
    //    (the 'featurizers!' macro combines one or more featurizers)
    let ftzr = featurizers![n_gram(3), book_ends((2, 2), n_gram(2))];

    let langs: Vec<LangColumn> = get_lang_names();

    //This is the fuzzy index used for lookup
    let preindex = Instant::now();
    let mut index: FuzzyIndex<LangColumn, _, SimplePoint> =
        FuzzyIndex::new(ftzr.clone(), langs.into_iter());
    let post_index = preindex.elapsed().as_millis();
    //ScratchPad is just a place to re-use mutable state that's needed in each look-up
    let mut sp = ScratchPad::default();
    // 'SearchParams' is a struct to hold the runtime configuration of a lookup
    let params = SearchParams {
        metric: Cosine,       //The comparison metric, also try 'Total' or 'Jaccard'
        depth: 16,            //the maximum depth of the search
        breadth: 16,          //the maximum breadth of the search
        timeout_n: 10,        //if the last maximum was 'timeout_n' ago, it will stop
        return_if_gt: 100, //given as a percentage between 0 and 100... Will stop after a good enough result
        max_comparisons: 100, //the maximum number of comparisons that can be made in a search
    };
    println!("index was built in {:?} milliseconds", post_index);
    println!("index has a total of {:?} entries", index.points.len());

    while true {
        print!("Enter a lang with a typo: ");
        let query_string = input().to_lowercase();
        if query_string == "quit" {
            break;
        }
        if !query_string.is_empty() {
            let query = LangColumn::from_query(query_string.to_owned());
            let now = Instant::now();
            let results = index.matches(query, &mut sp, &params);
            let elapsed = now.elapsed().as_micros();
            for (langroot, sim) in results.iter() {
                println!("\t{:?}, {:?}", &langroot.english_name, sim);
                println!("\t\t{:?}", langroot.id);
                println!("\t\t{:?}\n", langroot.native_name);
            }
            println!("\tLOOKUP COMPLETED IN {:?} μs", elapsed);
            println!("--------------------------------------------\n\n")
        }
    }
}
