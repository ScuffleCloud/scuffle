{% if publish %}
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-v{{ version }}-orange?labelColor=5C5C5C" height="20">](https://crates.io/crates/{{ package }}/{{ version }})
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-v{{ version }}-blue?labelColor=5C5C5C" height="20">](https://docs.rs/{{ package }}/{{ version }})
{% endif %}
{{ changelog }}
