//! A tela "Enviar Processo" (contracts/cli/processo-enviar.md): conferência
//! do formulário servido, a consulta do autocomplete que o script declara e
//! os pares do POST.
//!
//! Evidência (cassettes públicas do Todos, 37 telas nas sete instalações):
//! `form#frmAtividadeListar` `method="post"`; ocultos `hdnInfraTipoPagina`,
//! `hdnIdProtocolos` (o id), `hdnIdAtividades`, `hdnIdUnidade` e
//! `hdnUnidades` (vazios); `sbmEnviar`; `selOrgao` com `null` e "Todos"
//! (`""`) `selected` (vale a última, como no navegador); as três caixas e
//! os prazos desmarcados. O script declara o autocomplete de `hdnIdUnidade`
//! numa linha e o corpo da consulta no `prepararExecucao`, em duas formas
//! (4.0 sem e 4.1/5.0 com `&unidade_atual=0`).
use super::Destino;
use crate::arvore_processo::ACAO_DE_ENVIAR;
use crate::erro_html::ErroHtml;
use crate::formulario_web::decodificar_atributo;
use crate::formulario_web::{
    ConferenciaDaUnidade, ConsultaAjax, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, consulta_do_controlador, localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};

pub(crate) const FORMULARIO_DE_ENVIO: &str = "frmAtividadeListar";
const BOTAO_ENVIAR: &str = "sbmEnviar";
const OCULTOS: [&str; 5] = [
    "hdnInfraTipoPagina",
    "hdnIdProtocolos",
    "hdnIdAtividades",
    "hdnIdUnidade",
    "hdnUnidades",
];
const ORGAO: &str = "selOrgao";
const TEXTO_DA_UNIDADE: &str = "txtUnidade";
const MANTER_ABERTO: &str = "chkSinManterAberto";
const REMOVER_ANOTACOES: &str = "chkSinRemoverAnotacoes";
const UNIDADES: &str = "hdnUnidades";
const SELECOES: [&str; 3] = ["selProcedimentos", ORGAO, "selUnidades"];
const CAIXAS: [&str; 6] = [
    MANTER_ABERTO,
    REMOVER_ANOTACOES,
    "chkSinEnviarEmailNotificacao",
    "chkSinDiasUteis",
    "chkSinDiasUteisRetornoProgramado",
    "chkSinDiasUteisReaberturaProgramada",
];
const RADIOS: [&str; 3] = [
    "rdoPrazo",
    "rdoPrazoRetornoProgramado",
    "rdoPrazoReaberturaProgramada",
];
const DECLARACAO: &str = "objAutoCompletarUnidade = new infraAjaxAutoCompletar(";
const DECLARACAO_ATE_A_URL: &str =
    "objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','";
const PREPARAR: &str = "return 'palavras_pesquisa=";
const PREPARAR_40: &str = "return 'palavras_pesquisa='+document.getElementById('txtUnidade').value+'&id_orgao='+document.getElementById('selOrgao').value;";
const PREPARAR_41: &str = "return 'palavras_pesquisa='+document.getElementById('txtUnidade').value+'&id_orgao='+document.getElementById('selOrgao').value+'&unidade_atual=0';";
const TEXTOS: [&str; 6] = [
    "txtPrazo",
    "txtDias",
    "txtPrazoRetornoProgramado",
    "txtDiasRetornoProgramado",
    "txtPrazoReaberturaProgramada",
    "txtDiasReaberturaProgramada",
];
const USUARIO: [&str; 10] = [
    ORGAO,
    TEXTO_DA_UNIDADE,
    MANTER_ABERTO,
    REMOVER_ANOTACOES,
    TEXTOS[0],
    TEXTOS[1],
    TEXTOS[2],
    TEXTOS[3],
    TEXTOS[4],
    TEXTOS[5],
];

/// Enviar da tela de envio: janela sem barra da unidade, aberta pelo link
/// da barra da árvore da tela do processo conferida.
pub(crate) const ENVIAR_PROCESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: ACAO_DE_ENVIAR,
    formulario: FORMULARIO_DE_ENVIO,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO_ENVIAR),
    campos_ocultos_exigidos: &OCULTOS,
    campos_ocultos_reescritos: &[UNIDADES],
    campos_usuario: &USUARIO,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::UnidadesComOProcessoAberto,
    },
};

/// A consulta que o `infraAjaxAutoCompletar` de `hdnIdUnidade` faz.
pub(crate) const AUTOCOMPLETAR_UNIDADE: ConsultaAjax = ConsultaAjax {
    acao_ajax: "unidade_auto_completar_envio_processo",
    campos: &["palavras_pesquisa", "id_orgao", "unidade_atual"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/envio-do-processo.md: o histórico depois de duas consultas e duas telas só tem a criação e o envio",
    },
};

