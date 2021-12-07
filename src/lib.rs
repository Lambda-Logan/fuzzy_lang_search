use serde::{Deserialize, Serialize};
use serde_json_wasm::from_str;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
//use std::fmt::fmt;

mod testing;
use testing::{FuzzyIndexTest, HnswTester, Testable, TrainingAtom};

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

type LangFeaturizer =
    MultiFtzr<SkipScheme, MultiFtzr<SkipScheme, MultiFtzr<BookEndsFtzr<SkipScheme>, EmptyFtzr>>>;

fn make_index() -> FuzzyIndex<LangColumn, LangFeaturizer, SimplePoint> {
    let ftzr: LangFeaturizer = featurizers![n_gram(2), n_gram(3), book_ends((2, 2), n_gram(2))];
    let langs: Vec<LangColumn> = get_lang_names();
    let mut index: FuzzyIndex<LangColumn, _, SimplePoint> =
        FuzzyIndex::new(ftzr.clone(), langs.into_iter());
    index
}

lazy_static! {
    //static ref LANGS: Vec<LangColumn> = get_lang_names();
    static ref FUZZY_INDEX: FuzzyIndex<LangColumn, LangFeaturizer, SimplePoint> = make_index();
}

//https://github.com/second-state/wasm-learning/tree/master/browser/hello
#[wasm_bindgen]
pub fn lookup(s: String) -> String {
    let params = SearchParams {
        metric: Cosine,       //The comparison metric, also try 'Total' or 'Jaccard'
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
    let mut ret = "".to_string();
    for r in results.into_iter().rev().take(8) {
        ret.push_str(
            format!(
                "{:?}\n\n",
                (&r.0.as_ref().english_name, &r.0.as_ref().native_name, r.1)
            )
            .as_str(),
        )
    }
    ret
}