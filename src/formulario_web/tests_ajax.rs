//! RED/GREEN da consulta AJAX declarada pela tela (autocomplete da tela de
//! envio): URL relativa resolvida contra a tela, só `controlador_ajax.php`
//! da instalação com a `acao_ajax` da consulta, corpo na ordem dada e no
//! charset da tela. Hosts e hashes inventados.
use super::super::tests::instalacao;
use super::super::{
    CodificacaoDaTela, EfeitoAvaliado, ErroSubmissaoFormulario, EstadoDeSessao, TelaServida,
};
use super::{ConsultaAjax, derivar_consulta_ajax};
use crate::tela_html::HtmlServido;

const HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

const AUTOCOMPLETAR: ConsultaAjax = ConsultaAjax {
    acao_ajax: crate::acao_do_sei::AcaoAjaxDoSei::UnidadeAutoCompletarEnvioProcesso,
    campos: &["palavras_pesquisa", "id_orgao", "unidade_atual"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado { evidencia: "teste" },
};

fn tela_url() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10&infra_hash={HASH}"
    )
}

fn declarada() -> String {
    format!(
        "controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}"
    )
}

fn derivar(
    consulta: &ConsultaAjax,
    url: &str,
    campos: &[(&str, &str)],
) -> Result<super::SubmissaoDerivada, ErroSubmissaoFormulario> {
    let tela_url = tela_url();
    let tela = TelaServida {
        url: &tela_url,
        html: &HtmlServido::de_teste(""),
        codificacao: CodificacaoDaTela::Iso88591,
    };
    derivar_consulta_ajax(consulta, &instalacao(), &tela, url, campos)
}

#[test]
fn corpo_na_ordem_do_script_no_charset_da_tela() {
    let s = derivar(
        &AUTOCOMPLETAR,
        &declarada(),
        &[
            ("palavras_pesquisa", "ÁREA/GR"),
            ("id_orgao", ""),
            ("unidade_atual", "0"),
        ],
    )
    .unwrap();
    assert_eq!(
        s.url(),
        format!("https://sei.exemplo.gov.br/sei/{}", declarada())
    );
    assert_eq!(s.referer(), tela_url());
    assert_eq!(
        s.corpo(),
        b"palavras_pesquisa=%C1REA%2FGR&id_orgao=&unidade_atual=0"
    );
    assert!(!format!("{s:?}").contains(HASH));
}

#[test]
fn recusa_sem_rede_o_que_a_tela_nao_declarou() {
    let campos = [("palavras_pesquisa", "X")];
    let outra_acao = declarada().replace("envio_processo", "todas");
    let sem_hash = declarada().replace(&format!("&infra_hash={HASH}"), "");
    let com_acao = format!("{}&acao=procedimento_enviar", declarada());
    for (url, erro) in [
        (
            outra_acao.as_str(),
            ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao,
        ),
        (
            sem_hash.as_str(),
            ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico,
        ),
        (
            com_acao.as_str(),
            ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao,
        ),
        (
            "controlador.php?acao_ajax=unidade_auto_completar_envio_processo",
            ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao,
        ),
        (
            "https://outro.gov.br/sei/controlador_ajax.php",
            ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao,
        ),
        (
            "/sei/controlador_ajax.php",
            ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao,
        ),
        (
            "../sei/controlador_ajax.php",
            ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao,
        ),
    ] {
        assert_eq!(
            derivar(&AUTOCOMPLETAR, url, &campos).map(|_| ()),
            Err(erro),
            "{url}"
        );
    }
    assert_eq!(
        derivar(&AUTOCOMPLETAR, &declarada(), &[("txtUnidade", "X")]).map(|_| ()),
        Err(ErroSubmissaoFormulario::CampoForaDaListaDaOperacao {
            campo: "txtUnidade".into()
        })
    );
    let escrita = ConsultaAjax {
        efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::UnidadeAtual,
        },
        ..AUTOCOMPLETAR
    };
    assert_eq!(
        derivar(&escrita, &declarada(), &campos).map(|_| ()),
        Err(ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado)
    );
}

#[test]
fn consulta_ajax_ilegivel_preserva_causa() {
    use crate::consulta_do_controlador::ErroConsultaDoControlador as Causa;
    for (par, causa) in [
        ("sem_igual", Causa::ParSemIgual),
        ("Chave=x", Causa::ChaveDeConsultaNaoCanonica),
        ("chave=x;y", Causa::SeparadorDeConsultaAmbiguo),
    ] {
        let url = format!("{}&{par}", declarada());
        assert_eq!(
            derivar(&AUTOCOMPLETAR, &url, &[]).map(|_| ()),
            Err(ErroSubmissaoFormulario::ConsultaDaAcaoIlegivel(causa))
        );
    }
}

const TEXTO_PADRAO: ConsultaAjax = ConsultaAjax {
    acao_ajax: crate::acao_do_sei::AcaoAjaxDoSei::TextoPadraoAutoCompletar,
    campos: &["palavras_pesquisa"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "HTML Todos documento_gerar; transporte original ainda não observado",
    },
};

fn texto_padrao_declarada() -> String {
    format!(
        "controlador_ajax.php?acao_ajax=texto_padrao_auto_completar&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}"
    )
}

#[test]
fn texto_padrao_contrasta_charset_da_tela_com_encode_uri_component() {
    // A captura HTML do Todos (caabe699, documento_gerar.html) monta
    // palavras_pesquisa com encodeURIComponent. Isto caracteriza o
    // transporte atual do IES; não é replay de uma requisição SEI.
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_gerar&infra_hash={HASH}"
    );
    let html = HtmlServido::de_teste("");
    for (codificacao, esperado) in [
        (CodificacaoDaTela::Iso88591, "palavras_pesquisa=Of%EDcio"),
        (CodificacaoDaTela::Utf8, "palavras_pesquisa=Of%C3%ADcio"),
    ] {
        let tela = TelaServida {
            url: &url,
            html: &html,
            codificacao,
        };
        let s = derivar_consulta_ajax(
            &TEXTO_PADRAO,
            &instalacao(),
            &tela,
            &texto_padrao_declarada(),
            &[("palavras_pesquisa", "Ofício")],
        )
        .unwrap();
        assert_eq!(s.corpo(), esperado.as_bytes());
        assert_eq!(s.referer(), url);
    }
}

#[test]
fn texto_padrao_nao_aceita_acao_de_contato_nem_campo_de_cadastro() {
    let outra = texto_padrao_declarada().replace(
        "texto_padrao_auto_completar",
        "contato_auto_completar_contexto_RI1225",
    );
    assert_eq!(
        derivar(&TEXTO_PADRAO, &outra, &[("palavras_pesquisa", "Oficio")]).map(|_| ()),
        Err(ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao)
    );
    assert_eq!(
        derivar(
            &TEXTO_PADRAO,
            &texto_padrao_declarada(),
            &[("hdnIdTextoPadrao", "7")],
        )
        .map(|_| ()),
        Err(ErroSubmissaoFormulario::CampoForaDaListaDaOperacao {
            campo: "hdnIdTextoPadrao".into()
        })
    );
}
