#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::engine::*;
use scrypto::prelude::*;

fn create_account(engine: &mut InMemoryRadixEngine) -> Address {
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(false);

    // Publish Account blueprint
    proc.publish_at(
        include_bytes!("../../assets/account.wasm"),
        Address::Package([0u8; 26]),
    )
    .unwrap();

    // Create account
    let account: Address = proc
        .call_function(Address::Package([0u8; 26]), "Account", "new", args!())
        .and_then(decode_return)
        .unwrap();

    // Allocate 1 XRD
    let bucket = scrypto::resource::Bucket::from(proc.create_bucket(1.into(), Address::RadixToken));
    proc.call_method(account, "deposit", args!(bucket)).unwrap();

    // Commit
    proc.finalize().unwrap();
    runtime.commit();

    account
}

fn create_gumball_machine(engine: &mut InMemoryRadixEngine) -> Address {
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(false);

    let package = proc
        .publish(include_bytes!("../../assets/gumball-machine.wasm"))
        .unwrap();

    let component: Address = proc
        .call_function(package, "GumballMachine", "new", args!())
        .and_then(decode_return)
        .unwrap();

    proc.finalize().unwrap();
    runtime.commit();

    component
}

fn cross_component_call(b: &mut Bencher) {
    let mut engine = InMemoryRadixEngine::new();
    let account = create_account(&mut engine);
    let component = create_gumball_machine(&mut engine);

    b.iter(|| {
        let mut runtime = engine.start_transaction();
        let mut proc = runtime.start_process(false);
        let xrd: scrypto::resource::Bucket = proc
            .call_method(
                account,
                "withdraw",
                args!(Amount::one(), Address::RadixToken),
            )
            .and_then(decode_return)
            .unwrap();
        let gum: scrypto::resource::Bucket = proc
            .call_method(component, "get_gumball", args!(xrd))
            .and_then(decode_return)
            .unwrap();
        proc.call_method(account, "deposit", args!(gum)).unwrap();
        proc.finalize().unwrap();
        //runtime.commit();
    });
}

benchmark_group!(radix_engine, cross_component_call);
benchmark_main!(radix_engine);