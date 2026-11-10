//! Leitor da tela de atribuição por HTML servido, sem executar JS.
// Evidência: UnB 4.0.9, GO e Todos tests/test_ciclo_atribuir_010.py @323eb01.
//! O select não informa a atribuição atual: a árvore do processo é a fonte.
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    decodificar_atributo, derivar_submissao, localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::texto_de_recusa::decodificar_entidades;

const FORMULARIO: &str = "frmAtividadeAtribuir";
const ACAO: &str = "procedimento_atribuicao_cadastrar";
const SELECAO: &str = "selAtribuicao";
const SEM_ATRIBUICAO: &str = "null";

/// O POST da atribuição é derivado da tela, não fabricado pelo comando.
/// A própria operação só poderá enviá-lo depois de garantir a releitura.
pub(crate) const ATRIBUIR_PROCESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: ACAO,
    formulario: FORMULARIO,
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmSalvar"),
    campos_ocultos_exigidos: &["hdnInfraTipoPagina", "hdnIdProtocolo"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &[SELECAO],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AtribuicaoNaUnidade,
    },
};

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

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroAtribuir {
    UsuarioNaoAtribuivel,
    UsuarioAmbiguo,
    SituacaoNaoObservada,
    AtribuicaoJaAplicada,
    CharsetDaTelaNaoSuportado,
    #[causa(transparente)]
    Formulario(ErroSubmissaoFormulario),
    #[causa(transparente)]
    Html(ErroHtml),
}
impl ErroAtribuir {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }
}

type E = ErroAtribuir;
fn valor(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    valor_do_atributo(tag, nome)
        .map_err(|e| ErroHtml::atributo("tela_da_atribuicao_fora_da_forma", e))?
        .map(|bruto| {
            decodificar_atributo(&bruto)
                .ok_or(E::Html(ErroHtml::valor("tela_da_atribuicao_fora_da_forma")))
        })
        .transpose()
}
fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, E> {
    atributo_presente(tag, nome)
        .map_err(|e| E::Html(ErroHtml::atributo("tela_da_atribuicao_fora_da_forma", e)))
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
    let mut usuarios: Vec<UsuarioDaAtribuicao> = Vec::new();
    let mut item_nulo = 0;
    for filho in tag.children().all(parser) {
        let Some(opcao) = filho.as_tag() else {
            continue;
        };
        if opcao.name().as_utf8_str().as_ref() != "option" {
            return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
        }
        if presente(opcao, "disabled")? {
            return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
        }
        let id_usuario =
            valor(opcao, "value")?.ok_or(ErroHtml::valor("tela_da_atribuicao_fora_da_forma"))?;
        if id_usuario == SEM_ATRIBUICAO {
            if !presente(opcao, "selected")? {
                return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
            }
            item_nulo += 1;
            continue;
        }
        if presente(opcao, "selected")?
            || id_usuario.is_empty()
            || !id_usuario.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
        }
        let rotulo = decodificar_entidades(&opcao.inner_text(parser))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let (login, nome) = rotulo
            .split_once(" - ")
            .ok_or(ErroHtml::valor("tela_da_atribuicao_fora_da_forma"))?;
        if login.is_empty() || nome.is_empty() {
            return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
        }
        if usuarios.iter().any(|usuario| usuario.id == id_usuario) {
            return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
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
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::valor("tela_da_atribuicao_fora_da_forma"))?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, FORMULARIO)
        .map_err(|_| ErroHtml::valor("tela_da_atribuicao_fora_da_forma"))?;
    if valor(form, "method")?.as_deref() != Some("post")
        || presente(form, "disabled")?
        || presente(form, "hidden")?
        || !valor(form, "action")?
            .as_deref()
            .is_some_and(|action| acao_do_formulario(action, id))
    {
        return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
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
                    return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
                }
                match valor(tag, "name")?.as_deref() {
                    Some("hdnInfraTipoPagina") if valor(tag, "value")?.as_deref() == Some("2") => {
                        pagina += 1;
                    }
                    Some("hdnIdProtocolo") if valor(tag, "value")?.as_deref() == Some(id) => {
                        protocolo += 1;
                    }
                    _ => return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into()),
                }
            }
            "button" => {
                if presente(tag, "disabled")?
                    || presente(tag, "hidden")?
                    || presente(tag, "formaction")?
                    || presente(tag, "formmethod")?
                    || presente(tag, "formenctype")?
                {
                    return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
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
                    _ => return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into()),
                }
            }
            "select" => {
                if valor(tag, "id")?.as_deref() != Some(SELECAO)
                    || valor(tag, "name")?.as_deref() != Some(SELECAO)
                    || presente(tag, "multiple")?
                    || presente(tag, "disabled")?
                    || presente(tag, "hidden")?
                {
                    return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
                }
                selecao += 1;
                let (lidos, nulos) = ler_opcoes(tag, parser)?;
                usuarios.extend(lidos);
                item_nulo += nulos;
            }
            "fieldset" if presente(tag, "disabled")? => {
                // Controles dentro do fieldset não estão acionáveis no DOM.
                return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
            }
            "optgroup" | "textarea" => {
                return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
            }
            _ => {}
        }
    }
    if pagina != 1 || protocolo != 1 || botao != 1 || selecao != 1 || item_nulo != 1 {
        return Err(ErroHtml::valor("tela_da_atribuicao_fora_da_forma").into());
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
    if login.is_empty() {
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

/// A prévia já prova o payload do futuro POST sem enviar nada.
/// A seleção é feita uma vez sobre a mesma tela e mesma identidade.
#[allow(clippy::too_many_arguments)]
pub(crate) fn preparar_submissao(
    instalacao: &Instalacao,
    url: &str,
    html: &str,
    mime: &str,
    id: &str,
    unidade: &str,
    login: Option<&str>,
    atual: Option<&str>,
) -> Result<(String, SubmissaoDerivada), E> {
    let tela = ler_tela(html, id)?;
    let valor = planejar(&tela, login, atual)?;
    let codificacao = codificacao_da_tela(mime).ok_or(E::CharsetDaTelaNaoSuportado)?;
    let servida = TelaServida {
        url,
        html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &ATRIBUIR_PROCESSO,
        instalacao,
        &servida,
        unidade,
        &[(SELECAO, valor.as_str())],
    )
    .map_err(E::Formulario)?;
    Ok((valor, submissao))
}

mod fluxo;
pub(crate) use fluxo::{
    ErroProcessoAtribuir, EstadoDaAtribuicao, PedidoDaAtribuicao, ProcessoAtribuido,
    executar_previa,
};

#[cfg(test)]
#[path = "processo_atribuir/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "processo_atribuir/tests_fluxo.rs"]
mod tests_fluxo;

#[cfg(test)]
#[path = "processo_atribuir/acervo.rs"]
mod acervo;
