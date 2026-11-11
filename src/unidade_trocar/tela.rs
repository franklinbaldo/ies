//! O link da troca no Controle (`lnkInfraUnidade`), a tela Trocar Unidade
//! (`frmInfraSelecaoUnidade`, lista de unidades com permissão,
//! `selecionarUnidade()`), a escolha da sigla e o POST derivado, na forma
//! observada nas cassettes da `UnB` 4.0.9, UFRJ 4.1.5 e GO 5.0.3. Nenhum JS é
//! executado: as formas são conferidas como texto e atributos.
use super::ErroTroca;
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::decodificar_atributo;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio,
};
use crate::html_atributos::{
    atributo_presente, nos_com_id, nos_por_elemento_e_classe, valor_do_atributo,
};
use crate::instalacao::Instalacao;

#[path = "tela/consulta.rs"]
mod consulta;
pub(crate) use consulta::{ConsultaForaDaForma, conferir_consulta};

/// O oculto que `selecionarUnidade()` cria antes do `submit()`.
pub(crate) const CAMPO_CRIADO: &str = "selInfraUnidades";

/// O formulário da tela, declarado pela operação (sem catálogo central).
pub(crate) const TROCAR_UNIDADE: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::InfraTrocarUnidade,
    formulario: "frmInfraSelecaoUnidade",
    envio: EnvioDoFormulario::SubmitPorScriptComCampoCriado(CAMPO_CRIADO),
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        "hdnInfraItens",
        "hdnInfraItensHash",
        "hdnInfraItensSelecionados",
    ],
    // `infraSelecionarItens(this,'Infra')` põe o id clicado aqui.
    campos_ocultos_reescritos: &["hdnInfraItensSelecionados"],
    campos_usuario: &[
        "txtInfraSiglaUnidade",
        "txtInfraDescricaoUnidade",
        "selInfraOrgaoUnidade",
        "chkInfraItem",
    ],
    // A resposta observada é o 302; o efeito só se afirma pelo Controle novo.
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::UnidadeAtual,
    },
};

/// Uma linha da lista de unidades com permissão.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnidadeOferecida {
    pub(crate) sigla: String,
    pub(crate) descricao: String,
    pub(crate) id_unidade: String,
}

/// A tela lida: as unidades na ordem servida e o id do rádio marcado.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDeTroca {
    pub(crate) unidades: Vec<UnidadeOferecida>,
    pub(crate) marcada: UnidadeOferecida,
    /// Valores servidos dos filtros (`txtInfraSiglaUnidade`,
    /// `txtInfraDescricaoUnidade`, `selInfraOrgaoUnidade`), na ordem.
    pub(crate) filtros: [(&'static str, String); 3],
}

/// URL absoluta do link de troca publicado por `a#lnkInfraUnidade` (todos
/// iguais) na tela servida em `url_da_tela`.
pub(crate) fn link_da_troca(
    instalacao: &Instalacao,
    url_da_tela: &str,
    html: &str,
) -> Result<String, super::ErroTroca> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroTroca::LinkDeTrocaDeUnidadeAusente)?;
    let parser = dom.parser();
    let nos =
        nos_com_id(&dom, "lnkInfraUnidade").map_err(ErroTroca::AtributoDoLinkDeTrocaIlegivel)?;
    let mut onclicks = Vec::new();
    for no in nos {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if let Some(bruto) =
            valor_do_atributo(tag, "onclick").map_err(ErroTroca::AtributoDoLinkDeTrocaIlegivel)?
        {
            onclicks.push(
                decodificar_atributo(&bruto)
                    .ok_or(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada)?,
            );
        }
    }
    let Some(primeiro) = onclicks.first() else {
        return Err(ErroTroca::LinkDeTrocaDeUnidadeAusente);
    };
    if onclicks.iter().any(|o| o != primeiro) {
        return Err(super::ErroTroca::LinksDeTrocaDivergentes);
    }
    let relativo = primeiro
        .trim()
        .strip_prefix("window.location.href='")
        .and_then(|r| r.strip_suffix("';"))
        .ok_or(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada)?;
    let consulta = relativo
        .strip_prefix("controlador.php?")
        .ok_or(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada)?;
    if !relativo.bytes().all(|b| (33..=126).contains(&b)) || relativo.contains(['#', '\\', '\'']) {
        return Err(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada);
    }
    conferir_consulta(consulta, AcaoDoSei::InfraTrocarUnidade, |h| h.len() == 64)
        .map_err(|ConsultaForaDaForma| ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada)?;
    let base = url_da_tela.split('?').next().unwrap_or_default();
    let (pasta, _) = base
        .rsplit_once('/')
        .ok_or(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada)?;
    let url = format!("{pasta}/{relativo}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada);
    }
    Ok(url)
}

