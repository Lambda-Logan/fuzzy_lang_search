#![allow(warnings, unused)]

use serde_json;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
//use std::fmt::fmt;

mod feat;

//use feat::{
//    book_ends, n_gram, skipgram, AnonFtzr, BookEndsFtzr, CanGram, DefaultAscii, DefaultUnicode,
//    Doc, DocFtzr, EmptyFtzr, FeatEntry, Featurizer, FuzzyEntry, MultiFtzr, SkipScheme,
//};

mod dualiter;
use dualiter::*;

mod ftzrs;
use ftzrs::{
    book_ends, n_gram, skipgram, BookEndsFtzr, CanGram, EmptyFtzr, Feature, MultiFtzr, SkipScheme,
};

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
#[macro_use]
extern crate lazy_static;

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

//type LangFeaturizer = MultiFtzr<
//    SkipScheme,
//    MultiFtzr<BookEndsFtzr<SkipScheme>, MultiFtzr<BookEndsFtzr<SkipScheme>, EmptyFtzr>>,
//>;
//type LangFeaturizer =
//    MultiFtzr<SkipScheme, MultiFtzr<SkipScheme, MultiFtzr<BookEndsFtzr<SkipScheme>, EmptyFtzr>>>;
//type LangFeaturizer = SkipScheme;

type LangFeaturizer =
    MultiFtzr<SkipScheme, MultiFtzr<SkipScheme, MultiFtzr<BookEndsFtzr<SkipScheme>, EmptyFtzr>>>;

fn make_index() -> FuzzyIndex<LangColumn, LangFeaturizer, SimplePoint> {
    let ftzr: LangFeaturizer = featurizers![n_gram(2), n_gram(3), book_ends((2, 2), n_gram(2))];
    /*let ftzr: LangFeaturizer = SkipScheme {
        group_a: (0, 1),
        gap: (0, 1),
        group_b: (1, 2),
    };
    let ftzr: LangFeaturizer = featurizers![
        SkipScheme {
            group_a: (3, 4),
            gap: (0, 0),
            group_b: (0, 0)
        },
        book_ends(
            (3, 0),
            SkipScheme {
                group_a: (0, 1),
                gap: (0, 1),
                group_b: (1, 2)
            }
        ),
        book_ends(
            (0, 2),
            SkipScheme {
                group_a: (0, 1),
                gap: (0, 1),
                group_b: (0, 1)
            }
        )
    ]; */
    let langs: Vec<LangColumn> = get_lang_names();
    let mut index: FuzzyIndex<LangColumn, _, SimplePoint> =
        FuzzyIndex::new(ftzr.clone(), langs.into_iter());
    index.compress_index(20);
    index
}

lazy_static! {
    //static ref LANGS: Vec<LangColumn> = get_lang_names();
    static ref FUZZY_INDEX: FuzzyIndex<LangColumn, LangFeaturizer, SimplePoint> = make_index();
}

//https://github.com/second-state/wasm-learning/tree/master/browser/hello
#[wasm_bindgen]
pub fn lookup(s: String) -> String {
    let mut s = s.clone();
    s.push_str(" ");
    let params = SearchParams {
        metric: Total,        //The comparison metric, also try 'Total' or 'Jaccard'
        depth: 16,            //the maximum depth of the search
        breadth: 16,          //the maximum breadth of the search
        timeout_n: 10,        //if the last maximum was 'timeout_n' ago, it will stop
        return_if_gt: 100, //given as a percentage between 0 and 100... Will stop after a good enough result
        max_comparisons: 100, //the maximum number of comparisons that can be made in a search
    };
    let mut sp = ScratchPad::default();
    let query = LangColumn::from_query(s.clone());
    let index: &FuzzyIndex<LangColumn, LangFeaturizer, SimplePoint> = &*FUZZY_INDEX;
    let results: Vec<_> = index.matches(LangColumn::from_query(s), &mut sp, &params);
    //let mut ret = "".to_string();
    let max_sim = results.last().map(|x| x.1).unwrap_or_default();
    let mut ret = Vec::new();
    for r in results
        .into_iter()
        .rev()
        .take_while(|x| x.1 > (0.1 * max_sim))
    {
        let unidecoded = r.0.as_ref().unidecoded.clone();
        let en_name = r.0.as_ref().english_name.clone();
        let json = serde_json::json!({
            "Unidecoded name    ": if unidecoded == en_name {"".into()} else {unidecoded},
            "Native name         ":r.0.as_ref().native_name.clone(),
            "English name        ": en_name.clone(),
            "Similarity (based on cosine) ": format!("{:.3}", r.1)
        });
        ret.push(((-(r.1 * 1024.0) as isize + en_name.len() as isize), json));
    }
    //let v = serde::ser::Serialize(&ret);
    //serde_json_wasm::to_string(&ret).unwrap()

    ret.sort_by(|a, b| (a.0).cmp(&b.0));
    let ret: Vec<_> = Iterator::collect(ret.into_iter().map(|x| x.1));
    serde_json::to_string(&ret).unwrap()
}
