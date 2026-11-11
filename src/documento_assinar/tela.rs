//! A tela "Assinatura de Documento" (`form#frmAssinaturas`) e a resposta do
//! POST (contracts/cli/documento-assinar.md; observação em
//! docs/como-o-sei-funciona/assinatura-do-documento.md).
//!
//! Evidência (cassettes públicas do Todos, sete instalações, telas
//! idênticas fora `btnCopiarCodigo`): ocultos `hdnInfraTipoPagina`,
//! `hdnIdUsuario`, `hdnFormaAutenticacao`, `hdnLinkRetorno`,
//! `hdnFlagAssinatura`, `hdnIdDocumentos`, `hdnIdBlocos`; `selOrgao`,
//! `txtUsuario`, `selCargoFuncao`, `pwdSenha`; botões `type=button`. O
//! `assinarSenha()` põe `hdnFormaAutenticacao='S'` e chama `submit()`. A
//! resposta aceita é a mesma tela com `inicializar()` chamando `finalizar()`.
use super::erro::ErroTelaDeAssinatura;
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, decodificar_atributo, localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};

pub(crate) const FORMULARIO_DE_ASSINATURA: &str = "frmAssinaturas";
pub(crate) const FORMA_DE_AUTENTICACAO: &str = "hdnFormaAutenticacao";
/// `S`: senha (o `assinarSenha()` da tela).
pub(crate) const PELA_SENHA: &str = "S";
pub(crate) const ORGAO: &str = "selOrgao";
pub(crate) const USUARIO: &str = "txtUsuario";
pub(crate) const CARGO: &str = "selCargoFuncao";
pub(crate) const SENHA: &str = "pwdSenha";
const ID_USUARIO: &str = "hdnIdUsuario";
const DOCUMENTOS: &str = "hdnIdDocumentos";
const OCULTOS: [&str; 7] = [
    "hdnInfraTipoPagina",
    ID_USUARIO,
    FORMA_DE_AUTENTICACAO,
    "hdnLinkRetorno",
    "hdnFlagAssinatura",
    DOCUMENTOS,
    "hdnIdBlocos",
];
/// A opção vazia de `select` servida pela Infra.
const VAZIA: &str = "null";

/// Assinar com a senha: janela sem barra da unidade, aberta pelo link da
/// tela do documento conferido na árvore.
pub(crate) const ASSINAR_DOCUMENTO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::DocumentoAssinar,
    formulario: FORMULARIO_DE_ASSINATURA,
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        ID_USUARIO,
        FORMA_DE_AUTENTICACAO,
        "hdnFlagAssinatura",
        DOCUMENTOS,
    ],
    campos_ocultos_reescritos: &[FORMA_DE_AUTENTICACAO],
    campos_usuario: &[ORGAO, USUARIO, CARGO, SENHA],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AssinaturaDoDocumento,
    },
};

/// O que a tela conferida publica.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDeAssinatura {
    /// `txtUsuario` como servido: o usuário da sessão, como a caneta o mostra.
    pub(crate) assinante: String,
    /// O `value` da opção `selected` de `selOrgao`.
    pub(crate) orgao: String,
    /// Os `value` de `selCargoFuncao`, sem a opção vazia, na ordem servida.
    pub(crate) cargos: Vec<String>,
    /// A opção `selected` (ou a única, como o `inicializar()`).
    pub(crate) cargo_da_tela: Option<String>,
}