/// O que a tela conferida publica para a consulta e o POST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDeEnvio {
    /// O valor efetivo do `selOrgao` (a última opção `selected`).
    pub(crate) orgao: String,
    /// `(value, rótulo)` de cada opção do `selOrgao`, na ordem servida
    /// (rótulo com entidades decodificadas e espaços colapsados).
    pub(crate) orgaos: Vec<(String, String)>,
    /// Os textos de prazo servidos, na ordem da lista fechada.
    pub(crate) textos: Vec<&'static str>,
    /// A URL relativa que o script declara (assinada: nunca publicada).
    pub(crate) autocompletar: String,
    /// O `prepararExecucao` acrescenta `&unidade_atual=0` (4.1 e 5.0).
    pub(crate) unidade_atual: bool,
    /// A opção foi oferecida habilitada no formulário real.
    pub(crate) remover_anotacoes_oferecida: bool,
}

/// Confere a tela servida antes de qualquer POST: lista fechada de
/// controles enviáveis, o processo, nada preenchido, o órgão efetivo e as
/// duas linhas do script.
pub(crate) fn conferir_tela_de_envio(
    html: &str,
    id: &str,
) -> Result<TelaDeEnvio, super::erro::ErroTelaDeEnvio> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("tela_de_envio_nao_interpretada"))?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, FORMULARIO_DE_ENVIO)
        .map_err(|_| ErroHtml::ausente("formulario_de_envio_ausente"))?;
    let mut preenchida = false;
    let mut remover_anotacoes_oferecida = false;
    let mut textos = Vec::new();
    let mut selecao: Option<String> = None;
    let mut orgao: Option<String> = None;
    let mut orgao_primeira: Option<String> = None;
    let mut orgaos = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if elemento == "option" {
            let valor = decodificado(tag, "value")?
                .ok_or(ErroHtml::valor("campo_nao_previsto_na_tela_de_envio"))?;
            let marcada = presente(tag, "selected")?;
            match selecao.as_deref() {
                Some(ORGAO) => {
                    orgao_primeira.get_or_insert_with(|| valor.clone());
                    orgaos.push((valor.clone(), rotulo(tag, parser)));
                    if marcada {
                        orgao = Some(valor);
                    }
                }
                Some("selUnidades") => preenchida = true,
                Some(_) => preenchida |= marcada,
                None => return Err(ErroHtml::valor("campo_nao_previsto_na_tela_de_envio").into()),
            }
            continue;
        }
        if !matches!(
            elemento.as_str(),
            "input" | "button" | "select" | "textarea"
        ) {
            continue;
        }
        selecao = None;
        if presente(tag, "disabled")? {
            continue;
        }
        let Some(nome) = decodificado(tag, "name")?.filter(|n| !n.is_empty()) else {
            continue;
        };
        let padrao = if elemento == "button" {
            "submit"
        } else {
            "text"
        };
        let tipo = decodificado(tag, "type")?
            .map_or_else(|| padrao.to_owned(), |t| t.to_ascii_lowercase());
        let marcado = presente(tag, "checked")?;
        let servido = decodificado(tag, "value")?.unwrap_or_default();
        match (elemento.as_str(), tipo.as_str(), nome.as_str()) {
            ("input", "hidden", "hdnIdProtocolos") if servido != id => {
                return Err(super::erro::ErroTelaDeEnvio::TelaDeEnvioDeOutroProcesso);
            }
            ("input", "hidden", "hdnIdUnidade" | UNIDADES)
            | ("input", "text", TEXTO_DA_UNIDADE) => {
                preenchida |= !servido.is_empty();
            }
            ("input", "hidden", n) if OCULTOS.contains(&n) => {}
            ("input" | "button", "button" | "reset", _)
            | ("input" | "button", "submit", BOTAO_ENVIAR) => {}
            ("select", _, n) if SELECOES.contains(&n) => selecao = Some(nome.clone()),
            ("input", "checkbox" | "radio", n) if CAIXAS.contains(&n) || RADIOS.contains(&n) => {
                remover_anotacoes_oferecida |= n == REMOVER_ANOTACOES && tipo == "checkbox";
                preenchida |= marcado;
            }
            ("input", "text", n) if TEXTOS.contains(&n) => {
                preenchida |= !servido.is_empty();
                if let Some(texto) = TEXTOS.iter().find(|t| **t == n) {
                    textos.push(*texto);
                }
            }
            _ => return Err(ErroHtml::valor("campo_nao_previsto_na_tela_de_envio").into()),
        }
    }
    if preenchida {
        return Err(super::erro::ErroTelaDeEnvio::TelaDeEnvioComOpcoesPreenchidas);
    }
    // Sem opção marcada, o navegador envia a primeira.
    let orgao = orgao
        .or(orgao_primeira)
        .filter(|valor| valor != "null")
        .ok_or(super::erro::ErroTelaDeEnvio::OrgaoDaTelaDeEnvioNaoSelecionado)?;
    let (autocompletar, unidade_atual) = ler_script(html)?;
    Ok(TelaDeEnvio {
        orgao,
        orgaos,
        textos,
        autocompletar,
        unidade_atual,
        remover_anotacoes_oferecida,
    })
}

