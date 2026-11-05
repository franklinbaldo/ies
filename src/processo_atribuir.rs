//! Leitor da tela de atribuição por HTML servido, sem executar JS.
// Evidência: UnB 4.0.9, GO e Todos tests/test_ciclo_atribuir_010.py @323eb01.
//! O select não informa a atribuição atual: a árvore do processo é a fonte.
use crate::formulario_web::{decodificar_atributo, localizar_formulario};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::transporte::texto_de_recusa::decodificar_entidades;

const FORMULARIO: &str = "frmAtividadeAtribuir";
const ACAO: &str = "procedimento_atribuicao_cadastrar";
const SELECAO: &str = "selAtribuicao";
const SEM_ATRIBUICAO: &str = "null";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UsuarioDaAtribuicao {
    pub(crate) id: String,
    pub(crate) login: String,
    pub(crate) rotulo: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDaAtribuicao {
    pub(crate) usuarios: Vec<UsuarioDaAtribuicao>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAtribuir {
    TelaForaDaForma,
    UsuarioNaoAtribuivel,
    UsuarioAmbiguo,
    SituacaoNaoObservada,
    AtribuicaoJaAplicada,
}
impl ErroAtribuir {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::TelaForaDaForma => "tela_da_atribuicao_fora_da_forma",
            Self::UsuarioNaoAtribuivel => "usuario_nao_atribuivel",
            Self::UsuarioAmbiguo => "usuario_ambiguo_na_atribuicao",
            Self::SituacaoNaoObservada => "situacao_da_atribuicao_nao_observada",
            Self::AtribuicaoJaAplicada => "atribuicao_ja_aplicada",
        }
    }
}

type E = ErroAtribuir;
fn valor(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    valor_do_atributo(tag, nome)
        .map_err(|_| E::TelaForaDaForma)?
        .map(|bruto| decodificar_atributo(&bruto).ok_or(E::TelaForaDaForma))
        .transpose()
}
fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, E> {
    atributo_presente(tag, nome).map_err(|_| E::TelaForaDaForma)
}

/// Confere a URL assinada servida sem a publicar nem fabricar outra.
fn acao_do_formulario(action: &str, id: &str) -> bool {
    let Some(consulta) = action.strip_prefix("controlador.php?") else {
        return false;
    };
    let mut acao = 0;
    let mut protocolo = 0;
    let mut hash = 0;
    for par in consulta.split('&') {
        let Some((nome, valor)) = par.split_once('=') else {
            return false;
        };
        match nome {
            "acao" if valor == ACAO => acao += 1,
            "id_procedimento" if valor == id => protocolo += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hash += 1;
            }
            "acao" | "id_procedimento" | "infra_hash" => return false,
            _ => {
                if nome.is_empty() || valor.contains(['#', '?']) {
                    return false;
                }
            }
        }
    }
    acao == 1 && protocolo == 1 && hash == 1
}

/// Opções observadas do select, sem executar scripts.
fn ler_opcoes(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<(Vec<UsuarioDaAtribuicao>, usize), E> {
    let mut usuarios = Vec::new();
    let mut item_nulo = 0;
    for filho in tag.children().all(parser) {
        let Some(opcao) = filho.as_tag() else {
            continue;
        };
        if opcao.name().as_utf8_str().as_ref() != "option" {
            return Err(E::TelaForaDaForma);
        }
        if presente(opcao, "disabled")? {
            return Err(E::TelaForaDaForma);
        }
        let id_usuario = valor(opcao, "value")?.ok_or(E::TelaForaDaForma)?;
        if id_usuario == SEM_ATRIBUICAO {
            if !presente(opcao, "selected")? {
                return Err(E::TelaForaDaForma);
            }
            item_nulo += 1;
            continue;
        }
        if presente(opcao, "selected")?
            || id_usuario.is_empty()
            || !id_usuario.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(E::TelaForaDaForma);
        }
        let rotulo = decodificar_entidades(&opcao.inner_text(parser))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let (login, nome) = rotulo.split_once(" - ").ok_or(E::TelaForaDaForma)?;
        if login.is_empty() || nome.is_empty() || login.trim() != login {
            return Err(E::TelaForaDaForma);
        }
        usuarios.push(UsuarioDaAtribuicao {
            id: id_usuario,
            login: login.to_owned(),
            rotulo,
        });
    }
    Ok((usuarios, item_nulo))
}

