//! BDD-633-02 com a CLI real contra anti-IES e SQLite.
use super::*;

/// BDD-633-02: CLI real, TLS local, modal do anti-IES e SQLite.
/// Caso sintetico, sem corpo da modal observado em cassette.
#[test]
fn bdd_633_02_cli_real_recusa_modal_sem_credenciais_e_sem_post() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::ValidacaoExigida);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    // A recusa tipada do SEI, com a falta do arquivo de credenciais junto.
    falha(&saida, "ValidacaoDeAcessoSemArquivoDeCredenciais");
    let linhas = linhas_do_erro(&saida);
    // A causa é a primeira da cadeia (`Caused by: 0: …`).
    assert_eq!(
        linhas.get(3).map(String::as_str),
        Some("    0: Tela(ProcessoExigeValidacaoDeAcesso)"),
        "{linhas:?}"
    );
    assert_eq!(
        pedidos,
        ["GET procedimento_controlar", "GET procedimento_trabalhar"]
    );
}