fn texto(no: &tl::Node<'_>, parser: &tl::Parser<'_>) -> String {
    let bruto = no.inner_text(parser).replace("&nbsp;", " ");
    html_escape::decode_html_entities(&bruto)
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, super::ErroTroca> {
    match valor_do_atributo(tag, nome) {
        Ok(None) => Ok(None),
        Ok(Some(bruto)) => decodificar_atributo(&bruto)
            .map(Some)
            .ok_or(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada),
        Err(_) => Err(ErroTroca::LinhaDeUnidadeForaDaFormaObservada),
    }
}

fn tags<'p>(
    tag: &tl::HTMLTag<'_>,
    parser: &'p tl::Parser<'p>,
    seletor: &str,
) -> Vec<&'p tl::HTMLTag<'p>> {
    tag.query_selector(parser, seletor)
        .into_iter()
        .flatten()
        .filter_map(|h| h.get(parser))
        .filter_map(tl::Node::as_tag)
        .collect()
}

const LEGENDA: &str = "Lista de Unidades com Permissão (";

/// O total da legenda `Lista de Unidades com Permissão (N registros):`.
fn total_da_legenda(legenda: &str) -> Option<usize> {
    let resto = legenda.strip_prefix(LEGENDA)?;
    let (numero, fim) = resto.split_once(' ')?;
    (fim == "registros):" || fim == "registro):").then(|| numero.parse().ok())?
}

/// A tabela da lista: exatamente uma `table.infraTable` com a legenda.
fn tabela_da_lista<'d>(
    dom: &'d tl::VDom<'d>,
) -> Result<(&'d tl::HTMLTag<'d>, usize), super::ErroTroca> {
    let parser = dom.parser();
    let mut achada = None;
    for h in nos_por_elemento_e_classe(dom, None, "table", "infraTable")
        .map_err(ErroTroca::AtributoDaTelaDeTrocaIlegivel)?
    {
        let Some(tabela) = h.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let legendas: Vec<String> = tabela
            .query_selector(parser, "caption")
            .into_iter()
            .flatten()
            .filter_map(|c| c.get(parser))
            .map(|c| texto(c, parser))
            .collect();
        if !legendas.iter().any(|l| l.starts_with(LEGENDA)) {
            continue;
        }
        let (None, [legenda]) = (&achada, legendas.as_slice()) else {
            return Err(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida);
        };
        let total = total_da_legenda(legenda).ok_or(super::ErroTroca::ListaDeUnidadesIncompleta)?;
        achada = Some((tabela, total));
    }
    achada.ok_or(super::ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)
}

/// Uma linha de dados: o rádio e as células Sigla e Descrição.
fn ler_linha(
    tr: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<Option<(UnidadeOferecida, bool)>, super::ErroTroca> {
    let celulas: Vec<_> = tr
        .query_selector(parser, "td")
        .into_iter()
        .flatten()
        .filter_map(|h| h.get(parser))
        .collect();
    if celulas.is_empty() {
        return Ok(None);
    }
    let mut radios = Vec::new();
    for input in tags(tr, parser, "input") {
        if atributo(input, "name")?.as_deref() == Some("chkInfraItem") {
            radios.push(input);
        }
    }
    let [radio] = radios.as_slice() else {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    };
    let id =
        atributo(radio, "value")?.ok_or(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada)?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    }
    let esperado = format!("infraSelecionarItens(this,'Infra'); selecionarUnidade({id})");
    if atributo(radio, "type")?.as_deref() != Some("radio")
        || atributo(radio, "onclick")?.as_deref().map(str::trim) != Some(esperado.as_str())
    {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    }
    let [_, sigla, descricao, ..] = celulas.as_slice() else {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    };
    let sigla = texto(sigla, parser);
    if sigla.is_empty() || atributo(radio, "title")?.as_deref() != Some(sigla.as_str()) {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    }
    let marcada =
        atributo_presente(radio, "checked").map_err(ErroTroca::AtributoDaLinhaDeUnidadeIlegivel)?;
    let descricao = texto(descricao, parser);
    Ok(Some((
        UnidadeOferecida {
            sigla,
            descricao,
            id_unidade: id,
        },
        marcada,
    )))
}

