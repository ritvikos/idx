use hashbrown::HashMap;

enum Pos {
    Adjective,
    Noun,
    Punctuation,
    Verb,
}

pub struct Lemmatizer {
    lookup: Lookup,
}

struct Lookup {
    rules: HashMap<Pos, Vec<(String, String)>>,
    index: HashMap<Pos, HashMap<String, String>>,
    exceptions: HashMap<Pos, HashMap<String, String>>,
}
