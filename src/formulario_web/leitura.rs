//! Leitura estrita do `<form>` servido: localização, `action` e controles.
use super::codificacao::decodificar_atributo;
use super::{EnvioDoFormulario, ErroSubmissaoFormulario, OperacaoDeFormulario};
use crate::acao_do_sei::AcaoDoSei;
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::transporte::{FalhaTransporte, conferir_chave_de_consulta, conferir_valor_de_consulta};
pub(super) use crate::url_observada::UrlHttps;
use crate::url_observada::{pasta, tem_esquema};
use std::collections::HashSet;

/// Valor bruto lido da fonte da tag (não do mapa do `tl`, defeituoso).
fn atributo(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, ErroSubmissaoFormulario> {
    valor_do_atributo(tag, nome).map_err(ErroSubmissaoFormulario::AtributoHtml)
}

/// Exatamente um `<form>` com o `id` ou `name` da operação.
pub(crate) fn localizar_formulario<'d>(
    dom: &'d tl::VDom<'d>,
    identificador: &str,
) -> Result<&'d tl::HTMLTag<'d>, ErroSubmissaoFormulario> {
    let parser = dom.parser();
    let mut achados = Vec::new();
    for form in dom
        .query_selector("form")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
    {
        for chave in ["id", "name"] {
            if atributo(form, chave)?.is_some_and(|v| v == identificador) {
                achados.push(form);
                break;
            }
        }
    }
    match achados.as_slice() {
        [] => Err(ErroSubmissaoFormulario::FormularioAusenteNaTela),
        [form] => Ok(form),
        _ => Err(ErroSubmissaoFormulario::FormulariosComMesmoIdentificadorNaTela),
    }
}

pub(super) fn conferir_metodo_e_codificacao(
    form: &tl::HTMLTag<'_>,
) -> Result<(), ErroSubmissaoFormulario> {
    if !atributo(form, "method")?.is_some_and(|m| m.eq_ignore_ascii_case("post")) {
        return Err(ErroSubmissaoFormulario::MetodoDoFormularioDiferenteDePost);
    }
    if atributo(form, "enctype")?
        .is_some_and(|v| !v.eq_ignore_ascii_case("application/x-www-form-urlencoded"))
    {
        return Err(ErroSubmissaoFormulario::EnctypeDoFormularioNaoUrlencoded);
    }
    if atributo(form, "accept-charset")?.is_some() {
        return Err(ErroSubmissaoFormulario::AcceptCharsetPublicadoNoFormulario);
    }
    Ok(())
}

/// Resolve o atributo `action` contra a URL **servida** e confere instalação,
/// ação e `infra_hash`. Não normaliza `..`: o caminho tem de ser exato.
pub(super) fn resolver_action(
    form: &tl::HTMLTag<'_>,
    servida: &UrlHttps<'_>,
    instalacao: &Instalacao,
    operacao: &OperacaoDeFormulario,
) -> Result<String, ErroSubmissaoFormulario> {
    let absoluta = resolver_referencia(form, servida)?;
    let alvo = UrlHttps::interpretar(&absoluta)
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao)?;
    if !(instalacao.autoriza_controlador(alvo.host, alvo.caminho)
        || (operacao.acao == AcaoDoSei::EditorSalvar
            && instalacao.autoriza_editor_processar(alvo.host, alvo.caminho)))
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    conferir_consulta(alvo.consulta.unwrap_or_default(), operacao.acao)?;
    Ok(absoluta)
}

/// A action que o handler do formulário publica no script da tela ativa.
/// O destino continua restrito ao controlador da instalação e ao hash
/// assinado; não aceita URL absoluta, esquema, caminho raiz ou travessia.
pub(super) fn resolver_action_declarada(
    url_declarada: &str,
    servida: &UrlHttps<'_>,
    instalacao: &Instalacao,
    operacao: &OperacaoDeFormulario,
) -> Result<String, ErroSubmissaoFormulario> {
    if url_declarada.contains(':')
        || url_declarada.starts_with('/')
        || url_declarada.split(['/', '?']).any(|s| s == "..")
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    let absoluta = match servida.caminho.rsplit_once('/') {
        Some((pasta, _)) => format!("https://{}/{pasta}/{url_declarada}", servida.host),
        None => format!("https://{}/{url_declarada}", servida.host),
    };
    let alvo = UrlHttps::interpretar(&absoluta)
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao)?;
    if !(instalacao.autoriza_controlador(alvo.host, alvo.caminho)
        || (operacao.acao == AcaoDoSei::EditorSalvar
            && instalacao.autoriza_editor_processar(alvo.host, alvo.caminho)))
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    conferir_consulta(alvo.consulta.unwrap_or_default(), operacao.acao)?;
    Ok(absoluta)
}