/// Confere a tela servida antes de qualquer POST: lista fechada de
/// controles enviáveis, o documento, o assinante, o órgão e a senha.
pub(crate) fn conferir_tela_de_assinatura(
    html: &str,
    id_documento: &str,
) -> Result<TelaDeAssinatura, ErroTelaDeAssinatura> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroTelaDeAssinatura::HtmlDaTelaDeAssinaturaRecusadoPeloParser)?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, FORMULARIO_DE_ASSINATURA)
        .map_err(ErroTelaDeAssinatura::FormularioDeAssinaturaAusente)?;
    let (mut documentos, mut id_usuario, mut assinante) = (None, None, None);
    let (mut orgao, mut cargos, mut senha) = (None, None, false);
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if !matches!(
            elemento.as_str(),
            "input" | "button" | "select" | "textarea"
        ) || presente(tag, "disabled")?
        {
            continue;
        }
        let Some(nome) = valor(tag, "name")?.filter(|n| !n.is_empty()) else {
            continue;
        };
        let padrao = if elemento == "button" {
            "submit"
        } else {
            "text"
        };
        let tipo =
            valor(tag, "type")?.map_or_else(|| padrao.to_owned(), |t| t.to_ascii_lowercase());
        let servido = valor(tag, "value")?.unwrap_or_default();
        match (elemento.as_str(), tipo.as_str(), nome.as_str()) {
            ("input", "hidden", DOCUMENTOS) => documentos = Some(servido),
            ("input", "hidden", ID_USUARIO) => id_usuario = Some(servido),
            ("input", "hidden", n) if OCULTOS.contains(&n) => {}
            // `button`/`reset` não participam do envio (`submit()` por script).
            ("input" | "button", "button" | "reset", _) => {}
            ("input", "text", USUARIO) if assinante.is_none() => assinante = Some(servido),
            ("input", "password", SENHA) if !senha => senha = true,
            ("select", _, ORGAO) if orgao.is_none() => orgao = Some(opcoes(tag, parser)?),
            ("select", _, CARGO) if cargos.is_none() => cargos = Some(opcoes(tag, parser)?),
            _ => return Err(ErroTelaDeAssinatura::CampoNaoPrevistoNaTelaDeAssinatura),
        }
    }
    if documentos.as_deref() != Some(id_documento) {
        return Err(ErroTelaDeAssinatura::TelaDeAssinaturaDeOutroDocumento);
    }
    let assinante = assinante.unwrap_or_default();
    if assinante.trim().is_empty() || id_usuario.is_none_or(|id| id.trim().is_empty()) {
        return Err(ErroTelaDeAssinatura::AssinanteNaoPublicadoNaTela);
    }
    let orgao = match orgao.unwrap_or_default().as_slice() {
        [] => None,
        opcoes => {
            let marcadas: Vec<_> = opcoes.iter().filter(|(_, s)| *s).collect();
            match marcadas.as_slice() {
                [(valor, _)] if valor != VAZIA && !valor.is_empty() => Some(valor.clone()),
                _ => None,
            }
        }
    }
    .ok_or(ErroTelaDeAssinatura::OrgaoDoAssinanteNaoSelecionado)?;
    if !senha {
        return Err(ErroTelaDeAssinatura::CampoDeSenhaAusenteNaTela);
    }
    let opcoes = cargos.ok_or(ErroTelaDeAssinatura::CampoNaoPrevistoNaTelaDeAssinatura)?;
    let validas: Vec<&(String, bool)> = opcoes
        .iter()
        .filter(|(v, _)| v != VAZIA && !v.is_empty())
        .collect();
    let marcadas: Vec<&String> = validas.iter().filter(|(_, s)| *s).map(|(v, _)| v).collect();
    let cargo_da_tela = match (marcadas.as_slice(), validas.as_slice()) {
        ([uma], _) => Some((*uma).clone()),
        ([], [(unica, _)]) => Some(unica.clone()),
        _ => None,
    };
    Ok(TelaDeAssinatura {
        assinante,
        orgao,
        cargos: validas.into_iter().map(|(v, _)| v.clone()).collect(),
        cargo_da_tela,
    })
}

/// As opções do `select`: `value` (decodificado) e `selected`.
fn opcoes(
    select: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<Vec<(String, bool)>, ErroTelaDeAssinatura> {
    let mut lidas = Vec::new();
    for tag in select
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name().as_utf8_str().eq_ignore_ascii_case("option"))
    {
        let valor =
            valor(tag, "value")?.ok_or(ErroTelaDeAssinatura::TelaDeAssinaturaNaoInterpretada)?;
        lidas.push((valor, presente(tag, "selected")?));
    }
    Ok(lidas)
}

fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, ErroTelaDeAssinatura> {
    atributo_presente(tag, nome).map_err(ErroTelaDeAssinatura::TelaDeAssinaturaComAtributoIlegivel)
}

/// Valor do atributo com as entidades decodificadas.
fn valor(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, ErroTelaDeAssinatura> {
    match valor_do_atributo(tag, nome)
        .map_err(ErroTelaDeAssinatura::TelaDeAssinaturaComAtributoIlegivel)?
    {
        Some(bruto) => decodificar_atributo(&bruto)
            .map(Some)
            .ok_or(ErroTelaDeAssinatura::TelaDeAssinaturaNaoInterpretada),
        None => Ok(None),
    }
}

/// A resposta aceita: `inicializar()` começa com `//se realizou
/// assinatura` e `finalizar();` (na tela do GET, a linha seguinte é o `if`
/// do cargo).
pub(crate) fn assinatura_realizada(html: &str) -> bool {
    let linhas: Vec<&str> = html
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    linhas.windows(3).any(|janela| {
        janela
            == [
                "function inicializar(){",
                "//se realizou assinatura",
                "finalizar();",
            ]
    })
}
