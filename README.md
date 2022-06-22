# clap-handler

A command handler works with `clap-derive` to generating subcommand handlers.

## Example

```rust
use clap::{Parser, AppSettings};
use clap_handler::{Handler, handler, Context};

#[derive(Parser, Handler, Debug, Clone)]
#[clap(name = "Your program", author)]
pub struct Arguments {
    #[clap(subcommand)]
    subcommand: Subcommand,
}


#[derive(Parser, Handler, Debug, Clone)]
#[handler_inject(add_something)]
pub enum Subcommand {
    First(FirstSubcommand),
}

impl Subcommand {
    fn add_something(&self, ctx: &mut Context) -> anyhow::Result<()> {
        // insert something
        // ctx.insert(a_struct);
        Ok(())
    }
}

#[derive(Args, Debug, Clone)]
pub struct FirstSubcommand {
    arg: String,
}

#[handler(FirstSubcommand)]
fn handle_first(me: FirstSubcommand) -> anyhow::Result<()> {
    Ok(())
}

fn main() {
    let args = Arguments::parse();
    log::debug!("{:#?}", args);
    args.run().await
}
```

For more complex examples, see the [`anni`][anni] and [`sswa`][sswa].

[anni]: https://github.com/ProjectAnni/anni/blob/master/anni/src/main.rs

[sswa]: https://github.com/Yesterday17/sswa/blob/master/sswa/src/args.rs

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.