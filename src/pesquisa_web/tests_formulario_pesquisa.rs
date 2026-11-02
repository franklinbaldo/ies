//! RED/GREEN da operação de pesquisa de protocolo: HTML sintético com os nomes observados.
use super::PESQUISAR_PROTOCOLO;
use crate::formulario_web::tests::{instalacao, tela_iso, url_servida};
use crate::formulario_web::{
    EfeitoAvaliado, ErroSubmissaoFormulario, EstadoDeSessao, derivar_submissao,
};

// Estrutura sintética com os nomes observados; nenhum valor real.
fn formulario_observado() -> String {
    use std::fmt::Write as _;
    let mut ocultos = String::new();
    for nome in PESQUISAR_PROTOCOLO.campos_ocultos_exigidos {
        write!(
            ocultos,
            "<input type='hidden' id='{nome}' name='{nome}' value=''>"
        )
        .unwrap();
    }
    format!(
        "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
         <form id='frmPesquisaProtocolo' name='frmPesquisaProtocolo' method='post' \
         onsubmit='return onSubmitForm();' \
         action='controlador.php?acao=protocolo_pesquisar&amp;acao_origem=protocolo_pesquisar&amp;infra_hash={}#ancoraBarraPesquisa'>\
         <button type='submit' id='sbmPesquisar' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
         <button type='button' id='sbmReset' name='sbmReset' value='Limpar'>Limpar</button>\
         <input type='radio' id='optProcessos' name='rdoPesquisarEm' value='P'>\
         <input type='radio' id='optDocumentos' name='rdoPesquisarEm' value='D'>\
         <input type='text' id='txtProtocoloPesquisa' name='txtProtocoloPesquisa' value=''>\
         {ocultos}</form>\
         <input type='hidden' id='hdnInfraPrefixoCookie' name='hdnInfraPrefixoCookie' value='x'></html>",
        "b".repeat(64)
    )
}

#[test]
fn pesquisa_de_protocolo_declara_estado_de_sessao_lembrado() {
    assert_eq!(
        PESQUISAR_PROTOCOLO.efeito,
        EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PaginaOuFiltroLembrado
        }
    );
}

#[test]
fn formulario_com_nomes_observados_deriva_ocultos_campos_e_botao() {
    let html = formulario_observado();
    let url = url_servida();
    let submissao = derivar_submissao(
        &PESQUISAR_PROTOCOLO,
        &instalacao(),
        &tela_iso(&url, &html),
        "UNIDADE TESTE",
        &[("rdoPesquisarEm", "P"), ("txtProtocoloPesquisa", "0001")],
    )
    .unwrap();
    let mut esperados = vec!["sbmPesquisar", "rdoPesquisarEm", "txtProtocoloPesquisa"];
    esperados.extend(PESQUISAR_PROTOCOLO.campos_ocultos_exigidos);
    assert_eq!(submissao.nomes_dos_campos(), esperados);
}

#[test]
fn formulario_sem_um_oculto_observado_e_recusado() {
    let html = formulario_observado().replace("name='hdnInicio'", "name='outro'");
    let url = url_servida();
    assert_eq!(
        derivar_submissao(
            &PESQUISAR_PROTOCOLO,
            &instalacao(),
            &tela_iso(&url, &html),
            "UNIDADE TESTE",
            &[],
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::CampoOcultoAusenteNoFormulario {
            campo: "hdnInicio".to_owned()
        }
    );
}
