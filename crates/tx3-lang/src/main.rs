use miette::IntoDiagnostic as _;

mod ast;
mod parser;

fn main() -> miette::Result<()> {
    let input = r#"
        datum PoolState {
            pair_a: Token,
            pair_b: Token,
        }

        datum SwapParams {
            amount: Int,
            ratio: Int,
        }

        party Buyer;

        party Dex {
            address: addr1xxx,
        }

        tx swap(
            buyer: Buyer,
            ask: Token,
            bid: Token
        ) {
            input pool {
                from: dex,
                datum_is: PoolState,

                redeemer: SwapParams {
                    ask: ask,
                    bid: ask,
                }
            }
            
            input* payment {
                from: buyer,
                min_amount: fees + bid,
            }
            
            output {
                to: pool
                datum: PoolState {
                    pair_a: inputs.pool.pair_a - ask,
                    pair_b: inputs.pool.pair_b + bid,
                    ...inputs.pool.datum
                }
            }

            output {
                to: buyer,
                amount: inputs.payment.amount + ask - bid - fees,
            }
        }
    "#;

    let tmpls = parser::Tx3Parser::parse_program(input).into_diagnostic()?;

    println!("Successfully parsed templates:");
    println!("{:#?}", tmpls);

    Ok(())
}
