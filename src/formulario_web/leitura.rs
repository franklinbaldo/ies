//! Leitura estrita do `<form>` servido: localização, `action` e controles.
use super::codificacao::decodificar_atributo;
use super::{ErroSubmissaoFormulario as E, OperacaoDeFormulario};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::instalacao::{Instalacao, host_valido};
use crate::transporte::{FalhaTransporte, conferir_chave_de_consulta, conferir_valor_de_consulta};
use std::collections::HashSet;

/// URL HTTPS absoluta decomposta, sem fragmento nem caracteres de controle.
pub(super) struct UrlHttps<'a> {
    pub(super) host: &'a str,
    pub(super) caminho: &'a str,
    consulta: Option<&'a str>,
}

impl<'a> UrlHttps<'a> {
    pub(super) fn interpretar(url: &'a str) -> Option<Self> {
        let resto = url.strip_prefix("https://")?;
        if !url.bytes().all(|b| (33..=126).contains(&b)) || url.contains(['#', '\\']) {
            return None;
        }
        let (host, caminho_e_consulta) = resto.split_once('/')?;
        if !host_valido(host) {
            return None;
        }
        let (caminho, consulta) = match caminho_e_consulta.split_once('?') {
            Some((caminho, consulta)) => (caminho, Some(consulta)),
            None => (caminho_e_consulta, None),
        };
        Some(Self {
            host,
            caminho,
            consulta,
        })
    }
}

/// Valor bruto lido da fonte da tag (não do mapa do `tl`, defeituoso).
fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    valor_do_atributo(tag, nome).map_err(E::AtributoHtml)
}

/// Exatamente um `<form>` com o `id` ou `name` da operação.
pub(crate) fn localizar_formulario<'d>(
    dom: &'d tl::VDom<'d>,
    identificador: &str,
) -> Result<&'d tl::HTMLTag<'d>, E> {
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
        [] => Err(E::FormularioAusenteNaTela),
        [form] => Ok(form),
        _ => Err(E::FormulariosComMesmoIdentificadorNaTela),
    }
}

pub(super) fn conferir_metodo_e_codificacao(form: &tl::HTMLTag<'_>) -> Result<(), E> {
    if !atributo(form, "method")?.is_some_and(|m| m.eq_ignore_ascii_case("post")) {
        return Err(E::MetodoDoFormularioDiferenteDePost);
    }
    if atributo(form, "enctype")?
        .is_some_and(|v| !v.eq_ignore_ascii_case("application/x-www-form-urlencoded"))
    {
        return Err(E::EnctypeDoFormularioNaoUrlencoded);
    }
    if atributo(form, "accept-charset")?.is_some() {
        return Err(E::AcceptCharsetPublicadoNoFormulario);
    }
    Ok(())
}

fn tem_esquema(referencia: &str) -> bool {
    let fim = referencia.find(['/', '?']).unwrap_or(referencia.len());
    referencia[..fim]
        .split_once(':')
        .is_some_and(|(esquema, _)| {
            !esquema.is_empty()
                && esquema
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        })
}

/// Resolve o atributo `action` contra a URL **servida** e confere instalação,
/// ação e `infra_hash`. Não normaliza `..`: o caminho tem de ser exato.
pub(super) fn resolver_action(
    form: &tl::HTMLTag<'_>,
    servida: &UrlHttps<'_>,
    instalacao: &Instalacao,
    operacao: &OperacaoDeFormulario,
) -> Result<String, E> {
    let bruto = atributo(form, "action")?.ok_or(E::AcaoDoFormularioAusente)?;
    let decodificado =
        decodificar_atributo(&bruto).ok_or_else(|| E::EntidadeHtmlDesconhecidaNoAtributo {
            campo: "action".to_owned(),
        })?;
    // O navegador não envia o fragmento (`#ancoraBarraPesquisa` observado).
    let referencia = decodificado.split('#').next().unwrap_or_default();
    if referencia.is_empty() {
        return Err(E::AcaoDoFormularioAusente);
    }
    let absoluta = if referencia.starts_with("https://") {
        referencia.to_owned()
    } else if let Some(resto) = referencia.strip_prefix("//") {
        format!("https://{resto}")
    } else if referencia.starts_with('/') {
        format!("https://{}{referencia}", servida.host)
    } else if tem_esquema(referencia) {
        return Err(E::AcaoDoFormularioNaoEhUrlHttps);
    } else {
        match servida.caminho.rsplit_once('/') {
            Some((pasta, _)) => format!("https://{}/{pasta}/{referencia}", servida.host),
            None => format!("https://{}/{referencia}", servida.host),
        }
    };
    let alvo = UrlHttps::interpretar(&absoluta).ok_or(E::AcaoDoFormularioForaDaInstalacao)?;
    if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
        return Err(E::AcaoDoFormularioForaDaInstalacao);
    }
    conferir_consulta(alvo.consulta.unwrap_or_default(), operacao.acao)?;
    Ok(absoluta)
}

