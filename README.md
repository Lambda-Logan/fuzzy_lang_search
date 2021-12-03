# fuzzy_lang_search
a fuzzy and very fast search for language names with typos



This is a fun little project to play with approximate search in a real-world context. Picture a dropdown search-bar where the user types part of 1000+ options. How can typos be handled? This example uses a reverse index of n-grams, and the similarity metric used is cosine similarity (but jaccard or others works fine too).

In this example I'm using ```senor_borroso``` to search an index of 2000-ish language names.

# TRY IT OUT :-)

This assumes you have ```cargo``` installed
```
git clone https://github.com/Lambda-Logan/fuzzy_lang_search.git

cd fuzzy_lang_search

cargo run --release ```

This first time this is run, it will take some time to download a small number of deps. Then give it a try! 




All the language data is stored in lang_info.rs.

Check out the main function to see what's happening in the loop.

Here are some typos to try:

eenglish
enxglxish
arbbic
客話家
بيةفا
zhongwen
zhonggween

Most lookups happen in under 1/10th of a millisecond (<100 μs) and the entire index takes about 9 milleseconds to build.
