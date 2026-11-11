//! RED/GREEN do `<select multiple>`: um par por opção, na posição do `select`.
//! O primitivo não escolhe opções; só aceita a repetição que o HTML permite.
use super::tests::{instalacao, tela_iso, url_servida};
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::tela_html::HtmlServido;

const OPERACAO_COM_SELECAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProtocoloPesquisar,
    formulario: "frmPesquisaProtocolo",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmPesquisar"),
    campos_ocultos_exigidos: &["hdnInicio"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["selOrgaoPesquisa[]", "txtProtocoloPesquisa"],
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "pesquisaResultado",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

fn pagina(select: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
         <form id='frmPesquisaProtocolo' method='post' \
         action='controlador.php?acao=protocolo_pesquisar&amp;infra_hash={}'>\
         <button type='submit' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
         {select}\
         <input type='text' name='txtProtocoloPesquisa' value=''>\
         <input type='hidden' name='hdnInicio' value='0'></form></html>",
        "c".repeat(64)
    )
}

fn derivar(
    html: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = url_servida();
    derivar_submissao(
        &OPERACAO_COM_SELECAO,
        &instalacao(),
        &tela_iso(&url, &HtmlServido::de_teste(html)),
        Some("UNIDADE TESTE"),
        campos,
    )
}

const MULTIPLO: &str = "<select name='selOrgaoPesquisa[]' multiple>\
    <option value='1' selected>A</option><option value='2' selected>B</option></select>";

#[test]
fn select_multiple_emite_um_par_por_valor_na_posicao_do_select() {
    let submissao = derivar(
        &pagina(MULTIPLO),
        &[
            ("txtProtocoloPesquisa", "0001"),
            ("selOrgaoPesquisa[]", "1"),
            ("selOrgaoPesquisa[]", "2"),
        ],
    )
    .unwrap();
    assert_eq!(
        submissao.corpo(),
        b"sbmPesquisar=Pesquisar&selOrgaoPesquisa%5B%5D=1&selOrgaoPesquisa%5B%5D=2\
          &txtProtocoloPesquisa=0001&hdnInicio=0"
    );
    assert_eq!(
        submissao.nomes_dos_campos(),
        [
            "sbmPesquisar",
            "selOrgaoPesquisa[]",
            "selOrgaoPesquisa[]",
            "txtProtocoloPesquisa",
            "hdnInicio"
        ]
    );
}

#[test]
fn select_sem_multiple_continua_recusando_repeticao() {
    let simples = MULTIPLO.replace(" multiple", "");
    assert_eq!(
        derivar(
            &pagina(&simples),
            &[("selOrgaoPesquisa[]", "1"), ("selOrgaoPesquisa[]", "2")],
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez {
            campo: "selOrgaoPesquisa[]".to_owned()
        }
    );
}

#[test]
fn texto_com_mesmo_nome_de_select_multiple_nao_herda_a_excecao() {
    let html = pagina("<input type='text' name='selOrgaoPesquisa[]' value=''>");
    assert_eq!(
        derivar(
            &html,
            &[("selOrgaoPesquisa[]", "1"), ("selOrgaoPesquisa[]", "2")]
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez {
            campo: "selOrgaoPesquisa[]".to_owned()
        }
    );
}

#[test]
fn select_multiple_desabilitado_nao_e_publicado() {
    // A forma `multiple disabled` (corrompida pelo mapa do tl 0.7.8) está
    // coberta em `tests_atributos`; aqui `disabled` vem por último.
    let desabilitado = MULTIPLO.replace(" multiple>", " multiple='multiple' disabled>");
    assert_eq!(
        derivar(&pagina(&desabilitado), &[("selOrgaoPesquisa[]", "1")]).unwrap_err(),
        ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario {
            campo: "selOrgaoPesquisa[]".to_owned()
        }
    );
}
