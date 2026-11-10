//! Leitura da lista de tipos publicada em "Iniciar Processo".
//! Não executa o JavaScript do SEI nem monta URL assinada.
//! Issue #245: o POST de escolher tipo na 4.1/5.0 abre cadastro;
//! não é a escrita de criação de processo.
use crate::erro_html::ErroHtml;
use crate::html_atributos::{atributo_presente, nos_com_id, valor_do_atributo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroListaDeTipos {
    TipoNaoOferecido,
    TipoSemAcao,
    ListaIncompleta,
    Html(#[from] ErroHtml),
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

fn link_do_cadastro(href: &str) -> Option<String> {
    let query = href.split_once('?')?.1.split('#').next()?;
    let mut acao = 0;
    let mut id = 0;
    for (chave, valor) in query.split('&').filter_map(|par| par.split_once('=')) {
        if chave == "acao" && valor == "procedimento_gerar" {
            acao += 1;
        }
        if chave == "id_tipo_procedimento"
            && !valor.is_empty()
            && valor.bytes().all(|b| b.is_ascii_digit())
        {
            id += 1;
        }
    }
    (acao == 1 && id == 1).then(|| href.to_owned())
}

/// Compara o nome apresentado pelo SEI, sem escolher um candidato por
/// prefixo ou por posição. Entidades HTML são decodificadas pelo leitor
/// comum antes da comparação.
pub(crate) fn tipo_na_lista(html: &str, pedido: &str) -> Result<TipoEscolhido, ErroListaDeTipos> {
    use ErroListaDeTipos as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("lista_de_tipos_html_invalido"))?;
    let tabelas = nos_com_id(&dom, "tblTipoProcedimento")
        .map_err(|e| ErroHtml::atributo("lista_de_tipos_html_invalido", e))?;
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroHtml::ausente("tabela_de_tipos_ausente_ou_repetida").into());
    };
    let parser = dom.parser();
    let tag = tabela
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroHtml::html("lista_de_tipos_html_invalido"))?;
    if tag.name().as_utf8_str() != "table" {
        return Err(ErroHtml::ausente("tabela_de_tipos_ausente_ou_repetida").into());
    }
    let mut encontrados = Vec::new();
    for handle in tag.query_selector(parser, "a").into_iter().flatten() {
        let ancora = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("lista_de_tipos_html_invalido"))?;
        let classe = valor_do_atributo(ancora, "class")
            .map_err(|e| ErroHtml::atributo("lista_de_tipos_html_invalido", e))?;
        if !classe
            .as_deref()
            .is_some_and(|s| s.split_whitespace().any(|c| c == "ancoraOpcao"))
        {
            continue;
        }
        let nome = html_escape::decode_html_entities(&ancora.inner_text(parser))
            .into_owned()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if nome != pedido.split_whitespace().collect::<Vec<_>>().join(" ") {
            continue;
        }
        let href = valor_do_atributo(ancora, "href")
            .map_err(|e| ErroHtml::atributo("lista_de_tipos_html_invalido", e))?
            .map(|valor| {
                crate::formulario_web::decodificar_atributo(&valor)
                    .ok_or(E::Html(ErroHtml::html("lista_de_tipos_html_invalido")))
            })
            .transpose()?;
        let onclick = valor_do_atributo(ancora, "onclick")
            .map_err(|e| ErroHtml::atributo("lista_de_tipos_html_invalido", e))?;
        let desabilitada = atributo_presente(ancora, "disabled")
            .map_err(|e| ErroHtml::atributo("lista_de_tipos_html_invalido", e))?;
        let escolha = match (
            href.as_deref().and_then(link_do_cadastro),
            onclick.as_deref().and_then(escolher_literal),
        ) {
            (Some(link), None) => Some(EscolhaDoTipo::Link(link)),
            (None, Some(id)) if href.as_deref().is_none_or(|v| v == "#") => {
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
        [None] => Err(E::TipoSemAcao),
        [_, _, ..] => Err(ErroHtml::repetido("tipo_de_processo_ambiguo").into()),
        [] if lista_completa(html) => Err(E::TipoNaoOferecido),
        [] => Err(E::ListaIncompleta),
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
        let html = tela(
            "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12'>Ensaio</a>",
            true,
        );
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
        let html = tela(
            "<a class='ancoraOpcao' href='#' onclick='escolher(23)'>Ensaio</a>",
            true,
        );
        assert_eq!(
            tipo_na_lista(&html, "Ensaio").unwrap().escolha,
            EscolhaDoTipo::Formulario("23".into())
        );
    }

    #[test]
    fn nao_confunde_lista_curta_com_tipo_inexistente() {
        assert_eq!(
            tipo_na_lista(&tela("", false), "Outro"),
            Err(ErroListaDeTipos::ListaIncompleta)
        );
    }

    #[test]
    fn tipo_ausente_so_na_lista_completa() {
        assert_eq!(
            tipo_na_lista(&tela("", true), "Outro"),
            Err(ErroListaDeTipos::TipoNaoOferecido)
        );
    }

    #[test]
    fn nome_duplicado_eh_recusado() {
        let link = "<a class='ancoraOpcao' onclick='escolher(1)' href='#'>Ensaio</a>";
        assert_eq!(
            tipo_na_lista(&tela(&format!("{link}{link}"), true), "Ensaio"),
            Err(ErroListaDeTipos::Html(ErroHtml::repetido(
                "tipo_de_processo_ambiguo"
            )))
        );
    }

    #[test]
    fn acao_nao_reconhecida_eh_recusada() {
        let html = tela(
            "<a class='ancoraOpcao' href='#' onclick='avaliar(1)'>Ensaio</a>",
            true,
        );
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::TipoSemAcao)
        );
    }

    #[test]
    fn tabela_ausente_eh_recusada() {
        assert_eq!(
            tipo_na_lista("<div/>", "Ensaio"),
            Err(ErroListaDeTipos::Html(ErroHtml::ausente(
                "tabela_de_tipos_ausente_ou_repetida"
            )))
        );
    }

    #[test]
    fn tabela_repetida_eh_recusada() {
        let html = format!("{}{}", tela("", true), tela("", true));
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::Html(ErroHtml::ausente(
                "tabela_de_tipos_ausente_ou_repetida"
            )))
        );
    }

    #[test]
    fn nao_aceita_prefixo() {
        let html = tela(
            "<a class='ancoraOpcao' href='#' onclick='escolher(1)'>Ensaio Especial</a>",
            true,
        );
        assert_eq!(
            tipo_na_lista(&html, "Ensaio"),
            Err(ErroListaDeTipos::TipoNaoOferecido)
        );
    }

    #[test]
    fn ancora_desabilitada_nao_pode_ser_escolhida() {
        let html = tela(
            "<a class='ancoraOpcao' href='#' onclick='escolher(3)' disabled>Ensaio</a>",
            true,
        );
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
