//! RED/GREEN da colheita no login: o Controle que o login já obteve (o da
//! sessão gravada válida ou o do fim do login novo) alimenta o índice,
//! com a sequência de pedidos **exatamente** a de sempre e a chave do
//! cookie e da URL gravados.
use super::tests_apoio::*;
use super::{Conexao, entrar_colhendo, geracao_da_sessao};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;

const CONTEXTO: &str = "infra_sistema=100000100&amp;infra_unidade_atual=110000849";

fn href(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&amp;acao_origem=procedimento_controlar&amp;id_procedimento={id}&amp;{CONTEXTO}&amp;infra_hash={}",
        "c".repeat(64)
    )
}

/// O Controle de `tests_apoio` com uma linha em Recebidos.
pub(super) fn controle_com_linha() -> String {
    controle("UNIDADE-TESTE").replace(
        "<caption>0 registros:</caption></table>\n<table id=\"tblProcessosGerados\"",
        &format!(
            "<caption>1 registros:</caption><tr id=\"P77\"><td><a href=\"{}\" class=\"processoVisualizado\">NUP-LOGIN</a></td></tr></table>\n<table id=\"tblProcessosGerados\"",
            href("77")
        ),
    )
}

fn indice(ambiente: &Ambiente) -> IndiceLocal {
    IndiceLocal::em(ambiente.dir.join("indice").join("ies.sqlite3"))
}

fn conferir_colheita(ambiente: &Ambiente, indice: &IndiceLocal) {
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let cookie = Ambiente::ler(&ambiente.arquivos.cookie);
    let geracao = geracao_da_sessao(cookie.trim());
    assert_eq!(
        indice.id_por_nup(&instalacao, "NUP-LOGIN").as_deref(),
        Some("77")
    );
    let chave = crate::indice_local::ChaveDoLink {
        geracao_sessao: &geracao,
        unidade: "110000849",
        acao: "procedimento_trabalhar",
        alvo: "77",
    };
    assert_eq!(
        indice.link(&instalacao, chave).map(|u| u.url().to_owned()),
        Some(format!("{ORIGEM}/{}", href("77").replace("&amp;", "&")))
    );
}

#[test]
fn login_novo_colhe_o_controle_sem_pedido_a_mais() {
    let ambiente = Ambiente::novo();
    let indice = indice(&ambiente);
    let mut roteiro = Roteiro::novo(vec![
        r302(LOGIN_SIP, &["PHPSESSID=raiz; path=/"]),
        r200(&tela_acessar(), &[]),
        r302(
            "https://sei.exemplo.gov.br/sei/inicializar.php",
            &["PHPSESSID=autenticada; path=/"],
        ),
        r302(
            &url_do_controle().replace("https://sei.exemplo.gov.br/sei/", ""),
            &[],
        ),
        r200(&controle_com_linha(), &[]),
    ]);
    let resultado = entrar_colhendo(&ambiente.pedido(false), &indice, &mut roteiro).unwrap();
    assert_eq!(resultado.conexao, Conexao::LoginRealizado);
    assert_eq!(resultado.requisicoes, 5);
    assert_eq!(
        roteiro.sequencia(),
        [
            format!("GET {RAIZ}"),
            format!("GET {LOGIN_SIP}"),
            format!("POST {LOGIN_SIP}"),
            "GET https://sei.exemplo.gov.br/sei/inicializar.php".to_owned(),
            format!("GET {}", url_do_controle()),
        ]
    );
    assert!(roteiro.esgotado());
    conferir_colheita(&ambiente, &indice);
}

#[test]
fn sessao_gravada_valida_colhe_o_controle_do_unico_get() {
    let ambiente = Ambiente::novo();
    let indice = indice(&ambiente);
    ambiente.gravar_sessao(&url_do_controle(), "PHPSESSID=gravada");
    let mut roteiro = Roteiro::novo(vec![r200(&controle_com_linha(), &[])]);
    let resultado = entrar_colhendo(&ambiente.pedido(false), &indice, &mut roteiro).unwrap();
    assert_eq!(resultado.conexao, Conexao::SessaoExistenteValida);
    assert_eq!(resultado.requisicoes, 1);
    assert_eq!(roteiro.sequencia(), [format!("GET {}", url_do_controle())]);
    conferir_colheita(&ambiente, &indice);
}

#[test]
fn unidade_diferente_da_pedida_nao_colhe() {
    let ambiente = Ambiente::novo();
    let indice = indice(&ambiente);
    ambiente.gravar_sessao(&url_do_controle(), "PHPSESSID=gravada");
    let mut roteiro = Roteiro::novo(vec![r200(&controle_com_linha(), &[])]);
    let mut pedido = ambiente.pedido(false);
    pedido.unidade_esperada = Some("OUTRA");
    assert!(entrar_colhendo(&pedido, &indice, &mut roteiro).is_err());
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    assert_eq!(indice.id_por_nup(&instalacao, "NUP-LOGIN"), None);
}