/// A URL do autocomplete de `hdnIdUnidade` e a forma do `prepararExecucao`,
/// cada uma numa linha exata e única.
fn ler_script(html: &str) -> Result<(String, bool), super::erro::ErroTelaDeEnvio> {
    let mut urls = Vec::new();
    let mut formas = Vec::new();
    for linha in html.lines().map(str::trim) {
        if linha.starts_with(DECLARACAO) {
            urls.push(
                linha
                    .strip_prefix(DECLARACAO_ATE_A_URL)
                    .and_then(|resto| resto.strip_suffix("');"))
                    .filter(|url| {
                        !url.is_empty()
                            && url.bytes().all(|b| (33..=126).contains(&b))
                            && !url.contains(['\'', '\\', '"'])
                    }),
            );
        } else if linha.starts_with(PREPARAR) {
            formas.push(match linha {
                PREPARAR_40 => Some(false),
                PREPARAR_41 => Some(true),
                _ => None,
            });
        }
    }
    let url = match urls.as_slice() {
        [Some(url)] => (*url).to_owned(),
        _ => return Err(super::erro::ErroTelaDeEnvio::AutocompletarDeUnidadeNaoDeclarado),
    };
    match formas.as_slice() {
        [Some(unidade_atual)] => Ok((url, *unidade_atual)),
        _ => Err(ErroHtml::valor("consulta_do_autocompletar_fora_da_forma_observada").into()),
    }
}

/// O texto visível da opção: entidades decodificadas, espaços colapsados.
fn rotulo(opcao: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&opcao.inner_text(parser))
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, super::erro::ErroTelaDeEnvio> {
    atributo_presente(tag, nome).map_err(|e| {
        super::erro::ErroTelaDeEnvio::Html(ErroHtml::atributo("tela_de_envio_nao_interpretada", e))
    })
}

fn decodificado(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, super::erro::ErroTelaDeEnvio> {
    valor_do_atributo(tag, nome)
        .map_err(|e| ErroHtml::atributo("tela_de_envio_nao_interpretada", e))?
        .map(|bruto| {
            decodificar_atributo(&bruto).ok_or(super::erro::ErroTelaDeEnvio::Html(ErroHtml::html(
                "tela_de_envio_nao_interpretada",
            )))
        })
        .transpose()
}

/// Os pares do `prepararExecucao`: a sigla, o órgão efetivo e, no 4.1/5.0,
/// `unidade_atual=0`.
#[cfg(test)]
pub(crate) fn campos_da_consulta<'a>(
    tela: &'a TelaDeEnvio,
    sigla: &'a str,
) -> Vec<(&'static str, &'a str)> {
    campos_da_consulta_no_orgao(tela, sigla, &tela.orgao)
}

/// Os mesmos pares com `id_orgao` = `orgao` (o da tela ou o de uma opção
/// do `selOrgao`, como o navegador mandaria com ela escolhida).
pub(crate) fn campos_da_consulta_no_orgao<'a>(
    tela: &TelaDeEnvio,
    termo: &'a str,
    orgao: &'a str,
) -> Vec<(&'static str, &'a str)> {
    let mut campos = vec![("palavras_pesquisa", termo), ("id_orgao", orgao)];
    if tela.unidade_atual {
        campos.push(("unidade_atual", "0"));
    }
    campos
}

/// Os pares do usuário e o `hdnUnidades` reescrito (a forma
/// `ID±DESCRIÇÃO` do `infraLupaSelect`); o primitivo os põe na ordem do
/// documento.
pub(crate) fn campos_do_envio(
    tela: &TelaDeEnvio,
    destino: &Destino,
    manter_aberto: bool,
    remover_anotacoes: bool,
) -> Vec<(&'static str, String)> {
    let mut campos = vec![
        (ORGAO, tela.orgao.clone()),
        (TEXTO_DA_UNIDADE, String::new()),
    ];
    if manter_aberto {
        campos.push((MANTER_ABERTO, "on".to_owned()));
    }
    if remover_anotacoes {
        campos.push((REMOVER_ANOTACOES, "on".to_owned()));
    }
    campos.extend(tela.textos.iter().map(|texto| (*texto, String::new())));
    campos.push((
        UNIDADES,
        format!("{}\u{b1}{}", destino.id_unidade, destino.descricao),
    ));
    campos
}

/// O 302 do POST é o observado: `controlador.php?acao=arvore_visualizar`
/// (relativo, como gravado, ou no host e na pasta do POST: ver
/// `consulta_do_controlador`), com `acao_origem=procedimento_enviar` e o
/// id, cada chave uma vez.
pub(crate) fn redirecionamento_esperado(
    localizacao: Option<&str>,
    url_do_envio: &str,
    id: &str,
) -> bool {
    let Some(consulta) = localizacao.and_then(|l| consulta_do_controlador(l, url_do_envio)) else {
        return false;
    };
    let valores = |chave: &str| -> Vec<&str> {
        consulta
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(k, _)| *k == chave)
            .map(|(_, v)| v)
            .collect()
    };
    valores("acao") == ["arvore_visualizar"]
        && valores("acao_origem") == [ACAO_DE_ENVIAR]
        && valores("id_procedimento") == [id]
}
