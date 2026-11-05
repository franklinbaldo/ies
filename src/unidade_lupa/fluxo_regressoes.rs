//! Regressões de fronteira da leitura Web da lupa (#244).
//! Todas as URLs, hashes e respostas HTTP deste arquivo são sintéticos.
use super::*;

const ORIGEM: &str = "https://sei.example/sei";
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn link(query: &str) -> String {
    format!("<a href='https://sei.example/sei/controlador.php?{query}'>Unidades</a>")
}

fn controle() -> String {
    format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}")
}

#[test]
fn link_publicado_para_outra_acao_pode_ser_lido_com_hash_valido() {
    let consulta = format!("acao=grupo_unidade_listar&infra_hash={HASH}");
    assert_eq!(
        destino(&link(&consulta), &controle(), ORIGEM, "grupo_unidade_listar"),
        Ok(format!("{ORIGEM}/controlador.php?{consulta}"))
    );
}

#[test]
fn link_com_hash_repetido_valido_ou_invalido_eh_recusado() {
    for adicional in [HASH, "invalido", ""] {
        let consulta = format!("acao=grupo_unidade_listar&infra_hash={HASH}&infra_hash={adicional}");
        assert_eq!(
            destino(&link(&consulta), &controle(), ORIGEM, "grupo_unidade_listar"),
            Err(FalhaDaLupa::LinkAusente),
        );
    }
}

#[test]
fn consulta_malformada_ou_com_chave_nao_canonica_eh_recusada() {
    let exemplos = [
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&sem_igual"),
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&Acao=x"),
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&outra=x;y"),
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&acao=outra"),
    ];
    for consulta in exemplos {
        assert_eq!(
            destino(&link(&consulta), &controle(), ORIGEM, "grupo_unidade_listar"),
            Err(FalhaDaLupa::LinkAusente),
            "uma consulta malformada nunca deve autorizar a leitura"
        );
    }
}

#[test]
fn recusas_http_preservam_o_status_e_a_mensagem_saneada() {
    let recusa = crate::transporte::StatusHttpRecusado {
        status: 503,
        mensagem_do_servidor: Some("indisponivel".to_owned()),
    };
    let falha = FalhaDaLupa::Transporte(
        FalhaTransporte::StatusHttpDeErroDoServidor(recusa.clone()),
    );
    assert_eq!(falha.subcodigo(), "status_http_de_erro_do_servidor");
    assert_eq!(falha.status_http(), Some(&recusa));
}
