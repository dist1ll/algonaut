use algonaut::core::MicroAlgos;
use algonaut::transaction::{Pay, Txn};
use algonaut::{Algod, Kmd};
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;
    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let list_response = kmd.list_wallets()?;

    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "testwallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };

    let init_response = kmd.init_wallet_handle(&wallet_id, "testpassword")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let gen_response = kmd.generate_key(&wallet_handle_token)?;
    let from_address = gen_response.address.parse()?;
    println!("from_address: {}", &gen_response.address);

    let gen_response = kmd.generate_key(&wallet_handle_token)?;
    let to_address = gen_response.address.parse()?;
    println!("to_address: {}", &gen_response.address);

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 1000)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to_address)
                .build(),
        )
        .build();

    let sign_response = kmd.sign_transaction(&wallet_handle_token, "testpassword", &t)?;

    println!(
        "kmd made signed transaction with {} bytes",
        sign_response.signed_transaction.len()
    );

    // broadcast the transaction to the network
    // note: this transaction may get rejected because the accounts do not have any tokens
    let send_response = algod.raw_transaction(&sign_response.signed_transaction)?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
