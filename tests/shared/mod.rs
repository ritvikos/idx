pub(crate) fn get_test_corpus() -> Vec<String> {
    vec![
        "The quick brown fox jumps over the lazy dog.",
        "The quick brown fox.",
        "The quick brown fox jumps.",
        "The quick brown fox jumps over.",
        "The quick brown fox jumps over the lazy dog again.",
        "The lazy dog lies in the sun.",
        "The dog is lazy.",
        "Foxes are quick and brown.",
        "Foxes jump over lazy dogs.",
        "A fast brown fox leaps over lazy hounds.",
        "Speedy brown foxes jump over sluggish dogs.",
        "Foxes are cunning and quick.",
        "Dogs are loyal and lazy.",
        "A fox is quicker than a dog.",
        "Jumping foxes and sleeping dogs.",
        "The sun shines on the lazy dog.",
        "Quick thinking foxes outsmart lazy dogs.",
        "The fox and the hound.",
        "A quick brown fox outpaces a lazy brown dog.",
        "Clever foxes evade the lazy dogs.",
    ]
    .iter()
    .map(|document| document.to_string())
    .collect::<Vec<_>>()
}
