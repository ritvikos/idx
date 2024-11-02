use hashbrown::HashMap;

enum Pos {
    Adjective,
    Noun,
    Punctuation,
    Verb,
}

pub struct Lemmatizer {
    lookup: Lookup<Vec<(String, String)>>,
}

type Map<V> = HashMap<Pos, V>;

struct Lookup<R: IntoIterator<Item = (String, String)>> {
    exceptions: Map<HashMap<String, String>>,
    index: Map<HashMap<String, String>>,
    rules: Map<R>,
}

impl<R: IntoIterator<Item = (String, String)>> Lookup<R> {}
