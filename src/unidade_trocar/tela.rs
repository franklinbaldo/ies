//! O link da troca no Controle (`lnkInfraUnidade`), a tela Trocar Unidade
//! (`frmInfraSelecaoUnidade`, lista de unidades com permissão,
//! `selecionarUnidade()`), a escolha da sigla e o POST derivado, na forma
//! observada nas cassettes da `UnB` 4.0.9, UFRJ 4.1.5 e GO 5.0.3. Nenhum JS é
//! executado: as formas são conferidas como texto e atributos.
use super::ErroTroca as E;
use crate::erro_html::ErroHtml;
use crate::formulario_web::decodificar_atributo;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio,
};
use crate::html_atributos::{
    atributo_presente, nos_com_id, nos_por_elemento_e_classe, valor_do_atributo,
};
use crate::instalacao::Instalacao;

pub(crate) const ACAO_DE_TROCAR: &str = "infra_trocar_unidade";
/// O oculto que `selecionarUnidade()` cria antes do `submit()`.
pub(crate) const CAMPO_CRIADO: &str = "selInfraUnidades";

/// O formulário da tela, declarado pela operação (sem catálogo central).
pub(crate) const TROCAR_UNIDADE: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: ACAO_DE_TROCAR,
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
) -> Result<String, E> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        ErroHtml::ausente("link_de_troca_de_unidade_ausente")
            .com_mensagem("o Controle não publica o link de troca de unidade")
    })?;
    let parser = dom.parser();
    let nos = nos_com_id(&dom, "lnkInfraUnidade").map_err(|e| {
        ErroHtml::atributo("link_de_troca_de_unidade_fora_da_forma_observada", e)
            .com_mensagem("o link de troca de unidade não tem a forma observada")
    })?;
    let mut onclicks = Vec::new();
    for no in nos {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if let Some(bruto) = valor_do_atributo(tag, "onclick").map_err(|e| {
            ErroHtml::atributo("link_de_troca_de_unidade_fora_da_forma_observada", e)
                .com_mensagem("o link de troca de unidade não tem a forma observada")
        })? {
            onclicks.push(
                decodificar_atributo(&bruto).ok_or(
                    ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
                        .com_mensagem("o link de troca de unidade não tem a forma observada"),
                )?,
            );
        }
    }
    let Some(primeiro) = onclicks.first() else {
        return Err(ErroHtml::ausente("link_de_troca_de_unidade_ausente")
            .com_mensagem("o Controle não publica o link de troca de unidade")
            .into());
    };
    if onclicks.iter().any(|o| o != primeiro) {
        return Err(E::LinksDeTrocaDivergentes);
    }
    let relativo = primeiro
        .trim()
        .strip_prefix("window.location.href='")
        .and_then(|r| r.strip_suffix("';"))
        .ok_or(
            ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
                .com_mensagem("o link de troca de unidade não tem a forma observada"),
        )?;
    let consulta = relativo.strip_prefix("controlador.php?").ok_or(
        ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
            .com_mensagem("o link de troca de unidade não tem a forma observada"),
    )?;
    if !relativo.bytes().all(|b| (33..=126).contains(&b)) || relativo.contains(['#', '\\', '\'']) {
        return Err(
            ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
                .com_mensagem("o link de troca de unidade não tem a forma observada")
                .into(),
        );
    }
    conferir_consulta(consulta, ACAO_DE_TROCAR, |h| h.len() == 64).map_err(
        |ConsultaForaDaForma| {
            ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
                .com_mensagem("o link de troca de unidade não tem a forma observada")
        },
    )?;
    let base = url_da_tela.split('?').next().unwrap_or_default();
    let (pasta, _) = base.rsplit_once('/').ok_or(
        ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
            .com_mensagem("o link de troca de unidade não tem a forma observada"),
    )?;
    let url = format!("{pasta}/{relativo}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or(
            ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
                .com_mensagem("o link de troca de unidade não tem a forma observada"),
        )?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(
            ErroHtml::valor("link_de_troca_de_unidade_fora_da_forma_observada")
                .com_mensagem("o link de troca de unidade não tem a forma observada")
                .into(),
        );
    }
    Ok(url)
}

