//! O link da troca no Controle (`lnkInfraUnidade`), a tela Trocar Unidade
//! (`frmInfraSelecaoUnidade`, lista de unidades com permissão,
//! `selecionarUnidade()`), a escolha da sigla e o POST derivado, na forma
//! observada nas cassettes da `UnB` 4.0.9, UFRJ 4.1.5 e GO 5.0.3. Nenhum JS é
//! executado: as formas são conferidas como texto e atributos.
use super::ErroTroca;
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio,
};
use crate::instalacao::Instalacao;
use crate::legenda::{Legenda, ler_legenda};

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
    html: &crate::tela_html::HtmlServido,
) -> Result<String, super::ErroTroca> {
    let dom = crate::tela_html::arvore(html);
    let nos = dom.com_id("lnkInfraUnidade").collect::<Vec<_>>();
    let mut onclicks = Vec::new();
    for no in nos {
        onclicks.extend(no.atributo("onclick"));
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
        .map_err(ErroTroca::ConsultaDoLinkDeTrocaForaDaForma)?;
    let base = url_da_tela
        .split_once('?')
        .map_or(url_da_tela, |(antes, _)| antes);
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

fn texto(no: crate::tela_html::Elemento<'_>) -> String {
    no.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tags<'p>(
    tag: crate::tela_html::Elemento<'p>,
    seletor: &str,
) -> Vec<crate::tela_html::Elemento<'p>> {
    tag.elementos(seletor).collect()
}

const LEGENDA: &str = "Lista de Unidades com Permissão (";

/// O total da legenda `Lista de Unidades com Permissão (N registros):`, sem faixa.
fn total_da_legenda(legenda: &str) -> Result<usize, super::ErroTroca> {
    if !legenda.starts_with(LEGENDA) {
        return Err(super::ErroTroca::ListaDeUnidadesIncompleta);
    }
    let Legenda { total, faixa } = ler_legenda(legenda)?;
    if faixa.is_some() {
        return Err(super::ErroTroca::ListaDeUnidadesIncompleta);
    }
    Ok(total)
}

/// A tabela da lista: exatamente uma `table.infraTable` com a legenda.
fn tabela_da_lista(
    dom: &crate::tela_html::Documento,
) -> Result<(crate::tela_html::Elemento<'_>, usize), super::ErroTroca> {
    let mut achada = None;
    for h in dom
        .elementos("table")
        .filter(|elemento| elemento.tem_classe("infraTable"))
    {
        let legendas: Vec<String> = h.elementos("caption").map(texto).collect();
        if !legendas.iter().any(|l| l.starts_with(LEGENDA)) {
            continue;
        }
        let (None, [legenda]) = (&achada, legendas.as_slice()) else {
            return Err(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida);
        };
        let total = total_da_legenda(legenda)?;
        achada = Some((h, total));
    }
    achada.ok_or(super::ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)
}

/// Uma linha de dados: o rádio e as células Sigla e Descrição.
fn ler_linha(
    tr: crate::tela_html::Elemento<'_>,
) -> Result<Option<(UnidadeOferecida, bool)>, super::ErroTroca> {
    let celulas: Vec<_> = tr.elementos("td").collect();
    if celulas.is_empty() {
        return Ok(None);
    }
    let mut radios = Vec::new();
    for input in tags(tr, "input") {
        if input.atributo("name") == Some("chkInfraItem") {
            radios.push(input);
        }
    }
    let [radio] = radios.as_slice() else {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    };
    let id = radio
        .atributo("value")
        .map(str::to_owned)
        .ok_or(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada)?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    }
    let esperado = format!("infraSelecionarItens(this,'Infra'); selecionarUnidade({id})");
    if radio.atributo("type") != Some("radio")
        || radio.atributo("onclick").map(str::trim) != Some(esperado.as_str())
    {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    }
    let [_, sigla, descricao, ..] = celulas.as_slice() else {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    };
    let sigla = texto(*sigla);
    if sigla.is_empty() || radio.atributo("title") != Some(sigla.as_str()) {
        return Err(super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada);
    }
    let marcada = radio.presente("checked");
    let descricao = texto(*descricao);
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
    dom: &crate::tela_html::Documento,
    nome: &'static str,
) -> Result<(&'static str, String), super::ErroTroca> {
    let nos = dom.com_id(nome).collect::<Vec<_>>();
    let [no] = nos.as_slice() else {
        return Err(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida);
    };
    let tag = *no;
    let valor =
        |t: crate::tela_html::Elemento<'_>| t.atributo("value").unwrap_or_default().to_owned();
    if tag.nome() != "select" {
        return Ok((nome, valor(tag)));
    }
    let opcoes = tags(tag, "option");
    let mut escolhida = opcoes.first().copied();
    for opcao in &opcoes {
        if opcao.presente("selected") {
            escolhida = Some(*opcao);
        }
    }
    let opcao = escolhida.ok_or(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida)?;
    Ok((nome, valor(opcao)))
}

/// O script da tela define `selecionarUnidade`, cria `selInfraUnidades` e
/// envia: conferido como texto, sem executar.
fn publica_selecao_por_script(html: &str) -> bool {
    let Some((_, corpo)) = html.split_once("function selecionarUnidade(idUnidade){") else {
        return false;
    };
    let corpo = corpo.split_once('}').map_or(corpo, |(antes, _)| antes);
    corpo.contains("document.getElementById('frmInfraSelecaoUnidade')")
        && corpo.contains("input.name = 'selInfraUnidades';")
        && corpo.contains("input.value = idUnidade;")
        && corpo.contains("frm.appendChild(input);")
        && corpo.contains("frm.submit();")
}

/// Lê e confere a tela Trocar Unidade.
pub(crate) fn ler_tela_de_troca(
    html: &crate::tela_html::HtmlServido,
) -> Result<TelaDeTroca, super::ErroTroca> {
    let dom = crate::tela_html::arvore(html);
    let formularios = dom.com_id("frmInfraSelecaoUnidade").collect::<Vec<_>>();
    if formularios.len() != 1 {
        return Err(ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida);
    }
    let (tabela, total) = tabela_da_lista(&dom)?;
    if !publica_selecao_por_script(html) {
        return Err(super::ErroTroca::SelecaoDeUnidadePorScriptNaoPublicada);
    }
    let mut unidades: Vec<UnidadeOferecida> = Vec::new();
    let mut marcadas = Vec::new();
    for tr in tags(tabela, "tr") {
        let Some((unidade, marcada)) = ler_linha(tr)? else {
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
