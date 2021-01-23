use {
    bmfont::*,
    criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion},
    std::fs::File,
};

const INPUTS: &[(&str, &str)] = &[
    ("01: Empty", ""),
    ("02: Character", "w"),
    ("03: Word", "Score"),
    ("04: Pair", "Winner: Attackgoat"),
    ("05: Sentance", "The lazy dog can sit wherever she wishes."),
    ("06: Two lines", "Where is her sweater?\nIt's not in her car."),
    ("07: Long sentance", "The acquisition of wealth is no longer the driving force of our lives. We work to better ourselves and the rest of humanity."),
    ("08: Two long lines", "Someone once told me that time was a predator that stalked us all our lives. I rather believe that time is a\ncompanion who goes with us on the journey and reminds us to cherish every moment, because it will never come again. What we leave behind is not as important as how we've lived."),
    ("09: Lyrics", "Van Gogh my earlobe\nI can't hear, I'm here though\nI may be a weirdo, but this is my year, yo\nMy life may be crazy\nMy lack of the lazy has let me write code that I love on the daily.\nVan Gogh my earlobe\nI can't hear, I'm here though\nI may be a weirdo, but this is my year, yo\nMy life may be crazy\nMy lack of the lazy has let me write code that I love on the daily.\nVan Gogh my earlobe\nI can't hear, I'm here though\nI may be a weirdo, but this is my year, yo\nMy life may be crazy\nMy lack of the lazy has let me write code that I love on the daily."),
];

fn parse(c: &mut Criterion) {
    let file = File::open("font.fnt").unwrap();
    let font = BMFont::new(file, OrdinateOrientation::TopToBottom).unwrap();
    let mut group = c.benchmark_group("Parse");
    for (desc, input) in INPUTS.iter() {
        group.bench_with_input(BenchmarkId::new("Input", desc), input, |b, input| {
            b.iter(|| font.parse(black_box(input)))
        });
    }
}

criterion_group!(benches, parse);
criterion_main!(benches);
