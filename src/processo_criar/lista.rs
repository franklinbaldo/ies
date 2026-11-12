//! Leitura da lista de tipos publicada em "Iniciar Processo".
//! Não executa o JavaScript do SEI nem monta URL assinada.
//! Issue #245: o POST de escolher tipo na 4.1/5.0 abre cadastro;
//! não é a escrita de criação de processo.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroListaDeTipos {
    TipoNaoOferecido,
    TipoSemAcao,
    ListaIncompleta,
    TabelaDeTiposAusenteOuRepetida,
    TipoDeProcessoAmbiguo,
    /// A consulta do link de cadastro não se lê em pares `chave=valor`.
    ConsultaDoCadastroIlegivel(#[source] ErroConsultaDoControlador),
    /// O link de cadastro não tem exatamente uma `acao` reconhecida nem um
    /// `id_tipo_procedimento` único.
    ChaveDoCadastroIlegivel(#[source] ErroChaveDaConsulta),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EscolhaDoTipo {
    /// A URL é a que a própria âncora publicou; nunca expor em logs.
    Link(String),
    /// O id é o argumento literal de `escolher(id)` publicado na âncora.
    Formulario(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TipoEscolhido {
    pub(crate) nome: String,
    pub(crate) escolha: EscolhaDoTipo,
}

/// A lista pode trazer somente os tipos usados. Somente uma lista marcada
/// como completa permite concluir pela ausência de nome.
fn lista_completa(html: &str) -> bool {
    let tem_completa = html.contains("exibirTiposProcedimento('U')")
        || html.contains("exibirTiposProcedimento(&quot;U&quot;)");
    let tem_curta = html.contains("exibirTiposProcedimento('T')")
        || html.contains("exibirTiposProcedimento(&quot;T&quot;)");
    tem_completa && !tem_curta
}

fn escolher_literal(js: &str) -> Option<String> {
    let js = js.trim_end_matches(';');
    let id = js.strip_prefix("escolher(")?.strip_suffix(')')?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.to_owned())
}

/// Link de cadastro: a consulta é lida inteira, com uma `acao` reconhecida
/// e um `id_tipo_procedimento` numérico. Sem `?`, não é link de cadastro.
fn link_do_cadastro(href: &str) -> Result<Option<String>, ErroListaDeTipos> {
    let Some((_, consulta)) = href.split_once('?') else {
        return Ok(None);
    };
    let consulta = consulta.split('#').next().unwrap_or_default();
    let consulta = ConsultaDoControlador::ler(consulta)
        .map_err(ErroListaDeTipos::ConsultaDoCadastroIlegivel)?;
    let acao = consulta
        .acao()
        .map_err(ErroListaDeTipos::ChaveDoCadastroIlegivel)?;
    let id = consulta
        .unico("id_tipo_procedimento")
        .map_err(ErroListaDeTipos::ChaveDoCadastroIlegivel)?;
    let numerico = !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit());
    Ok((acao == AcaoDoSei::ProcedimentoGerar && numerico).then(|| href.to_owned()))
}

/// Compara o nome apresentado pelo SEI, sem escolher um candidato por
/// prefixo ou por posição. Entidades HTML são decodificadas pelo leitor
/// comum antes da comparação.
pub(crate) fn tipo_na_lista(
    html: &crate::tela_html::HtmlServido,
    pedido: &str,
) -> Result<TipoEscolhido, ErroListaDeTipos> {
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblTipoProcedimento").collect::<Vec<_>>();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroListaDeTipos::TabelaDeTiposAusenteOuRepetida);
    };
    let tag = *tabela;
    if tag.nome() != "table" {
        return Err(ErroListaDeTipos::TabelaDeTiposAusenteOuRepetida);
    }
    let mut encontrados = Vec::new();
    for handle in tag.elementos("a") {
        let classe = handle.atributo("class");
        if !classe.is_some_and(|s| s.split_whitespace().any(|c| c == "ancoraOpcao")) {
            continue;
        }
        let nome = handle
            .texto()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if nome != pedido.split_whitespace().collect::<Vec<_>>().join(" ") {
            continue;
        }
        let href = handle.atributo("href");
        let onclick = handle.atributo("onclick");
        let desabilitada = handle.presente("disabled");
        let link = href.map(link_do_cadastro).transpose()?.flatten();
        let escolha = match (link, onclick.and_then(escolher_literal)) {
            (Some(link), None) => Some(EscolhaDoTipo::Link(link)),
            (None, Some(id)) if href.is_none_or(|v| v == "#") => {
                Some(EscolhaDoTipo::Formulario(id))
            }
            _ => None,
        };
        encontrados.push(
            escolha
                .filter(|_| !desabilitada)
                .map(|escolha| TipoEscolhido { nome, escolha }),
        );
    }
    match encontrados.as_slice() {
        [Some(escolhido)] => Ok(escolhido.clone()),
        [None] => Err(ErroListaDeTipos::TipoSemAcao),
        [_, _, ..] => Err(ErroListaDeTipos::TipoDeProcessoAmbiguo),
        [] if lista_completa(html) => Err(ErroListaDeTipos::TipoNaoOferecido),
        [] => Err(ErroListaDeTipos::ListaIncompleta),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tela(links: &str, completa: bool) -> String {
        let filtro = if completa { "U" } else { "T" };
        format!(
            "<button onclick=\"exibirTiposProcedimento('{filtro}')\">Tipos</button><table id=\"tblTipoProcedimento\"><tbody><tr><td>{links}</td></tr></tbody></table>"
        )
    }

    #[test]
    fn escolhe_link_literal_da_versao_40() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12'>Ensaio</a>",
            true,
        ));
        // O valor de href é bruto, com a entidade publicada, preservada.
        assert!(matches!(
            tipo_na_lista(&html, "Ensaio"),
            Ok(TipoEscolhido {
                escolha: EscolhaDoTipo::Link(_),
                ..
            })
        ));
    }

    #[test]
    fn escolhe_formulario_da_versao_41() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='#' onclick='escolher(23)'>Ensaio</a>",
            true,
        ));
        assert_eq!(
            tipo_na_lista(&html, "Ensaio").unwrap().escolha,
            EscolhaDoTipo::Formulario("23".into())
        );
    }

    #[test]
    fn nao_confunde_lista_curta_com_tipo_inexistente() {
        assert_eq!(
            tipo_na_lista(
                &crate::tela_html::HtmlServido::de_teste(&tela("", false)),
                "Outro"
            ),
            Err(ErroListaDeTipos::ListaIncompleta)
        );
    }

    #[test]
    fn tipo_ausente_so_na_lista_completa() {
        assert_eq!(
            tipo_na_lista(
                &crate::tela_html::HtmlServido::de_teste(&tela("", true)),
                "Outro"
            ),
            Err(ErroListaDeTipos::TipoNaoOferecido)
        );
    }

    #[test]
    fn nome_duplicado_eh_recusado() {
        let link = "<a class='ancoraOpcao' onclick='escolher(1)' href='#'>Ensaio</a>";
        assert_eq!(
            tipo_na_lista(
                &crate::tela_html::HtmlServido::de_teste(&tela(&format!("{link}{link}"), true)),
                "Ensaio"
            ),
            Err(ErroListaDeTipos::TipoDeProcessoAmbiguo)
        );
    }

    #[test]
    fn acao_nao_reconhecida_eh_recusada() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='#' onclick='avaliar(1)'>Ensaio</a>",
            true,
        ));
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::TipoSemAcao)
        );
    }

    #[test]
    fn tabela_ausente_eh_recusada() {
        assert_eq!(
            tipo_na_lista(&crate::tela_html::HtmlServido::de_teste("<div/>"), "Ensaio"),
            Err(ErroListaDeTipos::TabelaDeTiposAusenteOuRepetida)
        );
    }

    #[test]
    fn tabela_repetida_eh_recusada() {
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}{}",
            tela("", true),
            tela("", true)
        ));
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::TabelaDeTiposAusenteOuRepetida)
        );
    }

    #[test]
    fn link_com_duas_acoes_nao_vira_escolha() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;acao=procedimento_excluir&amp;id_tipo_procedimento=12'>Ensaio</a>",
            true,
        ));
        assert!(tipo_na_lista(&html, "Ensaio").is_err());
    }

    #[test]
    fn link_com_par_sem_igual_nao_vira_escolha() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12&amp;lixo'>Ensaio</a>",
            true,
        ));
        assert!(tipo_na_lista(&html, "Ensaio").is_err());
    }

    #[test]
    fn nao_aceita_prefixo() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='#' onclick='escolher(1)'>Ensaio Especial</a>",
            true,
        ));
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::TipoNaoOferecido)
        );
    }

    #[test]
    fn ancora_desabilitada_nao_pode_ser_escolhida() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            "<a class='ancoraOpcao' href='#' onclick='escolher(3)' disabled>Ensaio</a>",
            true,
        ));
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::TipoSemAcao)
        );
    }

    #[test]
    fn nao_aceita_js_arbitrario() {
        assert_eq!(escolher_literal("escolher(1); executar()"), None);
        assert_eq!(escolher_literal("escolher('1')"), None);
    }
}
