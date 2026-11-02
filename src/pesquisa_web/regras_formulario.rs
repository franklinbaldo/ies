//! Regras do `onsubmit` de `frmPesquisaProtocolo`, escritas à mão a partir do
//! JavaScript servido em `unb`, `pen1`, `ufrj` e `go` (GET de
//! `protocolo_pesquisar` das cassettes `processo-busca-*`). Nada de JS é
//! executado ou interpretado aqui. Evidência: docs/decisoes/processo-buscar.md.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{ErroAtributoHtml, atributo_presente, valor_do_atributo};
use ErroRegraDoFormularioDePesquisa as E;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRegraDoFormularioDePesquisa {
    OpcaoDeProcessosAusenteNaTela,
    OpcoesDeProcessosRepetidasNaTela,
    OpcaoDeProcessosSemValorNaTela,
    OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm,
    AssinanteOcultoPreenchidoNaPesquisaEmProcessos,
    SeletorDeOrgaoDePesquisaAusenteNaTela,
    SeletoresDeOrgaoDePesquisaRepetidosNaTela,
    SeletorDeOrgaoDePesquisaSemMultiplaEscolha,
    OrgaoDePesquisaSemOpcaoSelecionadaNaTela,
    OpcaoDeOrgaoSelecionadaSemValorNaTela,
    InicioDaPaginacaoDiferenteDeZeroNaTela,
    EntidadeHtmlDesconhecidaNaRegra,
    /// Tag de abertura que o tokenizador de atributos recusou.
    AtributoHtml(ErroAtributoHtml),
}

impl ErroRegraDoFormularioDePesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::OpcaoDeProcessosAusenteNaTela => "opcao_de_processos_ausente_na_tela",
            Self::OpcoesDeProcessosRepetidasNaTela => "opcoes_de_processos_repetidas_na_tela",
            Self::OpcaoDeProcessosSemValorNaTela => "opcao_de_processos_sem_valor_na_tela",
            Self::OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm => {
                "opcao_de_processos_com_nome_diferente_de_rdo_pesquisar_em"
            }
            Self::AssinanteOcultoPreenchidoNaPesquisaEmProcessos => {
                "assinante_oculto_preenchido_na_pesquisa_em_processos"
            }
            Self::SeletorDeOrgaoDePesquisaAusenteNaTela => {
                "seletor_de_orgao_de_pesquisa_ausente_na_tela"
            }
            Self::SeletoresDeOrgaoDePesquisaRepetidosNaTela => {
                "seletores_de_orgao_de_pesquisa_repetidos_na_tela"
            }
            Self::SeletorDeOrgaoDePesquisaSemMultiplaEscolha => {
                "seletor_de_orgao_de_pesquisa_sem_multipla_escolha"
            }
            Self::OrgaoDePesquisaSemOpcaoSelecionadaNaTela => {
                "orgao_de_pesquisa_sem_opcao_selecionada_na_tela"
            }
            Self::OpcaoDeOrgaoSelecionadaSemValorNaTela => {
                "opcao_de_orgao_selecionada_sem_valor_na_tela"
            }
            Self::InicioDaPaginacaoDiferenteDeZeroNaTela => {
                "inicio_da_paginacao_diferente_de_zero_na_tela"
            }
            Self::EntidadeHtmlDesconhecidaNaRegra => "entidade_html_desconhecida_na_regra",
            Self::AtributoHtml(falha) => falha.subcodigo(),
        }
    }
}

/// Valores que o `onsubmit` exige e que a operação lê da tela servida.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegrasDaTela {
    pub(crate) opcao_de_processos: String,
    pub(crate) orgaos_selecionados: Vec<String>,
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    valor_do_atributo(tag, nome)
        .map_err(E::AtributoHtml)?
        .map(|bruto| decodificar_atributo(&bruto).ok_or(E::EntidadeHtmlDesconhecidaNaRegra))
        .transpose()
}

fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, E> {
    atributo_presente(tag, nome).map_err(E::AtributoHtml)
}

fn eh(tag: &tl::HTMLTag<'_>, elemento: &str) -> bool {
    tag.name().as_utf8_str().eq_ignore_ascii_case(elemento)
}

