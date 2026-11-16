//! RED/GREEN da troca para a visão completa: regra servida conferida
//! literalmente e POST derivado pelo primitivo de formulário.
use super::tests_apoio::*;
use super::*;
use crate::arvore_processo::tests::NUP;
use crate::formulario_web::{CodificacaoDaTela, EfeitoAvaliado, TelaServida, derivar_submissao};
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;

const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";

#[test]
fn regra_servida_na_visao_resumida_e_aceita() {
    let html = pagina_do_historico(NUP, "10", 'R', 2, 2);
    assert_eq!(conferir_regra_de_ver_historico(&html), Ok(()));
}

#[test]
fn regra_alterada_e_recusa() {
    let html = pagina_do_historico(NUP, "10", 'R', 2, 2);
    let casos = [
        (
            html.replace("function verHistorico(valor){", "function verHistorico(v){"),
            super::ErroAndamentos::RegraDeVerHistoricoAusenteNaPagina,
        ),
        (
            html.replace(
                "document.getElementById('hdnTipoHistorico').value = valor;",
                "document.getElementById('hdnTipoHistorico').value = 'P';",
            ),
            super::ErroAndamentos::RegraDeVerHistoricoAusenteNaPagina,
        ),
        (
            html.replace(
                "document.getElementById('frmProcedimentoHistorico').submit();",
                "infraSubmit();",
            ),
            super::ErroAndamentos::RegraDeVerHistoricoAusenteNaPagina,
        ),
    ];
    for (trocado, esperado) in casos {
        assert_eq!(conferir_regra_de_ver_historico(&trocado), Err(esperado));
    }
}

#[test]
fn post_reenvia_os_ocultos_servidos_com_o_tipo_p_e_sem_o_botao_de_escrita() {
    let html = pagina_do_historico(NUP, "10", 'R', 2, 2);
    let url = format!("{ORIGEM}/{}", link_do_historico("10"));
    let tela = TelaServida {
        url: &url,
        html: &HtmlServido::de_teste(&html),
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let submissao = derivar_submissao(
        &VER_HISTORICO_COMPLETO,
        &instalacao,
        &tela,
        None,
        &[(CAMPO_TIPO, "P")],
    )
    .unwrap();
    assert_eq!(
        submissao.url(),
        format!("{ORIGEM}/{}", action_do_historico("10"))
    );
    assert_eq!(
        String::from_utf8(submissao.corpo().to_vec()).unwrap(),
        corpo_ver_completo(2)
    );
    assert!(
        !submissao
            .nomes_dos_campos()
            .iter()
            .any(|n| n.starts_with("btn"))
    );
    assert_eq!(submissao.referer(), url);
    assert!(matches!(
        submissao.efeito(),
        EfeitoAvaliado::AlteraEstadoDaSessao { .. }
    ));
}