fn conferir_consulta(consulta: &str, acao: &str) -> Result<(), E> {
    let (mut acoes, mut acoes_iguais, mut hashes) = (0, 0, 0);
    for item in consulta.split('&').filter(|item| !item.is_empty()) {
        let (nome, valor) = item
            .split_once('=')
            .ok_or(E::Transporte(FalhaTransporte::UrlInvalida))?;
        conferir_chave_de_consulta(nome).map_err(E::Transporte)?;
        conferir_valor_de_consulta(valor).map_err(E::Transporte)?;
        match nome {
            "acao" => {
                acoes += 1;
                acoes_iguais += usize::from(valor == acao);
            }
            "infra_hash" => {
                if valor.len() != 64 || !valor.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(E::AcaoDoFormularioSemInfraHashUnico);
                }
                hashes += 1;
            }
            _ => {}
        }
    }
    if acoes != 1 || acoes_iguais != 1 {
        return Err(E::AcaoDoFormularioDiferenteDaOperacao);
    }
    if hashes != 1 {
        return Err(E::AcaoDoFormularioSemInfraHashUnico);
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

fn valor_decodificado(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<String, E> {
    decodificar_atributo(&atributo(tag, "value")?.unwrap_or_default()).ok_or_else(|| {
        E::EntidadeHtmlDesconhecidaNoAtributo {
            campo: nome.to_owned(),
        }
    })
}

fn classificar(tag: &tl::HTMLTag<'_>, botao: &str) -> Result<Option<Controle>, E> {
    let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
    if !matches!(elemento.as_str(), "input" | "button" | "select" | "textarea")
        // Navegadores não enviam controles desabilitados (com ou sem valor).
        || atributo_presente(tag, "disabled").map_err(E::AtributoHtml)?
    {
        return Ok(None);
    }
    let Some(bruto) = atributo(tag, "name")?.filter(|n| !n.is_empty()) else {
        return Ok(None);
    };
    let nome =
        decodificar_atributo(&bruto).ok_or_else(|| E::EntidadeHtmlDesconhecidaNoAtributo {
            campo: bruto.clone(),
        })?;
    if nome == "hdnAcao" {
        return Err(E::HdnAcaoPublicadoSemRegraDaOperacao);
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
        ("input" | "button", "submit") if nome == botao => {
            let valor = valor_decodificado(tag, &nome)?;
            Some(Controle::Botao { nome, valor })
        }
        ("input" | "button", "submit" | "button" | "reset" | "image") | ("button", _) => None,
        ("select", _) if atributo_presente(tag, "multiple").map_err(E::AtributoHtml)? => {
            Some(Controle::SelecaoMultipla(nome))
        }
        _ => Some(Controle::Usuario(nome)),
    })
}

/// Lê os controles do formulário e aplica as recusas estruturais.
pub(super) fn ler_controles(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    operacao: &OperacaoDeFormulario,
) -> Result<Vec<Controle>, E> {
    let mut controles = Vec::new();
    let mut ocultos = HashSet::new();
    let mut botoes = 0;
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let Some(controle) = classificar(tag, operacao.botao_de_envio)? else {
            continue;
        };
        match &controle {
            Controle::Oculto { nome, .. } if !ocultos.insert(nome.clone()) => {
                return Err(E::CampoOcultoRepetidoNoFormulario {
                    campo: nome.clone(),
                });
            }
            Controle::Botao { .. } => botoes += 1,
            _ => {}
        }
        controles.push(controle);
    }
    match botoes {
        0 => return Err(E::BotaoDeEnvioAusenteNoFormulario),
        1 => {}
        _ => return Err(E::BotoesDeEnvioRepetidosNoFormulario),
    }
    if let Some(ausente) = operacao
        .campos_ocultos_exigidos
        .iter()
        .find(|nome| !ocultos.contains(**nome))
    {
        return Err(E::CampoOcultoAusenteNoFormulario {
            campo: (*ausente).to_owned(),
        });
    }
    Ok(controles)
}