/// `onSubmitForm()` exige rádio marcado; `D` exigiria checkboxes que a
/// derivação não envia, então a operação usa `#optProcessos` como servido.
fn opcao_de_processos(tags: &[&tl::HTMLTag<'_>]) -> Result<String, E> {
    let mut achadas = Vec::new();
    for tag in tags.iter().filter(|t| eh(t, "input")) {
        if atributo(tag, "id")?.as_deref() == Some("optProcessos") {
            achadas.push(*tag);
        }
    }
    let radio = match achadas.as_slice() {
        [] => return Err(E::OpcaoDeProcessosAusenteNaTela),
        [radio] => *radio,
        _ => return Err(E::OpcoesDeProcessosRepetidasNaTela),
    };
    if atributo(radio, "name")?.as_deref() != Some("rdoPesquisarEm") {
        return Err(E::OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm);
    }
    atributo(radio, "value")?.ok_or(E::OpcaoDeProcessosSemValorNaTela)
}

/// Com `P`, `tratarCamposDocumento()` esvazia `hdnIdAssinante`; `navegar()`
/// só reescreve `hdnInicio` na paginação. Ocultos saem como servidos.
fn conferir_ocultos(tags: &[&tl::HTMLTag<'_>]) -> Result<(), E> {
    for tag in tags.iter().filter(|t| eh(t, "input")) {
        let valor = || atributo(tag, "value").map(Option::unwrap_or_default);
        match atributo(tag, "name")?.as_deref() {
            Some("hdnIdAssinante") if !valor()?.is_empty() => {
                return Err(E::AssinanteOcultoPreenchidoNaPesquisaEmProcessos);
            }
            Some("hdnInicio") if valor()? != "0" => {
                return Err(E::InicioDaPaginacaoDiferenteDeZeroNaTela);
            }
            _ => {}
        }
    }
    Ok(())
}

/// `onSubmitForm()` exige órgão em `selOrgaoPesquisa[]`: as opções com
/// `selected` servidas, na ordem do documento; nenhuma é inventada.
/// O `select` é achado pelo `name` (o nome serializado no POST), não pelo
/// `id`: a regra continua mais precisa mesmo sem o defeito do tl 0.7.8.
fn orgaos_selecionados(
    tags: &[&tl::HTMLTag<'_>],
    parser: &tl::Parser<'_>,
) -> Result<Vec<String>, E> {
    let mut achados = Vec::new();
    for tag in tags.iter().filter(|t| eh(t, "select")) {
        if atributo(tag, "name")?.as_deref() == Some("selOrgaoPesquisa[]") {
            achados.push(*tag);
        }
    }
    let select = match achados.as_slice() {
        [] => return Err(E::SeletorDeOrgaoDePesquisaAusenteNaTela),
        [select] => *select,
        _ => return Err(E::SeletoresDeOrgaoDePesquisaRepetidosNaTela),
    };
    if !presente(select, "multiple")? {
        return Err(E::SeletorDeOrgaoDePesquisaSemMultiplaEscolha);
    }
    let mut valores = Vec::new();
    for opcao in select
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if eh(opcao, "option") && presente(opcao, "selected")? {
            valores
                .push(atributo(opcao, "value")?.ok_or(E::OpcaoDeOrgaoSelecionadaSemValorNaTela)?);
        }
    }
    if valores.is_empty() {
        return Err(E::OrgaoDePesquisaSemOpcaoSelecionadaNaTela);
    }
    Ok(valores)
}

/// Lê, do `<form>` já localizado, os valores que o `onsubmit` exige.
pub(crate) fn ler_regras_da_tela(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<RegrasDaTela, ErroRegraDoFormularioDePesquisa> {
    let tags: Vec<_> = form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .collect();
    let opcao_de_processos = opcao_de_processos(&tags)?;
    conferir_ocultos(&tags)?;
    let orgaos_selecionados = orgaos_selecionados(&tags, parser)?;
    Ok(RegrasDaTela {
        opcao_de_processos,
        orgaos_selecionados,
    })
}
