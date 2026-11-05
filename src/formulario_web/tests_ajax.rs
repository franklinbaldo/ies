//! RED/GREEN da consulta AJAX declarada pela tela (autocomplete da tela de
//! envio): URL relativa resolvida contra a tela, só `controlador_ajax.php`
//! da instalação com a `acao_ajax` da consulta, corpo na ordem dada e no
//! charset da tela. Hosts e hashes inventados.
use super::super::tests::instalacao;
use super::super::{
    CodificacaoDaTela, EfeitoAvaliado, ErroSubmissaoFormulario as E, EstadoDeSessao, TelaServida,
};
use super::{ConsultaAjax, derivar_consulta_ajax};
use crate::erro_html::ErroHtml;

const HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

const AUTOCOMPLETAR: ConsultaAjax = ConsultaAjax {
    acao_ajax: "unidade_auto_completar_envio_processo",
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
) -> Result<super::SubmissaoDerivada, E> {
    let tela_url = tela_url();
    let tela = TelaServida {
        url: &tela_url,
        html: "",
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
        (outra_acao.as_str(), E::AcaoDoFormularioDiferenteDaOperacao),
        (sem_hash.as_str(), E::AcaoDoFormularioSemInfraHashUnico),
        (com_acao.as_str(), E::AcaoDoFormularioDiferenteDaOperacao),
        (
            "controlador.php?acao_ajax=unidade_auto_completar_envio_processo",
            E::Html(ErroHtml::valor("acao_do_formulario_fora_da_instalacao")),
        ),
        (
            "https://outro.gov.br/sei/controlador_ajax.php",
            E::Html(ErroHtml::valor("acao_do_formulario_fora_da_instalacao")),
        ),
        (
            "/sei/controlador_ajax.php",
            E::Html(ErroHtml::valor("acao_do_formulario_fora_da_instalacao")),
        ),
        (
            "../sei/controlador_ajax.php",
            E::Html(ErroHtml::valor("acao_do_formulario_fora_da_instalacao")),
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
        Err(E::CampoForaDaListaDaOperacao {
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
        Err(E::EfeitoDaOperacaoNaoAvaliado)
    );
}