/// Confere a consulta de um link assinado: chaves canônicas, valores sem
/// `;`, `acao` única igual a `acao`, um `infra_hash` hexadecimal aceito por
/// `comprimento`, sem `acao_ajax`. Devolve o `infra_unidade_atual` (único),
/// se houver.
pub(crate) fn conferir_consulta(
    consulta: &str,
    acao: &str,
    comprimento: impl Fn(&str) -> bool,
) -> Result<Option<String>, ConsultaForaDaForma> {
    let mut unidades = conferir(consulta, acao, comprimento).ok_or(ConsultaForaDaForma)?;
    if unidades.len() > 1 {
        return Err(ConsultaForaDaForma);
    }
    Ok(unidades.pop())
}

/// A consulta não tem a forma conferida por [`conferir_consulta`].
#[derive(Debug)]
pub(crate) struct ConsultaForaDaForma;

fn conferir(consulta: &str, acao: &str, comprimento: impl Fn(&str) -> bool) -> Option<Vec<String>> {
    use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
    let (mut acoes, mut hashes) = (0, 0);
    let mut unidades = Vec::new();
    for par in consulta.split('&') {
        let (nome, valor) = par.split_once('=')?;
        conferir_chave_de_consulta(nome).ok()?;
        conferir_valor_de_consulta(valor).ok()?;
        match nome {
            "acao" if valor == acao => acoes += 1,
            "infra_hash" if comprimento(valor) && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "infra_unidade_atual" => unidades.push(valor.to_owned()),
            "acao" | "infra_hash" | "acao_ajax" => return None,
            _ => {}
        }
    }
    (acoes == 1 && hashes == 1).then_some(unidades)
}