/// `action` do login: resolvido contra a URL servida, no **host** da
/// instalação, sem segmentos `.`/`..` e com consulta de nomes canônicos.
/// O login não tem `acao=` nem `infra_hash`.
pub(super) fn resolver_action_no_host(
    form: &tl::HTMLTag<'_>,
    servida: &UrlHttps<'_>,
    instalacao: &Instalacao,
) -> Result<String, ErroSubmissaoFormulario> {
    let absoluta = resolver_referencia(form, servida)?;
    let alvo = UrlHttps::interpretar(&absoluta)
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao)?;
    if !alvo.host.eq_ignore_ascii_case(instalacao.host())
        || alvo
            .caminho
            .split('/')
            .any(|segmento| segmento == "." || segmento == "..")
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    for item in alvo
        .consulta
        .unwrap_or_default()
        .split('&')
        .filter(|item| !item.is_empty())
    {
        let (nome, valor) = item
            .split_once('=')
            .ok_or(ErroSubmissaoFormulario::Transporte(
                FalhaTransporte::UrlInvalida,
            ))?;
        conferir_chave_de_consulta(nome)?;
        conferir_valor_de_consulta(valor)?;
    }
    Ok(absoluta)
}

/// `action` absoluto em HTTPS, resolvido contra a URL servida, sem o
/// fragmento. Não normaliza `..`.
fn resolver_referencia(
    form: &tl::HTMLTag<'_>,
    servida: &UrlHttps<'_>,
) -> Result<String, ErroSubmissaoFormulario> {
    let bruto =
        atributo(form, "action")?.ok_or(ErroSubmissaoFormulario::AcaoDoFormularioAusente)?;
    let decodificado = decodificar_atributo(&bruto).ok_or_else(|| {
        ErroSubmissaoFormulario::EntidadeHtmlDesconhecidaNoAtributo {
            campo: "action".to_owned(),
        }
    })?;
    // O navegador não envia o fragmento (`#ancoraBarraPesquisa` observado).
    let referencia = decodificado.split('#').next().unwrap_or_default();
    if referencia.is_empty() {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioAusente);
    }
    let absoluta = if referencia.starts_with("https://") {
        referencia.to_owned()
    } else if let Some(resto) = referencia.strip_prefix("//") {
        format!("https://{resto}")
    } else if referencia.starts_with('/') {
        format!("https://{}{referencia}", servida.host)
    } else if tem_esquema(referencia) {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioNaoEhUrlHttps);
    } else {
        match pasta(servida.caminho) {
            Some(diretorio) => format!("https://{}/{diretorio}/{referencia}", servida.host),
            None => format!("https://{}/{referencia}", servida.host),
        }
    };
    Ok(absoluta)
}

pub(super) fn conferir_consulta(
    consulta: &str,
    acao: AcaoDoSei,
) -> Result<(), ErroSubmissaoFormulario> {
    let (mut acoes, mut acoes_iguais, mut hashes) = (0, 0, 0);
    for item in consulta.split('&').filter(|item| !item.is_empty()) {
        let (nome, valor) = item
            .split_once('=')
            .ok_or(ErroSubmissaoFormulario::Transporte(
                FalhaTransporte::UrlInvalida,
            ))?;
        conferir_chave_de_consulta(nome)?;
        conferir_valor_de_consulta(valor)?;
        match nome {
            "acao" => {
                acoes += 1;
                acoes_iguais += usize::from(AcaoDoSei::ler(valor) == Some(acao));
            }
            "infra_hash" => {
                if valor.len() != 64 || !valor.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico);
                }
                hashes += 1;
            }
            _ => {}
        }
    }
    if acoes != 1 || acoes_iguais != 1 {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao);
    }
    if hashes != 1 {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico);
    }
    Ok(())
}

/// Controle bem-sucedido do formulário, na ordem do documento.
pub(super) enum Controle {
    Oculto {
        nome: String,
        valor: String,
    },
    Usuario(String),
    /// `<select multiple>`: o navegador envia um par por opção selecionada.
    SelecaoMultipla(String),
    Botao {
        nome: String,
        valor: String,
    },
}

impl Controle {
    pub(super) fn nome(&self) -> &str {
        match self {
            Self::Oculto { nome, .. }
            | Self::Botao { nome, .. }
            | Self::Usuario(nome)
            | Self::SelecaoMultipla(nome) => nome,
        }
    }
}

fn valor_decodificado(
    tag: &tl::HTMLTag<'_>,
    nome: &str,
) -> Result<String, ErroSubmissaoFormulario> {
    decodificar_atributo(&atributo(tag, "value")?.unwrap_or_default()).ok_or_else(|| {
        ErroSubmissaoFormulario::EntidadeHtmlDesconhecidaNoAtributo {
            campo: nome.to_owned(),
        }
    })
}

