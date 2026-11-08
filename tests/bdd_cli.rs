use std::process::{Command, Output};

use cucumber::{World as _, given, then, when};

#[derive(Debug, Default, cucumber::World)]
struct CliWorld {
    saida: Option<Output>,
}

#[given("que a CLI ies está disponível")]
fn cli_disponivel(_: &mut CliWorld) {
    assert!(
        std::path::Path::new(env!("CARGO_BIN_EXE_ies")).is_file(),
        "o binário ies deve estar disponível"
    );
}

#[when("executo ies --help")]
fn executar_ajuda(world: &mut CliWorld) {
    world.saida = Some(
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .arg("--help")
            .output()
            .expect("a CLI ies deve iniciar"),
    );
}

fn saida(world: &CliWorld) -> &Output {
    world.saida.as_ref().expect("o comando deve ter sido executado")
}

#[then("o código de saída deve ser 0")]
fn codigo_zero(world: &mut CliWorld) {
    assert_eq!(saida(world).status.code(), Some(0));
}

#[then("a saída deve apresentar o comando processo")]
fn ajuda_apresenta_processo(world: &mut CliWorld) {
    let stdout = String::from_utf8_lossy(&saida(world).stdout);
    assert!(stdout.contains("processo"), "ajuda sem comando processo: {stdout}");
}

fn main() {
    futures::executor::block_on(
        CliWorld::cucumber()
            .run_and_exit("tests/features/cli_ajuda.feature"),
    );
}
