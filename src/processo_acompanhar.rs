//! Núcleo verificado de `ies processo acompanhar` (#254).
//! A leitura/seleção não toca rede; a UI não infere grupo nem sucesso de escrita
//! pela resposta HTTP. Casos do Todos `tests/test_processo_acompanhar_010.py` @323eb01.

use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador,
};

mod exclusao;
mod fluxo;
mod submissao;
pub(crate) use fluxo::{Pedido, Previa, previa};

const FORMULARIO: &str = "frmAcompanhamentoCadastro";
const GRUPO: &str = "selGrupoAcompanhamento";
const PROTOCOLO: &str = "hdnIdProtocolo";
const TABELA: &str = "Tabela de Acompanhamentos.";

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAcompanhar {
    Abertura(#[from] crate::processo_obter::ErroProcessoObter),
    OrigemNaoAutorizada,
    UnidadeNaoPublicada,
    LinkNaoPublicado,
    Barra(#[from] crate::arvore_processo::ErroAcompanharNaBarra),
    Transporte(#[from] crate::transporte::FalhaTransporte),
    Formulario(#[from] crate::formulario_web::ErroSubmissaoFormulario),
    PedidoDeAcompanhamentoInvalido,
    RedirecionamentoDeAcompanhamentoInesperado,
    TelaDeAcompanhamentoInesperada,
    CadastroNaoPublicado,
    CadastroDeOutroProcesso,
    /// A action do cadastro não se lê em pares `chave=valor`.
    ConsultaDoCadastroIlegivel(#[source] ErroConsultaDoControlador),
    /// A action do cadastro não tem exatamente um `acao` nem um `id_procedimento`.
    ChaveDoCadastroIlegivel(#[source] ErroChaveDaConsulta),
    GruposNaoPublicados,
    GrupoNaoOferecido,
    GrupoAmbiguo,
    ListaNaoPublicada,
    GrupoJaAcompanhado,
    NenhumAcompanhamento,
    EscolhaDeAcompanhamentoAmbigua,
    AcompanhamentoNaoConfirmado,
    /// GET/decodificação falhou após POST único; efeito permanece incerto.
    ReleituraDaRetiradaFalhou(#[source] crate::transporte::FalhaTransporte),
    ExclusaoNaoOferecida,
    ColunasDeAcompanhamentosInvalidas,
    FormulariosDeAcompanhamentoRepetidos,
    HtmlDeAcompanhamentoInvalido,
    IdDeAcompanhamentoRepetido,
    LinhaDeAcompanhamentoInvalida,
    ListasDeAcompanhamentosRepetidas,
    OpcoesDeGrupoDeAcompanhamentoInvalidas,
    /// O endereço do cadastro de acompanhamento foi recusado.
    EnderecoDoCadastroRecusado(crate::arvore_processo::endereco::EnderecoRecusado),
    /// O endereço de exclusão observado foi recusado.
    EnderecoDaExclusaoRecusado(crate::arvore_processo::endereco::EnderecoRecusado),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GrupoOferecido {
    pub(crate) valor: String,
    pub(crate) nome: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Acompanhamento {
    pub(crate) id: String,
    pub(crate) grupo: String,
    pub(crate) observacao: String,
}

fn texto(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn campo(tag: crate::tela_html::Elemento<'_>, nome: &'static str) -> Option<String> {
    tag.atributo(nome).map(str::to_owned)
}

/// A página de cadastro da `UnB` 4.0.9 tem um form e o select publicado.
/// Confere a identidade do processo no action E no oculto antes de escolher.
pub(crate) fn grupos_do_cadastro(
    html: &crate::tela_html::HtmlServido,
    id_procedimento: &str,
) -> Result<Vec<GrupoOferecido>, ErroAcompanhar> {
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroAcompanhar::HtmlDeAcompanhamentoInvalido);
    }
    let dom = crate::tela_html::arvore(html);
    let mut forms = Vec::new();
    for no in dom.elementos("form") {
        if campo(no, "id").as_deref() == Some(FORMULARIO) {
            forms.push(no);
        }
    }
    let form = match forms.as_slice() {
        [] => return Err(ErroAcompanhar::CadastroNaoPublicado),
        [form] => *form,
        _ => return Err(ErroAcompanhar::FormulariosDeAcompanhamentoRepetidos),
    };
    let action = campo(form, "action").ok_or(ErroAcompanhar::CadastroNaoPublicado)?;
    conferir_action_do_cadastro(&action, id_procedimento)?;
    let mut protocolos = Vec::new();
    let mut selects = Vec::new();
    for tag in form.descendentes() {
        let nome_tag = tag.nome();
        if nome_tag == "input" && campo(tag, "name").as_deref() == Some(PROTOCOLO) {
            protocolos.push(campo(tag, "value").unwrap_or_default());
        }
        if nome_tag == "select" && campo(tag, "name").as_deref() == Some(GRUPO) {
            selects.push(tag);
        }
    }
    if protocolos != [id_procedimento] {
        return Err(ErroAcompanhar::CadastroDeOutroProcesso);
    }
    let select = match selects.as_slice() {
        [select] => *select,
        _ => return Err(ErroAcompanhar::GruposNaoPublicados),
    };
    let mut grupos = Vec::new();
    for tag in select.descendentes() {
        if tag.nome() != "option" {
            continue;
        }
        if tag.presente("disabled") {
            continue;
        }
        let valor =
            campo(tag, "value").ok_or(ErroAcompanhar::OpcoesDeGrupoDeAcompanhamentoInvalidas)?;
        let nome = texto(&tag.texto());
        if nome.is_empty() && valor.is_empty() {
            // Uma opção em branco pode ter valor vazio; não é ambiguidade de grupo.
            grupos.push(GrupoOferecido { valor, nome });
            continue;
        }
        if valor.is_empty() {
            return Err(ErroAcompanhar::OpcoesDeGrupoDeAcompanhamentoInvalidas);
        }
        grupos.push(GrupoOferecido { valor, nome });
    }
    if grupos.is_empty() {
        return Err(ErroAcompanhar::GruposNaoPublicados);
    }
    Ok(grupos)
}

/// A action do cadastro é este processo e esta ação, cada um uma vez na consulta.
fn conferir_action_do_cadastro(action: &str, id_procedimento: &str) -> Result<(), ErroAcompanhar> {
    let Some((_, consulta)) = action.split_once('?') else {
        return Err(ErroAcompanhar::CadastroDeOutroProcesso);
    };
    let consulta =
        ConsultaDoControlador::ler(consulta).map_err(ErroAcompanhar::ConsultaDoCadastroIlegivel)?;
    let acao = consulta
        .unico("acao")
        .map_err(ErroAcompanhar::ChaveDoCadastroIlegivel)?;
    let id = consulta
        .unico("id_procedimento")
        .map_err(ErroAcompanhar::ChaveDoCadastroIlegivel)?;
    if AcaoDoSei::ler(acao) != Some(AcaoDoSei::AcompanhamentoCadastrar) || id != id_procedimento {
        return Err(ErroAcompanhar::CadastroDeOutroProcesso);
    }
    Ok(())
}

/// O nome da opção "sem grupo" do `selGrupoAcompanhamento` do cadastro: o
/// SEI a publica como `<option value="null" selected="selected">&nbsp;</option>`
/// (cassettes `processo-acompanhar-*`), e o texto colapsado fica vazio.
pub(crate) const SEM_GRUPO: &str = "";

/// Casa APENAS pelo nome exato. Sem grupo pedido, seleciona a opção
/// [`SEM_GRUPO`] publicada.
pub(crate) fn escolher_grupo<'a>(
    grupos: &'a [GrupoOferecido],
    pedido: Option<&str>,
) -> Result<&'a GrupoOferecido, ErroAcompanhar> {
    let nome = pedido.unwrap_or(SEM_GRUPO);
    let achados: Vec<_> = grupos.iter().filter(|g| g.nome == nome).collect();
    match achados.as_slice() {
        [] => Err(ErroAcompanhar::GrupoNaoOferecido),
        [grupo] => Ok(grupo),
        _ => Err(ErroAcompanhar::GrupoAmbiguo),
    }
}

/// Uma lista vazia SÓ é comprovada por tabela com cabeçalho, nunca por HTML
/// cortado, tela de login ou ausência de tabela.
pub(crate) fn acompanhamentos_da_lista(
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<Acompanhamento>, ErroAcompanhar> {
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroAcompanhar::HtmlDeAcompanhamentoInvalido);
    }
    let dom = crate::tela_html::arvore(html);
    let mut tabelas = Vec::new();
    for no in dom.elementos("table") {
        if campo(no, "summary").as_deref() == Some(TABELA) {
            tabelas.push(no);
        }
    }
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroAcompanhar::ListaNaoPublicada),
        [tabela] => *tabela,
        _ => return Err(ErroAcompanhar::ListasDeAcompanhamentosRepetidas),
    };
    let mut cabecalho = false;
    let mut linhas = Vec::new();
    for tr in tabela.descendentes() {
        if tr.nome() != "tr" {
            continue;
        }
        let mut th = Vec::new();
        let mut td = Vec::new();
        for tag in tr.descendentes() {
            if tag.nome() == "th" {
                th.push(texto(&tag.texto()));
            }
            if tag.nome() == "td" {
                td.push(tag);
            }
        }
        if !th.is_empty() {
            if cabecalho
                || !th.iter().any(|h| h == "Grupo")
                || !th.iter().any(|h| h == "Observação")
            {
                return Err(ErroAcompanhar::ColunasDeAcompanhamentosInvalidas);
            }
            cabecalho = true;
            continue;
        }
        if td.is_empty() {
            continue;
        }
        let [selecao, grupo, observacao, _, _, _] = td.as_slice() else {
            return Err(ErroAcompanhar::LinhaDeAcompanhamentoInvalida);
        };
        let mut ids = Vec::new();
        for tag in selecao.descendentes() {
            if tag.nome() != "input" {
                continue;
            }
            if campo(tag, "type").as_deref() == Some("checkbox") {
                ids.push(campo(tag, "value").unwrap_or_default());
            }
        }
        let id = match ids.as_slice() {
            [id] if !id.is_empty() => id.clone(),
            _ => return Err(ErroAcompanhar::LinhaDeAcompanhamentoInvalida),
        };
        if linhas.iter().any(|l: &Acompanhamento| l.id == id) {
            return Err(ErroAcompanhar::IdDeAcompanhamentoRepetido);
        }
        linhas.push(Acompanhamento {
            id,
            grupo: texto(&grupo.texto()),
            observacao: texto(&observacao.texto()),
        });
    }
    if !cabecalho {
        return Err(ErroAcompanhar::ColunasDeAcompanhamentosInvalidas);
    }
    Ok(linhas)
}

/// Se já existe este grupo na mesma unidade, não escrever nem quando a
/// observação é diferente: o SEI recusa duplicatas de grupo.
pub(crate) fn conferir_novo_grupo(
    antes: &[Acompanhamento],
    grupo: &str,
) -> Result<(), ErroAcompanhar> {
    if antes.iter().any(|a| a.grupo == grupo) {
        Err(ErroAcompanhar::GrupoJaAcompanhado)
    } else {
        Ok(())
    }
}

/// Escolha de alvo da remoção; sem grupo só quando há exatamente uma linha.
pub(crate) fn alvo_para_deixar<'a>(
    antes: &'a [Acompanhamento],
    grupo: Option<&str>,
) -> Result<&'a Acompanhamento, ErroAcompanhar> {
    let candidatos: Vec<_> = antes
        .iter()
        .filter(|a| grupo.is_none_or(|g| a.grupo == g))
        .collect();
    match candidatos.as_slice() {
        [] => Err(ErroAcompanhar::NenhumAcompanhamento),
        [a] => Ok(a),
        _ => Err(ErroAcompanhar::EscolhaDeAcompanhamentoAmbigua),
    }
}

/// A operação só está confirmada se apareceu exatamente UMA linha nova
/// com grupo/observação pedidos e TODAS as linhas anteriores intactas.
pub(crate) fn conferir_inclusao(
    antes: &[Acompanhamento],
    depois: &[Acompanhamento],
    grupo: &str,
    observacao: &str,
) -> Result<(), ErroAcompanhar> {
    if depois.len() != antes.len() + 1
        || !antes.iter().all(|a| depois.contains(a))
        || depois
            .iter()
            .filter(|a| !antes.iter().any(|b| b.id == a.id))
            .filter(|a| a.grupo == grupo && a.observacao == observacao)
            .count()
            != 1
    {
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
    }
    Ok(())
}

/// Só remova quando a identidade exata desaparecer e todo o resto for idêntico.
pub(crate) fn conferir_remocao(
    antes: &[Acompanhamento],
    depois: &[Acompanhamento],
    id: &str,
) -> Result<(), ErroAcompanhar> {
    if antes.len() != depois.len() + 1
        || !antes.iter().any(|a| a.id == id)
        || depois.iter().any(|a| a.id == id)
        || !antes
            .iter()
            .filter(|a| a.id != id)
            .all(|a| depois.contains(a))
    {
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
    }
    Ok(())
}

#[cfg(test)]
#[path = "processo_acompanhar_tests.rs"]
mod tests;
