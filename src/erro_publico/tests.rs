//! O derive compõe caminho, ação e status pela estrutura dos tipos.
use super::*;

#[derive(Debug, Causa)]
enum Interno {
    LinkAusente,
    #[causa(nome = "nome_historico")]
    Renomeado,
    #[causa(acao = "relogar")]
    SessaoEncerrada,
    #[causa(http)]
    Recusa(StatusHttpRecusado),
    Repetidos {
        _quantos: usize,
    },
}

#[derive(Debug, Causa)]
enum Operacao {
    #[causa(interna)]
    Tela(Interno),
    Folha,
}

#[test]
fn caminho_compoe_externo_e_interno() {
    let erro = Operacao::Tela(Interno::LinkAusente);
    assert_eq!(caminho_de(&erro), ["tela", "link_ausente"]);
    assert_eq!(caminho_de(&Operacao::Folha), ["folha"]);
    let erro = Operacao::Tela(Interno::Repetidos { _quantos: 2 });
    assert_eq!(caminho_de(&erro), ["tela", "repetidos"]);
}

#[test]
fn nome_explicito_vence_o_derivado() {
    assert_eq!(caminho_de(&Interno::Renomeado), ["nome_historico"]);
}

#[test]
fn acao_e_status_vem_do_interno() {
    let erro = Operacao::Tela(Interno::SessaoEncerrada);
    assert_eq!(erro.acao(), Acao::Relogar);
    assert_eq!(Operacao::Folha.acao(), Acao::Reportar);
    let erro = Operacao::Tela(Interno::Recusa(StatusHttpRecusado::sem_mensagem(503)));
    assert_eq!(erro.status_http().map(|s| s.status), Some(503));
    assert_eq!(Operacao::Folha.status_http(), None);
}

#[test]
fn json_publica_acao_causa_e_escrita() {
    let erro = Operacao::Tela(Interno::Recusa(StatusHttpRecusado::sem_mensagem(503)));
    let escrita = Escrita {
        enviada: false,
        aplicada: None,
    };
    let v = json_da_falha("processo.criar", &erro, "falhou", Some(&escrita));
    assert_eq!(v["acao"], "reportar");
    assert_eq!(v["operacao"], "processo.criar");
    assert_eq!(v["causa"], json!(["tela", "recusa"]));
    assert_eq!(v["subcodigo"], "recusa");
    assert_eq!(v["status_http"], 503);
    assert_eq!(v["escrita"], json!({"enviada": false, "aplicada": null}));
    assert_eq!(
        linha_da_falha(&erro, "falhou"),
        "IES: falhou [tela/recusa] (HTTP 503)"
    );
}