/// Confere a forma observada: POST, identidade, lista simples e opção null.
pub(crate) fn ler_tela(html: &str, id: &str) -> Result<TelaDaAtribuicao, E> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::TelaForaDaForma)?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, FORMULARIO).map_err(|_| E::TelaForaDaForma)?;
    if valor(form, "method")?.as_deref() != Some("post")
        || presente(form, "disabled")?
        || !valor(form, "action")?
            .as_deref()
            .is_some_and(|action| acao_do_formulario(action, id))
    {
        return Err(E::TelaForaDaForma);
    }

    let (mut pagina, mut protocolo, mut botao, mut selecao) = (0, 0, 0, 0);
    let mut usuarios = Vec::new();
    let mut item_nulo = 0;
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        match elemento.as_str() {
            "input" => {
                if presente(tag, "disabled")? || valor(tag, "type")?.as_deref() != Some("hidden") {
                    return Err(E::TelaForaDaForma);
                }
                match valor(tag, "name")?.as_deref() {
                    Some("hdnInfraTipoPagina") if valor(tag, "value")?.as_deref() == Some("2") => {
                        pagina += 1;
                    }
                    Some("hdnIdProtocolo") if valor(tag, "value")?.as_deref() == Some(id) => {
                        protocolo += 1;
                    }
                    _ => return Err(E::TelaForaDaForma),
                }
            }
            "button" => {
                if presente(tag, "disabled")?
                    || presente(tag, "formaction")?
                    || presente(tag, "formmethod")?
                    || presente(tag, "formenctype")?
                {
                    return Err(E::TelaForaDaForma);
                }
                match (
                    valor(tag, "type")?.as_deref(),
                    valor(tag, "name")?.as_deref(),
                ) {
                    (Some("submit"), Some("sbmSalvar"))
                        if valor(tag, "value")?.as_deref() == Some("Salvar") =>
                    {
                        botao += 1;
                    }
                    (Some("button"), Some("btnCancelar")) => {}
                    _ => return Err(E::TelaForaDaForma),
                }
            }
            "select" => {
                if valor(tag, "id")?.as_deref() != Some(SELECAO)
                    || valor(tag, "name")?.as_deref() != Some(SELECAO)
                    || presente(tag, "multiple")?
                    || presente(tag, "disabled")?
                {
                    return Err(E::TelaForaDaForma);
                }
                selecao += 1;
                let (lidos, nulos) = ler_opcoes(tag, parser)?;
                usuarios.extend(lidos);
                item_nulo += nulos;
            }
            "optgroup" | "textarea" => return Err(E::TelaForaDaForma),
            _ => {}
        }
    }
    if pagina != 1 || protocolo != 1 || botao != 1 || selecao != 1 || item_nulo != 1 {
        return Err(E::TelaForaDaForma);
    }
    Ok(TelaDaAtribuicao { usuarios })
}

/// Valor exato da opção: nenhum POST é feito aqui.
/// atual=None significa não observado, e não ausência de atribuição.
pub(crate) fn planejar(
    tela: &TelaDaAtribuicao,
    login: Option<&str>,
    atual: Option<&str>,
) -> Result<String, E> {
    let atual = atual.ok_or(E::SituacaoNaoObservada)?;
    let Some(login) = login else {
        if atual.is_empty() {
            return Err(E::AtribuicaoJaAplicada);
        }
        return Ok(SEM_ATRIBUICAO.to_owned());
    };
    if login.is_empty() || login.trim() != login {
        return Err(E::UsuarioNaoAtribuivel);
    }
    let mut achados = tela.usuarios.iter().filter(|u| u.login == login);
    let usuario = achados.next().ok_or(E::UsuarioNaoAtribuivel)?;
    if achados.next().is_some() {
        return Err(E::UsuarioAmbiguo);
    }
    if atual == usuario.login {
        return Err(E::AtribuicaoJaAplicada);
    }
    Ok(usuario.id.clone())
}

#[cfg(test)]
#[path = "processo_atribuir/tests.rs"]
mod tests;
