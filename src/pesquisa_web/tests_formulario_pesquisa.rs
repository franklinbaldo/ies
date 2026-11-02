//! RED/GREEN da operação de pesquisa de protocolo: HTML sintético com os
//! nomes e a estrutura observados (rádio `D` marcado, órgão `multiple`).
use super::*;
use crate::formulario_web::tests::{instalacao, tela_iso, url_servida};
use ErroRegraDoFormularioDePesquisa as R;

const OPCOES_DE_ORGAO: &str = "<option value='1' selected>A</option>\
    <option value='2'>B</option><option value='3' selected='selected'>C</option>";

fn select_de_orgao(opcoes: &str) -> String {
    format!(
        "<select id='selOrgaoPesquisa' name='selOrgaoPesquisa[]' multiple='multiple' \
         onchange='tratarSelecaoOrgao()'>{opcoes}</select>"
    )
}

fn pagina(interior: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
         <form id='frmPesquisaProtocolo' name='frmPesquisaProtocolo' method='post' \
         onsubmit='return onSubmitForm();' \
         action='controlador.php?acao=protocolo_pesquisar&amp;acao_origem=protocolo_pesquisar&amp;infra_hash={}#ancoraBarraPesquisa'>\
         {interior}</form>\
         <select name='selOrgaoPesquisa[]' multiple><option value='9' selected>fora</option></select>\
         <input type='hidden' id='hdnInfraPrefixoCookie' name='hdnInfraPrefixoCookie' value='x'></html>",
        "b".repeat(64)
    )
}

// Ordem e tipos dos controles como servidos; nenhum valor real.
fn interior(orgao: &str) -> String {
    format!(
        "<input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='1'>\
         <button type='submit' id='sbmPesquisar' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
         <button type='button' id='sbmReset' name='sbmReset' value='Limpar'>Limpar</button>\
         <input type='radio' id='optProcessos' name='rdoPesquisarEm' value='P' onclick='tratarTipoPesquisa()'>\
         <input type='checkbox' id='chkSinConsiderarDocumentos' name='chkSinConsiderarDocumentos' value='S'>\
         <input type='radio' id='optDocumentos' name='rdoPesquisarEm' value='D' checked='checked'>\
         <input type='checkbox' id='chkSinDocumentosGerados' name='chkSinDocumentosGerados' value='S' checked>\
         <input type='checkbox' id='chkSinDocumentosRecebidos' name='chkSinDocumentosRecebidos' value='S' checked>\
         <input type='text' id='q' name='q' value=''>{orgao}\
         <input type='checkbox' id='chkSinRestringirOrgao' name='chkSinRestringirOrgao' value='S'>\
         <input type='hidden' id='hdnIdUnidade' name='hdnIdUnidade' value=''>\
         <input type='hidden' id='hdnIdAssunto' name='hdnIdAssunto' value=''>\
         <input type='hidden' id='hdnIdAssinante' name='hdnIdAssinante' value=''>\
         <input type='hidden' id='hdnIdContato' name='hdnIdContato' value=''>\
         <input type='text' id='txtProtocoloPesquisa' name='txtProtocoloPesquisa' value=''>\
         <select id='selTipoProcedimentoPesquisa' name='selTipoProcedimentoPesquisa'><option value='null'>-</option></select>\
         <input type='hidden' id='hdnIdUsuarioGerador1' name='hdnIdUsuarioGerador1' value=''>\
         <input type='hidden' id='hdnIdUsuarioGerador2' name='hdnIdUsuarioGerador2' value=''>\
         <input type='hidden' id='hdnIdUsuarioGerador3' name='hdnIdUsuarioGerador3' value=''>\
         <select id='selData' name='selData'><option value='I' selected>I</option></select>\
         <input type='hidden' id='hdnInicio' name='hdnInicio' value='0'>"
    )
}

fn observado() -> String {
    pagina(&interior(&select_de_orgao(OPCOES_DE_ORGAO)))
}

fn derivar(html: &str) -> Result<SubmissaoDerivada, ErroPesquisaProtocolo> {
    let url = url_servida();
    derivar_pesquisa_de_protocolo(
        &instalacao(),
        &tela_iso(&url, html),
        "UNIDADE TESTE",
        "0001",
    )
}

