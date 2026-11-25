//! Montagem dos argumentos da CLI para os casos E2E, sem rede própria.
use super::*;

fn pedido(caso: Caso) -> (&'static str, &'static str) {
    match caso {
        Caso::AtribuirExistente
        | Caso::AtribuirAusente
        | Caso::AtribuirSemCatalogo
        | Caso::AtribuirConfirmado
        | Caso::AtribuirNaoConfirmado => ("--marcador", "501#Revisão"),
        Caso::RetirarAusente | Caso::RetirarSemMarcadores => ("--retirar", "502#Pendente"),
        Caso::RetirarPresente | Caso::RetirarConfirmado | Caso::RetirarNaoConfirmado => {
            ("--retirar", "501#Revisão")
        }
    }
}

pub(super) fn executar_caso_cli(
    pasta: &Pasta,
    arquivos: [&std::path::Path; 4],
    porta_proxy: u16,
    caso: Caso,
) -> Output {
    let (operacao, marcador) = pedido(caso);
    let mut argumentos = vec!["--json", "processo", "marcar", NUP, operacao, marcador];
    if matches!(
        caso,
        Caso::RetirarConfirmado
            | Caso::RetirarNaoConfirmado
            | Caso::AtribuirConfirmado
            | Caso::AtribuirNaoConfirmado
    ) {
        argumentos.push("--confirmar");
    }
    if matches!(
        caso,
        Caso::AtribuirAusente
            | Caso::AtribuirSemCatalogo
            | Caso::AtribuirConfirmado
            | Caso::AtribuirNaoConfirmado
    ) {
        argumentos.extend(["--texto", "Nova revisão"]);
    }
    executar_cli(pasta, arquivos, porta_proxy, &argumentos)
}