fn classificar(
    tag: &tl::HTMLTag<'_>,
    operacao: &OperacaoDeFormulario,
) -> Result<Option<Controle>, ErroSubmissaoFormulario> {
    let botao = botao_do_envio(operacao.envio);
    let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
    if !matches!(elemento.as_str(), "input" | "button" | "select" | "textarea")
        // Navegadores não enviam controles desabilitados (com ou sem valor).
        || atributo_presente(tag, "disabled")?
    {
        return Ok(None);
    }
    let Some(bruto) = atributo(tag, "name")?.filter(|n| !n.is_empty()) else {
        return Ok(None);
    };
    let nome = decodificar_atributo(&bruto).ok_or_else(|| {
        ErroSubmissaoFormulario::EntidadeHtmlDesconhecidaNoAtributo {
            campo: bruto.clone(),
        }
    })?;
    // `hdnAcao` só com a regra observada da operação (o `onsubmit` do login).
    if nome == "hdnAcao" && !operacao.campos_ocultos_reescritos.contains(&"hdnAcao") {
        return Err(ErroSubmissaoFormulario::HdnAcaoPublicadoSemRegraDaOperacao);
    }
    let padrao = if elemento == "button" {
        "submit"
    } else {
        "text"
    };
    let tipo = atributo(tag, "type")?.map_or_else(|| padrao.to_owned(), |t| t.to_ascii_lowercase());
    Ok(match (elemento.as_str(), tipo.as_str()) {
        ("input", "hidden") => {
            let valor = valor_decodificado(tag, &nome)?;
            Some(Controle::Oculto { nome, valor })
        }
        ("input" | "button", "submit") if botao == Some(nome.as_str()) => {
            let valor = valor_decodificado(tag, &nome)?;
            Some(Controle::Botao { nome, valor })
        }
        ("input" | "button", "submit" | "button" | "reset" | "image") | ("button", _) => None,
        ("select", _) if atributo_presente(tag, "multiple")? => {
            Some(Controle::SelecaoMultipla(nome))
        }
        _ => Some(Controle::Usuario(nome)),
    })
}

/// Nome do botão servido que a operação declara (o desabilitado pelo
/// `onsubmit` é conferido e depois retirado); `submit()` por script não
/// tem botão.
fn botao_do_envio(envio: EnvioDoFormulario) -> Option<&'static str> {
    match envio {
        EnvioDoFormulario::BotaoDeEnvio(nome)
        | EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit(nome) => Some(nome),
        EnvioDoFormulario::SubmitPorScript
        | EnvioDoFormulario::SubmitPorScriptComCampoCriado(_) => None,
    }
}

/// Lê os controles do formulário e aplica as recusas estruturais.
pub(super) fn ler_controles(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    operacao: &OperacaoDeFormulario,
) -> Result<Vec<Controle>, ErroSubmissaoFormulario> {
    let mut controles = Vec::new();
    let mut ocultos = HashSet::new();
    let mut botoes = 0;
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let Some(controle) = classificar(tag, operacao)? else {
            continue;
        };
        match &controle {
            Controle::Oculto { nome, .. } if !ocultos.insert(nome.clone()) => {
                return Err(ErroSubmissaoFormulario::CampoOcultoRepetidoNoFormulario {
                    campo: nome.clone(),
                });
            }
            Controle::Botao { .. } => botoes += 1,
            _ => {}
        }
        controles.push(controle);
    }
    match (operacao.envio, botoes) {
        (
            EnvioDoFormulario::SubmitPorScript
            | EnvioDoFormulario::SubmitPorScriptComCampoCriado(_),
            _,
        )
        | (EnvioDoFormulario::BotaoDeEnvio(_), 1) => {}
        // Servido uma vez, mas desabilitado antes de o navegador montar o corpo.
        (EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit(_), 1) => {
            controles.retain(|c| !matches!(c, Controle::Botao { .. }));
        }
        (
            EnvioDoFormulario::BotaoDeEnvio(_)
            | EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit(_),
            0,
        ) => return Err(ErroSubmissaoFormulario::BotaoDeEnvioAusenteNoFormulario),
        (
            EnvioDoFormulario::BotaoDeEnvio(_)
            | EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit(_),
            _,
        ) => {
            return Err(ErroSubmissaoFormulario::BotoesDeEnvioRepetidosNoFormulario);
        }
    }
    if let Some(ausente) = operacao
        .campos_ocultos_exigidos
        .iter()
        .find(|nome| !ocultos.contains(**nome))
    {
        return Err(ErroSubmissaoFormulario::CampoOcultoAusenteNoFormulario {
            campo: (*ausente).to_owned(),
        });
    }
    Ok(controles)
}
