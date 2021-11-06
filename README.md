# spell

Peter Norvig's Spelling Corrector in Rust.

## Usage

```rust
use spell::SpellingCorrector;

fn main() -> Result<(), anyhow::Error> {
    let sc = SpellingCorrector::new("data/big.txt")?;
    let c = sc.correction("speling");
    assert_eq!(c, "spelling");
    Ok(())
}
```

## References

- [How to Write a Spelling Corrector](https://norvig.com/spell-correct.html)

## License

[MIT License](LICENSE)