fn regra(html: &str) -> R {
    match derivar(html).unwrap_err() {
        ErroPesquisaProtocolo::RegraDoFormulario(erro) => erro,
        outro => panic!("esperada regra do formulário, veio {outro:?}"),
    }
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
fn deriva_opcao_de_processos_e_orgaos_selecionados_servidos_na_ordem_do_documento() {
    let submissao = derivar(&observado()).unwrap();
    assert_eq!(
        String::from_utf8(submissao.corpo().to_vec()).unwrap(),
        "hdnInfraTipoPagina=1&sbmPesquisar=Pesquisar&rdoPesquisarEm=P\
         &selOrgaoPesquisa%5B%5D=1&selOrgaoPesquisa%5B%5D=3\
         &hdnIdUnidade=&hdnIdAssunto=&hdnIdAssinante=&hdnIdContato=\
         &txtProtocoloPesquisa=0001\
         &hdnIdUsuarioGerador1=&hdnIdUsuarioGerador2=&hdnIdUsuarioGerador3=&hdnInicio=0"
    );
}

#[test]
fn opcao_de_processos_ausente_repetida_sem_valor_ou_com_outro_nome_e_recusada() {
    let sem = observado().replace("id='optProcessos'", "id='outro'");
    assert_eq!(regra(&sem), R::OpcaoDeProcessosAusenteNaTela);
    let repetida = observado().replace(
        "<input type='checkbox' id='chkSinConsiderarDocumentos'",
        "<input type='radio' id='optProcessos' name='rdoPesquisarEm' value='P'>\
         <input type='checkbox' id='chkSinConsiderarDocumentos'",
    );
    assert_eq!(regra(&repetida), R::OpcoesDeProcessosRepetidasNaTela);
    let sem_valor = observado().replace("name='rdoPesquisarEm' value='P'", "name='rdoPesquisarEm'");
    assert_eq!(regra(&sem_valor), R::OpcaoDeProcessosSemValorNaTela);
    let outro_nome = observado().replace(
        "id='optProcessos' name='rdoPesquisarEm'",
        "id='optProcessos' name='rdoOutro'",
    );
    assert_eq!(
        regra(&outro_nome),
        R::OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm
    );
}

#[test]
fn assinante_oculto_preenchido_e_recusado_na_pesquisa_em_processos() {
    let html = observado().replace(
        "name='hdnIdAssinante' value=''",
        "name='hdnIdAssinante' value='5'",
    );
    assert_eq!(
        regra(&html),
        R::AssinanteOcultoPreenchidoNaPesquisaEmProcessos
    );
}

#[test]
fn seletor_de_orgao_ausente_repetido_ou_sem_multiple_e_recusado() {
    // O `select` fora do `<form>` em `pagina` não conta.
    assert_eq!(
        regra(&pagina(&interior(""))),
        R::SeletorDeOrgaoDePesquisaAusenteNaTela
    );
    let dois = format!(
        "{}{}",
        select_de_orgao(OPCOES_DE_ORGAO),
        select_de_orgao(OPCOES_DE_ORGAO)
    );
    assert_eq!(
        regra(&pagina(&interior(&dois))),
        R::SeletoresDeOrgaoDePesquisaRepetidosNaTela
    );
    let simples = observado().replace(" multiple='multiple'", "");
    assert_eq!(
        regra(&simples),
        R::SeletorDeOrgaoDePesquisaSemMultiplaEscolha
    );
}

#[test]
fn orgao_sem_selecao_servida_ou_opcao_sem_valor_e_recusado_sem_inventar_orgao() {
    let nenhum = select_de_orgao("<option value='1'>A</option><option value='2'>B</option>");
    assert_eq!(
        regra(&pagina(&interior(&nenhum))),
        R::OrgaoDePesquisaSemOpcaoSelecionadaNaTela
    );
    let sem_valor = select_de_orgao("<option selected>A</option>");
    assert_eq!(
        regra(&pagina(&interior(&sem_valor))),
        R::OpcaoDeOrgaoSelecionadaSemValorNaTela
    );
    let entidade = select_de_orgao("<option value='1&desconhecida;' selected>A</option>");
    assert_eq!(
        regra(&pagina(&interior(&entidade))),
        R::EntidadeHtmlDesconhecidaNaRegra
    );
}

/// Forma bruta servida nas quatro instalações: `multiple` sem valor antes
/// de `id`. O defeito do tl 0.7.8 (`id` lido como `d`) foi corrigido pelo
/// tokenizador `html_atributos`; o `select` continua localizado pelo
/// `name`, que é o nome efetivamente serializado no POST.
#[test]
fn forma_bruta_observada_do_select_com_multiple_sem_valor_e_lida() {
    let bruto = "<select style=\"display: none\" multiple id=\"selOrgaoPesquisa\" \
                 name=\"selOrgaoPesquisa[]\" onchange=\"tratarSelecaoOrgao()\" \
                 class=\"infraSelect multipleSelect\" tabindex=\"509\">\
                 <option value=\"4\" selected=\"selected\">A</option></select>";
    let submissao = derivar(&pagina(&interior(bruto))).unwrap();
    let corpo = String::from_utf8(submissao.corpo().to_vec()).unwrap();
    assert!(corpo.contains("&selOrgaoPesquisa%5B%5D=4&"), "{corpo}");
}

#[test]
fn booleano_antes_de_cada_atributo_preserva_regras_e_corpo() {
    let esperado = derivar(&observado()).unwrap();
    let html = crate::html_atributos::com_booleano_antes_dos_atributos(&observado());
    let obtido = derivar(&html).unwrap();
    assert_eq!(obtido.corpo(), esperado.corpo());
    assert_eq!(obtido.url(), esperado.url());
    let opcoes =
        "<option selected value='4'>A</option><option hidden selected value='5'>B</option>";
    let corpo = derivar(&pagina(&interior(&select_de_orgao(opcoes)))).unwrap();
    let corpo = String::from_utf8(corpo.corpo().to_vec()).unwrap();
    assert!(
        corpo.contains("&selOrgaoPesquisa%5B%5D=4&selOrgaoPesquisa%5B%5D=5&"),
        "{corpo}"
    );
}

#[test]
fn seletor_de_orgao_bruto_com_multiple_disabled_nao_e_publicado() {
    let bruto = "<select multiple disabled id=\"selOrgaoPesquisa\" name=\"selOrgaoPesquisa[]\">\
                 <option value=\"4\" selected=\"selected\">A</option></select>";
    assert_eq!(
        derivar(&pagina(&interior(bruto))).unwrap_err(),
        ErroPesquisaProtocolo::Submissao(
            ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario {
                campo: "selOrgaoPesquisa[]".to_owned()
            }
        )
    );
}

#[test]
fn seletor_de_orgao_desabilitado_nao_e_publicado_como_no_navegador() {
    let html = observado().replace(" multiple='multiple'", " multiple='multiple' disabled");
    assert_eq!(
        derivar(&html).unwrap_err(),
        ErroPesquisaProtocolo::Submissao(
            ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario {
                campo: "selOrgaoPesquisa[]".to_owned()
            }
        )
    );
}

#[test]
fn inicio_da_paginacao_servido_diferente_de_zero_e_recusado() {
    let html = observado().replace("name='hdnInicio' value='0'", "name='hdnInicio' value='20'");
    assert_eq!(regra(&html), R::InicioDaPaginacaoDiferenteDeZeroNaTela);
}

#[test]
fn recusas_do_primitivo_continuam_tipadas_na_operacao() {
    let sem_oculto = observado().replace("name='hdnInicio'", "name='outro'");
    assert_eq!(
        derivar(&sem_oculto).unwrap_err(),
        ErroPesquisaProtocolo::Submissao(ErroSubmissaoFormulario::CampoOcultoAusenteNoFormulario {
            campo: "hdnInicio".to_owned()
        })
    );
    let url = url_servida();
    let html = observado();
    assert_eq!(
        derivar_pesquisa_de_protocolo(
            &instalacao(),
            &tela_iso(&url, &html),
            "OUTRA UNIDADE",
            "0001",
        )
        .unwrap_err(),
        ErroPesquisaProtocolo::Submissao(
            ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada
        )
    );
    let sem_form = html.replace("frmPesquisaProtocolo", "frmOutro");
    assert_eq!(
        derivar(&sem_form).unwrap_err(),
        ErroPesquisaProtocolo::Submissao(ErroSubmissaoFormulario::FormularioAusenteNaTela)
    );
}

#[test]
fn subcodigos_das_regras_sao_distintos() {
    let todos = [
        R::OpcaoDeProcessosAusenteNaTela,
        R::OpcoesDeProcessosRepetidasNaTela,
        R::OpcaoDeProcessosSemValorNaTela,
        R::OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm,
        R::AssinanteOcultoPreenchidoNaPesquisaEmProcessos,
        R::SeletorDeOrgaoDePesquisaAusenteNaTela,
        R::SeletoresDeOrgaoDePesquisaRepetidosNaTela,
        R::SeletorDeOrgaoDePesquisaSemMultiplaEscolha,
        R::OrgaoDePesquisaSemOpcaoSelecionadaNaTela,
        R::OpcaoDeOrgaoSelecionadaSemValorNaTela,
        R::InicioDaPaginacaoDiferenteDeZeroNaTela,
        R::EntidadeHtmlDesconhecidaNaRegra,
    ];
    let unicos: std::collections::HashSet<_> = todos.iter().map(|e| e.subcodigo()).collect();
    assert_eq!(unicos.len(), todos.len());
}
