//! Núcleo verificado de `ies processo acompanhar` (#254).
//! A leitura/seleção não toca rede; a UI não infere grupo nem sucesso de escrita
//! pela resposta HTTP. Casos do Todos tests/test_processo_acompanhar_010.py @323eb01.

use crate::entidades_html::decodificar_texto;
use crate::html_atributos::{atributo_presente, valor_do_atributo};

const FORMULARIO: &str = "frmAcompanhamentoCadastro";
const GRUPO: &str = "selGrupoAcompanhamento";
const PROTOCOLO: &str = "hdnIdProtocolo";
const TABELA: &str = "Tabela de Acompanhamentos.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAcompanhar {
    HtmlInvalido,
    CadastroNaoPublicado,
    FormularioDuplicado,
    CadastroDeOutroProcesso,
    GruposNaoPublicados,
    OpcoesDeGrupoInvalidas,
    GrupoNaoOferecido,
    GrupoAmbiguo,
    ListaNaoPublicada,
    ListaDuplicada,
    ColunasDaListaInvalidas,
    LinhaDaListaInvalida,
    IdDeAcompanhamentoRepetido,
    GrupoJaAcompanhado,
    NenhumAcompanhamento,
    EscolhaDeAcompanhamentoAmbigua,
    AcompanhamentoNaoConfirmado,
}

impl ErroAcompanhar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlInvalido => "html_de_acompanhamento_invalido",
            Self::CadastroNaoPublicado => "cadastro_de_acompanhamento_nao_publicado",
            Self::FormularioDuplicado => "formularios_de_acompanhamento_repetidos",
            Self::CadastroDeOutroProcesso => "cadastro_de_acompanhamento_de_outro_processo",
            Self::GruposNaoPublicados => "grupos_de_acompanhamento_nao_publicados",
            Self::OpcoesDeGrupoInvalidas => "opcoes_de_grupo_de_acompanhamento_invalidas",
            Self::GrupoNaoOferecido => "grupo_de_acompanhamento_nao_oferecido",
            Self::GrupoAmbiguo => "grupo_de_acompanhamento_ambiguo",
            Self::ListaNaoPublicada => "lista_de_acompanhamentos_nao_publicada",
            Self::ListaDuplicada => "listas_de_acompanhamentos_repetidas",
            Self::ColunasDaListaInvalidas => "colunas_de_acompanhamentos_invalidas",
            Self::LinhaDaListaInvalida => "linha_de_acompanhamento_invalida",
            Self::IdDeAcompanhamentoRepetido => "id_de_acompanhamento_repetido",
            Self::GrupoJaAcompanhado => "grupo_ja_acompanhado_no_processo",
            Self::NenhumAcompanhamento => "processo_sem_acompanhamento",
            Self::EscolhaDeAcompanhamentoAmbigua => "acompanhamento_para_remover_ambiguo",
            Self::AcompanhamentoNaoConfirmado => "acompanhamento_nao_confirmado_por_leitura",
        }
    }
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
    decodificar_texto(texto)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn campo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroAcompanhar> {
    valor_do_atributo(tag, nome).map_err(|_| ErroAcompanhar::HtmlInvalido)
}