/// O valor servido de um filtro: `value` do `input`, ou a opção que o
/// navegador seleciona (a última com `selected`; sem nenhuma, a primeira).
fn filtro(
    dom: &tl::VDom<'_>,
    nome: &'static str,
) -> Result<(&'static str, String), super::ErroTroca> {
    let parser = dom.parser();
    let nos = nos_com_id(dom, nome).map_err(ErroTroca::AtributoDaTelaDeTrocaIlegivel)?;
    let [no] = nos.as_slice() else {
        return Err(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida);
    };
    let tag = no
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)?;
    let valor = |t: &tl::HTMLTag<'_>| -> Result<String, super::ErroTroca> {
        Ok(atributo(t, "value")
            .map_err(|_| super::ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)?
            .unwrap_or_default())
    };
    if tag.name() != "select" {
        return Ok((nome, valor(tag)?));
    }
    let opcoes = tags(tag, parser, "option");
    let mut escolhida = opcoes.first().copied();
    for opcao in &opcoes {
        if atributo_presente(opcao, "selected").map_err(ErroTroca::AtributoDaTelaDeTrocaIlegivel)? {
            escolhida = Some(opcao);
        }
    }
    let opcao = escolhida.ok_or(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)?;
    Ok((nome, valor(opcao)?))
}

/// O script da tela define `selecionarUnidade`, cria `selInfraUnidades` e
/// envia: conferido como texto, sem executar.
fn publica_selecao_por_script(html: &str) -> bool {
    let Some((_, corpo)) = html.split_once("function selecionarUnidade(idUnidade){") else {
        return false;
    };
    let corpo = corpo.split('}').next().unwrap_or_default();
    corpo.contains("document.getElementById('frmInfraSelecaoUnidade')")
        && corpo.contains("input.name = 'selInfraUnidades';")
        && corpo.contains("input.value = idUnidade;")
        && corpo.contains("frm.appendChild(input);")
        && corpo.contains("frm.submit();")
}

/// Lê e confere a tela Trocar Unidade.
pub(crate) fn ler_tela_de_troca(html: &str) -> Result<TelaDeTroca, super::ErroTroca> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)?;
    let parser = dom.parser();
    let formularios = nos_com_id(&dom, "frmInfraSelecaoUnidade")
        .map_err(ErroTroca::AtributoDaTelaDeTrocaIlegivel)?;
    if formularios.len() != 1 {
        return Err(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida);
    }
    let (tabela, total) = tabela_da_lista(&dom)?;
    if !publica_selecao_por_script(html) {
        return Err(super::ErroTroca::SelecaoDeUnidadePorScriptNaoPublicada);
    }
    let mut unidades: Vec<UnidadeOferecida> = Vec::new();
    let mut marcadas = Vec::new();
    for tr in tags(tabela, parser, "tr") {
        let Some((unidade, marcada)) = ler_linha(tr, parser)? else {
            continue;
        };
        if unidades.iter().any(|u| u.id_unidade == unidade.id_unidade) {
            return Err(ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
        }
        if marcada {
            marcadas.push(unidade.clone());
        }
        unidades.push(unidade);
    }
    if unidades.len() != total {
        return Err(super::ErroTroca::ListaDeUnidadesIncompleta);
    }
    let [marcada] = marcadas.as_slice() else {
        return Err(super::ErroTroca::UnidadeMarcadaDiferenteDaAtiva);
    };
    let filtros = [
        filtro(&dom, "txtInfraSiglaUnidade")?,
        filtro(&dom, "txtInfraDescricaoUnidade")?,
        filtro(&dom, "selInfraOrgaoUnidade")?,
    ];
    Ok(TelaDeTroca {
        marcada: marcada.clone(),
        unidades,
        filtros,
    })
}

/// A única linha com `sigla` (byte a byte); nenhuma ou mais de uma é recusa.
pub(crate) fn escolher<'t>(
    tela: &'t TelaDeTroca,
    sigla: &str,
) -> Result<&'t UnidadeOferecida, super::ErroTroca> {
    let candidatas: Vec<_> = tela.unidades.iter().filter(|u| u.sigla == sigla).collect();
    match candidatas.as_slice() {
        [unica] => Ok(unica),
        [] => Err(super::ErroTroca::UnidadeNaoOferecida {
            oferecidas: tela.unidades.clone(),
        }),
        _ => Err(super::ErroTroca::SiglaDeUnidadeAmbigua {
            candidatas: candidatas.into_iter().cloned().collect(),
        }),
    }
}

/// Os campos que o navegador envia ao clicar no rádio de `escolhida`: os
/// filtros como servidos, o rádio, o oculto que `infraSelecionarItens`
/// reescreve e o que `selecionarUnidade()` cria.
pub(crate) fn campos_do_envio<'t>(
    tela: &'t TelaDeTroca,
    escolhida: &'t UnidadeOferecida,
) -> Vec<(&'static str, &'t str)> {
    let id = escolhida.id_unidade.as_str();
    let mut campos: Vec<(&'static str, &'t str)> =
        tela.filtros.iter().map(|(n, v)| (*n, v.as_str())).collect();
    campos.extend([
        ("chkInfraItem", id),
        ("hdnInfraItensSelecionados", id),
        (CAMPO_CRIADO, id),
    ]);
    campos
}
