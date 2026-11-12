//! Leitor da tela de atribuição por HTML servido, sem executar JS.
// Evidência: UnB 4.0.9, GO e Todos tests/test_ciclo_atribuir_010.py @323eb01.
//! O select não informa a atribuição atual: a árvore do processo é a fonte.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_submissao, localizar_formulario,
};
use crate::instalacao::Instalacao;

const FORMULARIO: &str = "frmAtividadeAtribuir";
const SELECAO: &str = "selAtribuicao";
const SEM_ATRIBUICAO: &str = "null";

/// O POST da atribuição é derivado da tela, não fabricado pelo comando.
/// A própria operação só poderá enviá-lo depois de garantir a releitura.
pub(crate) const ATRIBUIR_PROCESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoAtribuicaoCadastrar,
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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAtribuir {
    UsuarioNaoAtribuivel,
    UsuarioAmbiguo,
    AtribuicaoJaAplicada,
    CharsetDaTelaNaoSuportado,
    Formulario(#[from] ErroSubmissaoFormulario),
    TelaDaAtribuicaoForaDaForma,
    /// O formulário de atribuição não foi localizado na tela.
    FormularioDaAtribuicaoNaoLocalizado(#[source] crate::formulario_web::ErroSubmissaoFormulario),
}

/// Confere a URL assinada servida sem a publicar nem fabricar outra.
fn acao_do_formulario(action: &str, id: &str) -> bool {
    let Some(consulta) = action.strip_prefix("controlador.php?") else {
        return false;
    };
    // Domínio: `#` e `?` nunca aparecem na URL assinada, em nenhum parâmetro.
    if consulta.contains(['#', '?']) {
        return false;
    }
    ConsultaDoControlador::ler(consulta).is_ok_and(|consulta| {
        consulta.acao() == Ok(AcaoDoSei::ProcedimentoAtribuicaoCadastrar)
            && consulta.unico("id_procedimento") == Ok(id)
            && consulta
                .unico("infra_hash")
                .is_ok_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
    })
}

/// Opções observadas do select, sem executar scripts.
fn ler_opcoes(
    tag: crate::tela_html::Elemento<'_>,
) -> Result<(Vec<UsuarioDaAtribuicao>, usize), ErroAtribuir> {
    let mut usuarios: Vec<UsuarioDaAtribuicao> = Vec::new();
    let mut item_nulo = 0;
    for opcao in tag.descendentes() {
        if opcao.nome() != "option" {
            return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
        }
        if opcao.presente("disabled") {
            return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
        }
        let id_usuario = opcao
            .atributo("value")
            .map(str::to_owned)
            .ok_or(ErroAtribuir::TelaDaAtribuicaoForaDaForma)?;
        if id_usuario == SEM_ATRIBUICAO {
            if !opcao.presente("selected") {
                return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
            }
            item_nulo += 1;
            continue;
        }
        if opcao.presente("selected")
            || id_usuario.is_empty()
            || !id_usuario.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
        }
        let rotulo = opcao
            .texto()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let (login, nome) = rotulo
            .split_once(" - ")
            .ok_or(ErroAtribuir::TelaDaAtribuicaoForaDaForma)?;
        if login.is_empty() || nome.is_empty() {
            return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
        }
        if usuarios.iter().any(|usuario| usuario.id == id_usuario) {
            return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
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
pub(crate) fn ler_tela(
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<TelaDaAtribuicao, ErroAtribuir> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, FORMULARIO)
        .map_err(ErroAtribuir::FormularioDaAtribuicaoNaoLocalizado)?;
    if form.atributo("method") != Some("post")
        || form.presente("disabled")
        || form.presente("hidden")
        || !form
            .atributo("action")
            .is_some_and(|action| acao_do_formulario(action, id))
    {
        return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
    }

    let (mut pagina, mut protocolo, mut botao, mut selecao) = (0, 0, 0, 0);
    let mut usuarios = Vec::new();
    let mut item_nulo = 0;
    for tag in form.descendentes() {
        let elemento = tag.nome().to_ascii_lowercase();
        match elemento.as_str() {
            "input" => {
                if tag.presente("disabled") || tag.atributo("type") != Some("hidden") {
                    return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
                }
                match tag.atributo("name") {
                    Some("hdnInfraTipoPagina") if tag.atributo("value") == Some("2") => {
                        pagina += 1;
                    }
                    Some("hdnIdProtocolo") if tag.atributo("value") == Some(id) => {
                        protocolo += 1;
                    }
                    _ => return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma),
                }
            }
            "button" => {
                if tag.presente("disabled")
                    || tag.presente("hidden")
                    || tag.presente("formaction")
                    || tag.presente("formmethod")
                    || tag.presente("formenctype")
                {
                    return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
                }
                match (tag.atributo("type"), tag.atributo("name")) {
                    (Some("submit"), Some("sbmSalvar"))
                        if tag.atributo("value") == Some("Salvar") =>
                    {
                        botao += 1;
                    }
                    (Some("button"), Some("btnCancelar")) => {}
                    _ => return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma),
                }
            }
            "select" => {
                if tag.atributo("id") != Some(SELECAO)
                    || tag.atributo("name") != Some(SELECAO)
                    || tag.presente("multiple")
                    || tag.presente("disabled")
                    || tag.presente("hidden")
                {
                    return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
                }
                selecao += 1;
                let (lidos, nulos) = ler_opcoes(tag)?;
                usuarios.extend(lidos);
                item_nulo += nulos;
            }
            "fieldset" if tag.presente("disabled") => {
                // Controles dentro do fieldset não estão acionáveis no DOM.
                return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
            }
            "optgroup" | "textarea" => {
                return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
            }
            _ => {}
        }
    }
    if pagina != 1 || protocolo != 1 || botao != 1 || selecao != 1 || item_nulo != 1 {
        return Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma);
    }
    Ok(TelaDaAtribuicao { usuarios })
}