/// A página de cadastro da UnB 4.0.9 tem um form e o select publicado.
/// Confere a identidade do processo no action E no oculto antes de escolher.
pub(crate) fn grupos_do_cadastro(
    html: &str,
    id_procedimento: &str,
) -> Result<Vec<GrupoOferecido>, ErroAcompanhar> {
    use ErroAcompanhar as E;
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(E::HtmlInvalido);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::HtmlInvalido)?;
    let parser = dom.parser();
    let mut forms = Vec::new();
    for no in dom.query_selector("form").into_iter().flatten() {
        let tag = no.get(parser).and_then(tl::Node::as_tag).ok_or(E::HtmlInvalido)?;
        if campo(tag, "id")?.as_deref() == Some(FORMULARIO) {
            forms.push(tag);
        }
    }
    let form = match forms.as_slice() {
        [] => return Err(E::CadastroNaoPublicado),
        [form] => *form,
        _ => return Err(E::FormularioDuplicado),
    };
    let action = campo(form, "action")?.ok_or(E::CadastroNaoPublicado)?;
    let action = crate::formulario_web::decodificar_atributo(&action).ok_or(E::HtmlInvalido)?;
    if !parametro_unico(&action, "acao", "acompanhamento_cadastrar")
        || !parametro_unico(&action, "id_procedimento", id_procedimento)
    {
        return Err(E::CadastroDeOutroProcesso);
    }
    let mut protocolos = Vec::new();
    let mut selects = Vec::new();
    for no in form.children().all(parser) {
        let Some(tag) = no.as_tag() else { continue };
        let nome_tag = tag.name().as_utf8_str();
        if nome_tag == "input" && campo(tag, "name")?.as_deref() == Some(PROTOCOLO) {
            protocolos.push(campo(tag, "value")?.unwrap_or_default());
        }
        if nome_tag == "select" && campo(tag, "name")?.as_deref() == Some(GRUPO) {
            selects.push(tag);
        }
    }
    if protocolos != [id_procedimento] {
        return Err(E::CadastroDeOutroProcesso);
    }
    let select = match selects.as_slice() {
        [select] => *select,
        _ => return Err(E::GruposNaoPublicados),
    };
    let mut grupos = Vec::new();
    for no in select.children().all(parser) {
        let Some(tag) = no.as_tag() else { continue };
        if tag.name() != "option" {
            continue;
        }
        if atributo_presente(tag, "disabled").map_err(|_| E::HtmlInvalido)? {
            continue;
        }
        let valor = campo(tag, "value")?.ok_or(E::OpcoesDeGrupoInvalidas)?;
        let nome = texto(&tag.inner_text(parser));
        if nome.is_empty() && valor.trim().is_empty() {
            // Uma opção em branco pode ter valor vazio; não é ambiguidade de grupo.
            grupos.push(GrupoOferecido { valor, nome });
            continue;
        }
        if valor.is_empty() || valor.chars().any(char::is_control) {
            return Err(E::OpcoesDeGrupoInvalidas);
        }
        grupos.push(GrupoOferecido { valor, nome });
    }
    if grupos.is_empty() {
        return Err(E::GruposNaoPublicados);
    }
    Ok(grupos)
}

fn parametro_unico(url: &str, nome: &str, valor: &str) -> bool {
    url.split_once('?')
        .is_some_and(|(_, query)| query.split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(chave, _)| *chave == nome)
            .map(|(_, v)| v)
            .collect::<Vec<_>>() == [valor])
}

/// Casa APENAS pelo nome exato. O branco seleciona a opção vazia publicada.
pub(crate) fn escolher_grupo<'a>(
    grupos: &'a [GrupoOferecido],
    pedido: Option<&str>,
) -> Result<&'a GrupoOferecido, ErroAcompanhar> {
    let nome = pedido.unwrap_or("");
    let achados: Vec<_> = grupos.iter().filter(|g| g.nome == nome).collect();
    match achados.as_slice() {
        [] => Err(ErroAcompanhar::GrupoNaoOferecido),
        [grupo] => Ok(grupo),
        _ => Err(ErroAcompanhar::GrupoAmbiguo),
    }
}

/// Uma lista vazia SÓ é comprovada por tabela com cabeçalho, nunca por HTML
/// cortado, tela de login ou ausência de tabela.
pub(crate) fn acompanhamentos_da_lista(html: &str) -> Result<Vec<Acompanhamento>, ErroAcompanhar> {
    use ErroAcompanhar as E;
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(E::HtmlInvalido);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::HtmlInvalido)?;
    let parser = dom.parser();
    let mut tabelas = Vec::new();
    for no in dom.query_selector("table").into_iter().flatten() {
        let tag = no.get(parser).and_then(tl::Node::as_tag).ok_or(E::HtmlInvalido)?;
        if campo(tag, "summary")?.as_deref() == Some(TABELA) {
            tabelas.push(tag);
        }
    }
    let tabela = match tabelas.as_slice() {
        [] => return Err(E::ListaNaoPublicada),
        [tabela] => *tabela,
        _ => return Err(E::ListaDuplicada),
    };
    let mut cabecalho = false;
    let mut linhas = Vec::new();
    for no in tabela.children().all(parser) {
        let Some(tr) = no.as_tag() else { continue };
        if tr.name() != "tr" { continue; }
        let mut th = Vec::new();
        let mut td = Vec::new();
        for celula in tr.children().all(parser) {
            let Some(tag) = celula.as_tag() else { continue };
            if tag.name() == "th" { th.push(texto(&tag.inner_text(parser))); }
            if tag.name() == "td" { td.push(tag); }
        }
        if !th.is_empty() {
            if cabecalho || !th.iter().any(|h| h == "Grupo")
                || !th.iter().any(|h| h == "Observação")
            {
                return Err(E::ColunasDaListaInvalidas);
            }
            cabecalho = true;
            continue;
        }
        if td.is_empty() { continue; }
        if td.len() != 6 { return Err(E::LinhaDaListaInvalida); }
        let mut ids = Vec::new();
        for item in td[0].children().all(parser) {
            let Some(tag) = item.as_tag() else {continue};
            if tag.name() != "input" {continue;}
            if campo(tag, "type")?.as_deref() == Some("checkbox") {
                ids.push(campo(tag, "value")?.unwrap_or_default());
            }
        }
        let id = match ids.as_slice() {
            [id] if !id.is_empty() && !id.chars().any(char::is_control) => id.clone(),
            _ => return Err(E::LinhaDaListaInvalida),
        };
        if linhas.iter().any(|l: &Acompanhamento| l.id == id) {
            return Err(E::IdDeAcompanhamentoRepetido);
        }
        linhas.push(Acompanhamento {
            id, grupo: texto(&td[1].inner_text(parser)),
            observacao: texto(&td[2].inner_text(parser)),
        });
    }
    if !cabecalho { return Err(E::ColunasDaListaInvalidas); }
    Ok(linhas)
}