fn texto(no: &tl::Node<'_>, parser: &tl::Parser<'_>) -> String {
    let bruto = no.inner_text(parser).replace("&nbsp;", " ");
    html_escape::decode_html_entities(&bruto)
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    match valor_do_atributo(tag, nome) {
        Ok(None) => Ok(None),
        Ok(Some(bruto)) => decodificar_atributo(&bruto).map(Some).ok_or(E::Html(
            ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
                .com_mensagem("uma linha da lista de unidades não tem a forma observada"),
        )),
        Err(_) => Err(ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
            .com_mensagem("uma linha da lista de unidades não tem a forma observada")
            .into()),
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
fn tabela_da_lista<'d>(dom: &'d tl::VDom<'d>) -> Result<(&'d tl::HTMLTag<'d>, usize), E> {
    let parser = dom.parser();
    let mut achada = None;
    for h in nos_por_elemento_e_classe(dom, None, "table", "infraTable").map_err(|e| {
        ErroHtml::atributo("tela_de_troca_de_unidade_nao_reconhecida", e)
            .com_mensagem("a tela de troca de unidade não foi reconhecida")
    })? {
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
        if achada.is_some() || legendas.len() != 1 {
            return Err(ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
                .com_mensagem("a tela de troca de unidade não foi reconhecida")
                .into());
        }
        let total = total_da_legenda(&legendas[0]).ok_or(E::ListaDeUnidadesIncompleta)?;
        achada = Some((tabela, total));
    }
    achada.ok_or(E::Html(
        ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
            .com_mensagem("a tela de troca de unidade não foi reconhecida"),
    ))
}

/// Uma linha de dados: o rádio e as células Sigla e Descrição.
fn ler_linha(
    tr: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<Option<(UnidadeOferecida, bool)>, E> {
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
        return Err(E::Html(
            ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
                .com_mensagem("uma linha da lista de unidades não tem a forma observada"),
        ));
    };
    let id = atributo(radio, "value")?.ok_or(E::Html(
        ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
            .com_mensagem("uma linha da lista de unidades não tem a forma observada"),
    ))?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(E::Html(
            ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
                .com_mensagem("uma linha da lista de unidades não tem a forma observada"),
        ));
    }
    let esperado = format!("infraSelecionarItens(this,'Infra'); selecionarUnidade({id})");
    if atributo(radio, "type")?.as_deref() != Some("radio")
        || atributo(radio, "onclick")?.as_deref().map(str::trim) != Some(esperado.as_str())
        || celulas.len() < 3
    {
        return Err(E::Html(
            ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
                .com_mensagem("uma linha da lista de unidades não tem a forma observada"),
        ));
    }
    let sigla = texto(celulas[1], parser);
    if sigla.is_empty() || atributo(radio, "title")?.as_deref() != Some(sigla.as_str()) {
        return Err(E::Html(
            ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
                .com_mensagem("uma linha da lista de unidades não tem a forma observada"),
        ));
    }
    let marcada = atributo_presente(radio, "checked").map_err(|e| {
        ErroHtml::atributo("linha_de_unidade_fora_da_forma_observada", e)
            .com_mensagem("uma linha da lista de unidades não tem a forma observada")
    })?;
    let descricao = texto(celulas[2], parser);
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
fn filtro(dom: &tl::VDom<'_>, nome: &'static str) -> Result<(&'static str, String), E> {
    let parser = dom.parser();
    let nos = nos_com_id(dom, nome).map_err(|e| {
        ErroHtml::atributo("tela_de_troca_de_unidade_nao_reconhecida", e)
            .com_mensagem("a tela de troca de unidade não foi reconhecida")
    })?;
    let [no] = nos.as_slice() else {
        return Err(ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
            .com_mensagem("a tela de troca de unidade não foi reconhecida")
            .into());
    };
    let tag = no.get(parser).and_then(tl::Node::as_tag).ok_or(
        ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
            .com_mensagem("a tela de troca de unidade não foi reconhecida"),
    )?;
    let valor = |t: &tl::HTMLTag<'_>| -> Result<String, E> {
        Ok(atributo(t, "value")
            .map_err(|_| {
                E::Html(
                    ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
                        .com_mensagem("a tela de troca de unidade não foi reconhecida"),
                )
            })?
            .unwrap_or_default())
    };
    if tag.name() != "select" {
        return Ok((nome, valor(tag)?));
    }
    let opcoes = tags(tag, parser, "option");
    let mut escolhida = opcoes.first().copied();
    for opcao in &opcoes {
        if atributo_presente(opcao, "selected").map_err(|e| {
            ErroHtml::atributo("tela_de_troca_de_unidade_nao_reconhecida", e)
                .com_mensagem("a tela de troca de unidade não foi reconhecida")
        })? {
            escolhida = Some(opcao);
        }
    }
    let opcao = escolhida.ok_or(
        ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
            .com_mensagem("a tela de troca de unidade não foi reconhecida"),
    )?;
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
pub(crate) fn ler_tela_de_troca(html: &str) -> Result<TelaDeTroca, E> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
            .com_mensagem("a tela de troca de unidade não foi reconhecida")
    })?;
    let parser = dom.parser();
    let formularios = nos_com_id(&dom, "frmInfraSelecaoUnidade").map_err(|e| {
        ErroHtml::atributo("tela_de_troca_de_unidade_nao_reconhecida", e)
            .com_mensagem("a tela de troca de unidade não foi reconhecida")
    })?;
    if formularios.len() != 1 {
        return Err(ErroHtml::valor("tela_de_troca_de_unidade_nao_reconhecida")
            .com_mensagem("a tela de troca de unidade não foi reconhecida")
            .into());
    }
    let (tabela, total) = tabela_da_lista(&dom)?;
    if !publica_selecao_por_script(html) {
        return Err(E::SelecaoDeUnidadePorScriptNaoPublicada);
    }
    let mut unidades: Vec<UnidadeOferecida> = Vec::new();
    let mut marcadas = Vec::new();
    for tr in tags(tabela, parser, "tr") {
        let Some((unidade, marcada)) = ler_linha(tr, parser)? else {
            continue;
        };
        if unidades.iter().any(|u| u.id_unidade == unidade.id_unidade) {
            return Err(ErroHtml::valor("linha_de_unidade_fora_da_forma_observada")
                .com_mensagem("uma linha da lista de unidades não tem a forma observada")
                .into());
        }
        if marcada {
            marcadas.push(unidade.clone());
        }
        unidades.push(unidade);
    }
    if unidades.len() != total {
        return Err(E::ListaDeUnidadesIncompleta);
    }
    let [marcada] = marcadas.as_slice() else {
        return Err(E::UnidadeMarcadaDiferenteDaAtiva);
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
pub(crate) fn escolher<'t>(tela: &'t TelaDeTroca, sigla: &str) -> Result<&'t UnidadeOferecida, E> {
    let candidatas: Vec<_> = tela.unidades.iter().filter(|u| u.sigla == sigla).collect();
    match candidatas.as_slice() {
        [unica] => Ok(unica),
        [] => Err(E::UnidadeNaoOferecida {
            oferecidas: tela.unidades.clone(),
        }),
        _ => Err(E::SiglaDeUnidadeAmbigua {
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