/// Quem a unidade tem atribuído na árvore. `Ninguem` é fato publicado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Atribuicao<'a> {
    Ninguem,
    Usuario(&'a str),
}

impl<'a> Atribuicao<'a> {
    /// O login atribuído, ou `None` quando a unidade não tem atribuição.
    pub(crate) fn login(self) -> Option<&'a str> {
        match self {
            Self::Ninguem => None,
            Self::Usuario(login) => Some(login),
        }
    }
}

/// Valor exato da opção: nenhum POST é feito aqui.
/// `atual` é a situação já lida da árvore; quem a lê recusa antes de chegar aqui.
pub(crate) fn planejar(
    tela: &TelaDaAtribuicao,
    login: Option<&str>,
    atual: Atribuicao<'_>,
) -> Result<String, ErroAtribuir> {
    let Some(login) = login else {
        if atual == Atribuicao::Ninguem {
            return Err(ErroAtribuir::AtribuicaoJaAplicada);
        }
        return Ok(SEM_ATRIBUICAO.to_owned());
    };
    if login.is_empty() {
        return Err(ErroAtribuir::UsuarioNaoAtribuivel);
    }
    let mut achados = tela.usuarios.iter().filter(|u| u.login == login);
    let usuario = achados.next().ok_or(ErroAtribuir::UsuarioNaoAtribuivel)?;
    if achados.next().is_some() {
        return Err(ErroAtribuir::UsuarioAmbiguo);
    }
    if atual == Atribuicao::Usuario(usuario.login.as_str()) {
        return Err(ErroAtribuir::AtribuicaoJaAplicada);
    }
    Ok(usuario.id.clone())
}

/// A prévia já prova o payload do futuro POST sem enviar nada.
/// A seleção é feita uma vez sobre a mesma tela e mesma identidade.
pub(crate) fn preparar_submissao(
    instalacao: &Instalacao,
    servida: &TelaServida<'_>,
    id: &str,
    unidade: &str,
    login: Option<&str>,
    atual: Atribuicao<'_>,
) -> Result<(String, SubmissaoDerivada), ErroAtribuir> {
    let tela = ler_tela(servida.html, id)?;
    let valor = planejar(&tela, login, atual)?;
    let submissao = derivar_submissao(
        &ATRIBUIR_PROCESSO,
        instalacao,
        servida,
        Some(unidade),
        &[(SELECAO, valor.as_str())],
    )
    .map_err(ErroAtribuir::Formulario)?;
    Ok((valor, submissao))
}

mod fluxo;
pub(crate) use fluxo::{
    EstadoDaAtribuicao, PedidoDaAtribuicao, ProcessoAtribuido, executar_previa,
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