/// Se já existe este grupo na mesma unidade, não escrever nem quando a
/// observação é diferente: o SEI recusa duplicatas de grupo.
pub(crate) fn conferir_novo_grupo(
    antes: &[Acompanhamento], grupo: &str
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
    let candidatos: Vec<_> = antes.iter()
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
    antes: &[Acompanhamento], depois: &[Acompanhamento],
    grupo: &str, observacao: &str,
) -> Result<(), ErroAcompanhar> {
    use ErroAcompanhar as E;
    if depois.len() != antes.len() + 1
        || !antes.iter().all(|a| depois.contains(a))
        || depois.iter().filter(|a| !antes.iter().any(|b| b.id == a.id))
            .filter(|a| a.grupo == grupo && a.observacao == observacao).count() != 1
    {
        return Err(E::AcompanhamentoNaoConfirmado);
    }
    Ok(())
}

/// Só remova quando a identidade exata desaparecer e todo o resto for idêntico.
pub(crate) fn conferir_remocao(
    antes: &[Acompanhamento], depois: &[Acompanhamento], id: &str,
) -> Result<(), ErroAcompanhar> {
    if antes.len() != depois.len() + 1
        || !antes.iter().any(|a| a.id == id)
        || depois.iter().any(|a| a.id == id)
        || !antes.iter().filter(|a| a.id != id).all(|a| depois.contains(a))
    {
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "12345";
    fn cadastro(grupos: &str) -> String {
        format!(r#"<form id="frmAcompanhamentoCadastro" method="post"
        action="controlador.php?acao=acompanhamento_cadastrar&amp;id_procedimento=12345&amp;infra_hash=SYNTHETIC">
        <input type="hidden" name="hdnIdProtocolo" value="12345">
        <select name="selGrupoAcompanhamento">{grupos}</select>
        <textarea name="txaObservacao"></textarea>
        <button type="submit" name="sbmCadastrarAcompanhamento">Salvar</button></form>"#)
    }
    fn tabela(linhas: &str) -> String {
        format!(r#"<table summary="Tabela de Acompanhamentos.">
        <tr><th></th><th>Grupo</th><th>Observação</th><th>Usuário</th><th>Data</th><th>Ações</th></tr>{linhas}</table>"#)
    }
    fn linha(id: &str, grupo: &str, obs: &str) -> String {
        format!(r#"<tr><td><input type="checkbox" value="{id}" /></td>
        <td>{grupo}</td><td>{obs}</td><td>aluno</td><td>15/10/2026</td><td>ação</td></tr>"#)
    }
    fn a(id: &str, grupo: &str, obs: &str) -> Acompanhamento {
        Acompanhamento { id: id.into(), grupo: grupo.into(), observacao: obs.into() }
    }
    #[test]
    fn grupos_da_tela_observada_e_nome_exato() {
        // Caso de todos tests/test_processo_acompanhar_010.py @323eb01.
        let html = cadastro(r#"<option value="null">&nbsp;</option>
           <option value="19">acomp fiel</option><option value="64">Boas-Vindas</option>"#);
        let grupos = grupos_do_cadastro(&html, ID).unwrap();
        assert_eq!(escolher_grupo(&grupos, Some("acomp fiel")).unwrap().valor, "19");
        assert_eq!(escolher_grupo(&grupos, None).unwrap().valor, "null");
        assert_eq!(escolher_grupo(&grupos, Some("ACOMP FIEL")), Err(ErroAcompanhar::GrupoNaoOferecido));
    }
    #[test]
    fn recusa_cadastro_de_outro_processo_e_acao_duplicada() {
        let html = cadastro(r#"<option value="19">G</option>"#);
        assert_eq!(grupos_do_cadastro(&html, "999"), Err(ErroAcompanhar::CadastroDeOutroProcesso));
        let duplicado = html.replace("acao=acompanhamento_cadastrar", "acao=acompanhamento_cadastrar&amp;acao=acompanhamento_cadastrar");
        assert_eq!(grupos_do_cadastro(&duplicado, ID), Err(ErroAcompanhar::CadastroDeOutroProcesso));
    }
    #[test]
    fn grupos_ambiguos_e_ausentes_recusados() {
        let html = cadastro(r#"<option value="1">Mesmo</option><option value="2">Mesmo</option>"#);
        let grupos = grupos_do_cadastro(&html, ID).unwrap();
        assert_eq!(escolher_grupo(&grupos, Some("Mesmo")), Err(ErroAcompanhar::GrupoAmbiguo));
        assert_eq!(escolher_grupo(&grupos, None), Err(ErroAcompanhar::GrupoNaoOferecido));
    }
    #[test]
    fn lista_sem_tabela_nao_prova_vazia() {
        // Caso de todos tests/test_processo_acompanhar_010.py @323eb01.
        assert_eq!(acompanhamentos_da_lista("<html>cortada</html>"), Err(ErroAcompanhar::ListaNaoPublicada));
        assert_eq!(acompanhamentos_da_lista(&tabela("")), Ok(vec![]));
    }
    #[test]
    fn lista_identidades_duplicadas_falha_fechado() {
        let html = tabela(&format!("{}{}", linha("831", "Grupo", "O1"), linha("831", "G2", "O2")));
        assert_eq!(acompanhamentos_da_lista(&html), Err(ErroAcompanhar::IdDeAcompanhamentoRepetido));
    }
    #[test]
    fn evita_duplicata_antes_de_enviar() {
        assert_eq!(conferir_novo_grupo(&[a("1", "G", "Antiga")], "G"), Err(ErroAcompanhar::GrupoJaAcompanhado));
        assert!(conferir_novo_grupo(&[a("1", "G", "Antiga")], "Outra").is_ok());
    }
    #[test]
    fn remocao_exige_alvo_sem_ambiguidade() {
        let antes = vec![a("1", "G", "A"), a("2", "H", "B")];
        assert_eq!(alvo_para_deixar(&antes, None), Err(ErroAcompanhar::EscolhaDeAcompanhamentoAmbigua));
        assert_eq!(alvo_para_deixar(&antes, Some("H")).unwrap().id, "2");
    }
    #[test]
    fn reconciliacao_so_aceita_uma_alteracao_precisa() {
        let antes = vec![a("1", "G", "A")];
        let depois = vec![a("1", "G", "A"), a("2", "H", "B")];
        assert!(conferir_inclusao(&antes, &depois, "H", "B").is_ok());
        assert_eq!(conferir_inclusao(&antes, &[a("1","G","A")], "H", "B"),
            Err(ErroAcompanhar::AcompanhamentoNaoConfirmado));
        assert_eq!(conferir_inclusao(&antes, &[a("1","G","MUDOU"), a("2","H","B")], "H", "B"),
            Err(ErroAcompanhar::AcompanhamentoNaoConfirmado));
        assert!(conferir_remocao(&depois, &antes, "2").is_ok());
        assert_eq!(conferir_remocao(&depois, &antes, "1"),
            Err(ErroAcompanhar::AcompanhamentoNaoConfirmado));
    }
    #[test]
    fn lista_realista_com_entidades() {
        let html = tabela(&linha("831", "Boas-Vindas", "Prazos &amp; revisões"));
        let lida = acompanhamentos_da_lista(&html).unwrap();
        assert_eq!(lida[0], a("831", "Boas-Vindas", "Prazos & revisões"));
    }
}
