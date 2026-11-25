//! Leitura estrita do `<form>` servido: localização, `action` e controles.
use super::{EnvioDoFormulario, ErroSubmissaoFormulario, OperacaoDeFormulario};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;
pub(super) use crate::url_observada::UrlHttps;
use crate::url_observada::{pasta, tem_esquema};
use std::collections::HashSet;

/// Exatamente um `<form>` com o `id` ou `name` da operação.
pub(crate) fn localizar_formulario<'d>(
    dom: &'d crate::tela_html::Documento,
    identificador: &str,
) -> Result<crate::tela_html::Elemento<'d>, ErroSubmissaoFormulario> {
    let mut achados = Vec::new();
    for form in dom.elementos("form") {
        if ["id", "name"]
            .into_iter()
            .any(|chave| form.atributo(chave) == Some(identificador))
        {
            achados.push(form);
        }
    }
    match achados.as_slice() {
        [] => Err(ErroSubmissaoFormulario::FormularioAusenteNaTela),
        [form] => Ok(*form),
        _ => Err(ErroSubmissaoFormulario::FormulariosComMesmoIdentificadorNaTela),
    }
}

pub(super) fn conferir_metodo_e_codificacao(
    form: crate::tela_html::Elemento<'_>,
) -> Result<(), ErroSubmissaoFormulario> {
    if !form
        .atributo("method")
        .is_some_and(|m| m.eq_ignore_ascii_case("post"))
    {
        return Err(ErroSubmissaoFormulario::MetodoDoFormularioDiferenteDePost);
    }
    if form
        .atributo("enctype")
        .is_some_and(|v| !v.eq_ignore_ascii_case("application/x-www-form-urlencoded"))
    {
        return Err(ErroSubmissaoFormulario::EnctypeDoFormularioNaoUrlencoded);
    }
    if form.atributo("accept-charset").is_some() {
        return Err(ErroSubmissaoFormulario::AcceptCharsetPublicadoNoFormulario);
    }
    Ok(())
}

fn validar_destino(
    absoluta: String,
    instalacao: &Instalacao,
    operacao: &OperacaoDeFormulario,
) -> Result<String, ErroSubmissaoFormulario> {
    let alvo = UrlHttps::interpretar(&absoluta)
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao)?;
    if !(instalacao.autoriza_controlador(alvo.host, alvo.caminho)
        || (operacao.acao == AcaoDoSei::EditorSalvar
            && instalacao.autoriza_editor_processar(alvo.host, alvo.caminho)))
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao);
    }
    conferir_consulta_url(alvo.consulta, operacao.acao)?;
    Ok(absoluta)
}

/// Resolve o atributo `action` contra a URL **servida** e confere instalação,
/// ação e `infra_hash`. Não normaliza `..`: o caminho tem de ser exato.
pub(super) fn resolver_action(
    form: crate::tela_html::Elemento<'_>,
    servida: &UrlHttps<'_>,
    instalacao: &Instalacao,
    operacao: &OperacaoDeFormulario,
) -> Result<String, ErroSubmissaoFormulario> {
    let absoluta = resolver_referencia(form, servida)?;
    validar_destino(absoluta, instalacao, operacao)
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
    validar_destino(absoluta, instalacao, operacao)
}

/// `action` do login: resolvido contra a URL servida, no **host** da
/// instalação, sem segmentos `.`/`..` e com consulta de nomes canônicos.
/// O login não tem `acao=` nem `infra_hash`.
pub(super) fn resolver_action_no_host(
    form: crate::tela_html::Elemento<'_>,
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
    ConsultaDoControlador::da_url(alvo.consulta)?;
    Ok(absoluta)
}

/// `action` absoluto em HTTPS, resolvido contra a URL servida, sem o
/// fragmento. Não normaliza `..`.
fn resolver_referencia(
    form: crate::tela_html::Elemento<'_>,
    servida: &UrlHttps<'_>,
) -> Result<String, ErroSubmissaoFormulario> {
    let bruto = form
        .atributo("action")
        .ok_or(ErroSubmissaoFormulario::AcaoDoFormularioAusente)?;
    // O navegador não envia o fragmento (`#ancoraBarraPesquisa` observado).
    let referencia = bruto.split_once('#').map_or(bruto, |(antes, _)| antes);
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

fn conferir_consulta_url(
    consulta: Option<&str>,
    acao: AcaoDoSei,
) -> Result<(), ErroSubmissaoFormulario> {
    let consulta = ConsultaDoControlador::da_url(consulta)?;
    if consulta.acao() != Ok(acao.como_na_url()) {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao);
    }
    if !consulta
        .unico("infra_hash")
        .is_ok_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico);
    }
    Ok(())
}

pub(super) fn conferir_consulta(
    consulta: &str,
    acao: AcaoDoSei,
) -> Result<(), ErroSubmissaoFormulario> {
    conferir_consulta_url(Some(consulta), acao)
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

fn valor(tag: crate::tela_html::Elemento<'_>) -> String {
    tag.atributo("value").unwrap_or_default().to_owned()
}

fn classificar(
    tag: crate::tela_html::Elemento<'_>,
    operacao: &OperacaoDeFormulario,
) -> Result<Option<Controle>, ErroSubmissaoFormulario> {
    let botao = botao_do_envio(operacao.envio);
    let elemento = tag.nome().to_ascii_lowercase();
    if !matches!(elemento.as_str(), "input" | "button" | "select" | "textarea")
        // Navegadores não enviam controles desabilitados (com ou sem valor).
        || tag.presente("disabled")
    {
        return Ok(None);
    }
    let Some(nome) = tag.atributo("name").filter(|n| !n.is_empty()) else {
        return Ok(None);
    };
    // `hdnAcao` só com a regra observada da operação (o `onsubmit` do login).
    if nome == "hdnAcao" && !operacao.campos_ocultos_reescritos.contains(&"hdnAcao") {
        return Err(ErroSubmissaoFormulario::HdnAcaoPublicadoSemRegraDaOperacao);
    }
    let padrao = if elemento == "button" {
        "submit"
    } else {
        "text"
    };
    let tipo = tag.atributo("type").map(str::to_ascii_lowercase);
    Ok(
        match (elemento.as_str(), tipo.as_deref().unwrap_or(padrao)) {
            ("input", "hidden") => Some(Controle::Oculto {
                nome: nome.to_owned(),
                valor: valor(tag),
            }),
            ("input" | "button", "submit") if botao == Some(nome) => Some(Controle::Botao {
                nome: nome.to_owned(),
                valor: valor(tag),
            }),
            ("input" | "button", "submit" | "button" | "reset" | "image") | ("button", _) => None,
            ("select", _) if tag.presente("multiple") => {
                Some(Controle::SelecaoMultipla(nome.to_owned()))
            }
            _ => Some(Controle::Usuario(nome.to_owned())),
        },
    )
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
    form: crate::tela_html::Elemento<'_>,
    operacao: &OperacaoDeFormulario,
) -> Result<Vec<Controle>, ErroSubmissaoFormulario> {
    let mut controles = Vec::new();
    let mut ocultos = HashSet::new();
    let mut botoes = 0;
    for tag in form.descendentes() {
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
            Controle::Oculto { .. } | Controle::Usuario(_) | Controle::SelecaoMultipla(_) => {}
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
