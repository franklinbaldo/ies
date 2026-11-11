//! RED/GREEN do envio por script e dos ocultos reescritos pela operação.
//! HTML sintético modelado no `frmProcedimentoControlar` servido.
use super::tests::{instalacao, tela_iso};
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::tela_html::HtmlServido;

const OPERACAO_POR_SCRIPT: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoControlar,
    formulario: "frmProcedimentoControlar",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnAPaginaAtual"],
    campos_ocultos_reescritos: &["hdnAPaginaAtual", "hdnBPaginaAtual"],
    campos_usuario: &["txtBloco"],
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "tabelaControle",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

const HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn url() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"
    )
}

fn html(interior: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>TESTE</a>\
         <form id='frmProcedimentoControlar' method='post' \
         action='controlador.php?acao=procedimento_controlar&acao_origem=procedimento_controlar&infra_hash={HASH}'>\
         {interior}</form></html>"
    )
}

const INTERIOR: &str = "<input type='hidden' name='hdnAPaginaAtual' value='0'>\
    <input type='text' name='txtBloco' value=''>\
    <input type='hidden' name='hdnBPaginaAtual' value='0'>\
    <input type='hidden' name='hdnCriterios' value='aaaa'>";

fn derivar(
    interior: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = url();
    let pagina = HtmlServido::de_teste(&html(interior));
    derivar_submissao(
        &OPERACAO_POR_SCRIPT,
        &instalacao(),
        &tela_iso(&url, &pagina),
        Some("TESTE"),
        campos,
    )
}

#[test]
fn envio_por_script_dispensa_botao_e_nao_emite_nenhum() {
    let com_botao = format!("{INTERIOR}<input type='submit' name='sbmQualquer' value='x'>");
    for interior in [INTERIOR, com_botao.as_str()] {
        let submissao = derivar(interior, &[("txtBloco", "")]).unwrap();
        assert_eq!(
            submissao.nomes_dos_campos(),
            [
                "hdnAPaginaAtual",
                "txtBloco",
                "hdnBPaginaAtual",
                "hdnCriterios"
            ]
        );
    }
}

#[test]
fn oculto_reescrito_troca_so_o_valor_na_posicao_servida() {
    let submissao = derivar(INTERIOR, &[("hdnAPaginaAtual", "3"), ("txtBloco", "")]).unwrap();
    assert_eq!(
        std::str::from_utf8(submissao.corpo()).unwrap(),
        "hdnAPaginaAtual=3&txtBloco=&hdnBPaginaAtual=0&hdnCriterios=aaaa"
    );
}

#[test]
fn oculto_reescrito_nao_informado_sai_como_servido() {
    let submissao = derivar(INTERIOR, &[("hdnBPaginaAtual", "1")]).unwrap();
    assert_eq!(
        std::str::from_utf8(submissao.corpo()).unwrap(),
        "hdnAPaginaAtual=0&hdnBPaginaAtual=1&hdnCriterios=aaaa"
    );
}

#[test]
fn oculto_fora_da_lista_de_reescritos_e_recusado() {
    assert_eq!(
        derivar(INTERIOR, &[("hdnCriterios", "bbbb")]).err(),
        Some(ErroSubmissaoFormulario::CampoForaDaListaDaOperacao {
            campo: "hdnCriterios".to_owned()
        })
    );
}

#[test]
fn oculto_reescrito_nao_publicado_e_recusado() {
    let sem_b = "<input type='hidden' name='hdnAPaginaAtual' value='0'>";
    assert_eq!(
        derivar(sem_b, &[("hdnBPaginaAtual", "1")]).err(),
        Some(ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario {
            campo: "hdnBPaginaAtual".to_owned()
        })
    );
}

#[test]
fn oculto_reescrito_informado_duas_vezes_e_recusado() {
    assert_eq!(
        derivar(
            INTERIOR,
            &[("hdnAPaginaAtual", "1"), ("hdnAPaginaAtual", "2")]
        )
        .err(),
        Some(ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez {
            campo: "hdnAPaginaAtual".to_owned()
        })
    );
}

#[test]
fn valor_do_oculto_reescrito_nao_aparece_no_debug() {
    let submissao = derivar(INTERIOR, &[("hdnAPaginaAtual", "987654")]).unwrap();
    let texto = format!("{submissao:?}");
    assert!(!texto.contains("987654"));
    assert!(!texto.contains(HASH));
    assert!(!texto.contains("aaaa"));
}
