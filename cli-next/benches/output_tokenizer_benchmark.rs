use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wukong_next::output::tokenizer::OutputTokenizer;

fn bench_tokenize_long_sentence(c: &mut Criterion) {
    let sentence = r#"
The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is SUCCESS.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is TERMINAL.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is SUCCESS.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is ABORT.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is ABORT.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is TERMINAL.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is SUCCESS.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is ABORT.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is ABORT.

The last build for mv-ci-mock is build-213. This is built from PR #31 by abc@gmail.com (@jk-gan).
The commit hash is a9a97d98635e7a5218c554ee9a41132e3603cc97. The status is SUCCESS."#;

    c.bench_function("tokenize long sentence", |b| {
        b.iter(|| OutputTokenizer::tokenize(black_box(sentence.to_string())))
    });
}

fn bench_tokenize_table_output(c: &mut Criterion) {
    let sentence = r#"
CD pipeline list for application mv-wukong-ci-mock:
┌───────┬─────────┬──────────────┬────────────────┬──────────────────────────┬─────────────────┬───────────┐
│ Prod                                                                                                     │
├───────┼─────────┼──────────────┼────────────────┼──────────────────────────┼─────────────────┼───────────┤
│ Name  │ Enabled │ Deployed Ref │ Build Artifact │ Triggered By             │ Last deployment │ Status    │
├───────┼─────────┼──────────────┼────────────────┼──────────────────────────┼─────────────────┼───────────┤
│ Blue  │ Ready   │ 1e3dfa2      │ main-build-9   │ alex.tuan@mindvalley.com │ 3 days ago      │ SUCCEEDED │
├───────┼─────────┼──────────────┼────────────────┼──────────────────────────┼─────────────────┼───────────┤
│ Green │ Ready   │ b6c2581      │ main-build-11  │ alex.tuan@mindvalley.com │ 6 hours ago     │ SUCCEEDED │
└───────┴─────────┴──────────────┴────────────────┴──────────────────────────┴─────────────────┴───────────┘

┌───────┬─────────┬──────────────┬────────────────┬──────────────────────────┬─────────────────┬───────────┐
│ Staging                                                                                                  │
├───────┼─────────┼──────────────┼────────────────┼──────────────────────────┼─────────────────┼───────────┤
│ Name  │ Enabled │ Deployed Ref │ Build Artifact │ Triggered By             │ Last deployment │ Status    │
├───────┼─────────┼──────────────┼────────────────┼──────────────────────────┼─────────────────┼───────────┤
│ Blue  │ Ready   │ b6c2581      │ main-build-8   │ alex.tuan@mindvalley.com │ 2 days ago      │ SUCCEEDED │
├───────┼─────────┼──────────────┼────────────────┼──────────────────────────┼─────────────────┼───────────┤
│ Green │ Ready   │ b6c2581      │ main-build-8   │ alex.tuan@mindvalley.com │ a day ago       │ TERMINAL  │
└───────┴─────────┴──────────────┴────────────────┴──────────────────────────┴─────────────────┴───────────┘"#;

    c.bench_function("tokenize table output", |b| {
        b.iter(|| OutputTokenizer::tokenize(black_box(sentence.to_string())))
    });
}

criterion_group!(
    benches,
    bench_tokenize_long_sentence,
    bench_tokenize_table_output
);
criterion_main!(benches);
