//! Consulta de texto padrão publicado no cadastro; não cria documento.
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, ErroSubmissaoFormulario, TelaServida, derivar_consulta_ajax,
    localizar_formulario,
};
use crate::instalacao::Instalacao;
use crate::resposta_ajax::{ErroXml, RespostaAjax};
use crate::tela_html::HtmlServido;
use crate::transporte::{FalhaTransporte, ler_limitado};
use crate::visao_do_controle::EnvioDoControle;

const PREFIXO: &str = "objAutoCompletarTextoPadrao = new infraAjaxAutoCompletar('hdnIdTextoPadrao','txtTextoPadrao','";
const CONSULTA: ConsultaAjax = ConsultaAjax {
    acao_ajax: crate::acao_do_sei::AcaoAjaxDoSei::TextoPadraoAutoCompletar,
    campos: &["palavras_pesquisa"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos documento_gerar.html: consulta de opções de texto padrão",
    },
};

#[derive(Debug, serde::Serialize)]
pub(crate) struct TextoPadrao {
    pub(crate) id: String,
    pub(crate) nome: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTextoPadrao {
    ControlesNaoOferecidos,
    AutocompletarNaoDeclarado,
    Formulario(#[from] ErroSubmissaoFormulario),
    Transporte(#[from] FalhaTransporte),
    Corpo(#[from] crate::transporte::corpo::ErroCorpoHttp),
    Xml(#[from] ErroXml),
    RespostaNaoReconhecida,
    NaoOferecido,
    Ambiguo,
}

pub(super) fn url_publicada(html: &HtmlServido) -> Result<String, ErroTextoPadrao> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, "frmDocumentoCadastro")?;
    for (nome, tipo, valor) in [
        ("rdoTextoInicial", "radio", Some("T")),
        ("txtTextoPadrao", "text", None),
        ("hdnIdTextoPadrao", "hidden", None),
    ] {
        let controles: Vec<_> = form
            .elementos("input")
            .filter(|campo| {
                campo.atributo("name") == Some(nome)
                    && campo.atributo("type") == Some(tipo)
                    && valor.is_none_or(|v| campo.atributo("value") == Some(v))
            })
            .collect();
        if !matches!(controles.as_slice(), [campo] if !campo.presente("disabled")) {
            return Err(ErroTextoPadrao::ControlesNaoOferecidos);
        }
    }
    let inicializar_no_onload = dom
        .elementos("body")
        .any(|body| body.atributo("onload").map(str::trim) == Some("inicializar();"));
    let mut urls = Vec::new();
    for script in dom.elementos("script") {
        if script.presente("src")
            || script
                .atributo("type")
                .is_some_and(|tipo| !tipo.trim().eq_ignore_ascii_case("text/javascript"))
        {
            continue;
        }
        let texto = script.texto();
        if texto.contains(PREFIXO) {
            urls.extend(
                urls_declaradas(&texto, inicializar_no_onload)
                    .ok_or(ErroTextoPadrao::AutocompletarNaoDeclarado)?,
            );
        }
    }
    match urls.as_slice() {
        [url] if url.starts_with("controlador_ajax.php?") => Ok(url.clone()),
        _ => Err(ErroTextoPadrao::AutocompletarNaoDeclarado),
    }
}

/// Reconhece a forma inline sintética e a função inicializar publicada
/// pelo cadastro observado. Não interpreta o fluxo de controle do JS.
fn urls_declaradas(texto: &str, chamado_no_onload: bool) -> Option<Vec<String>> {
    let mut urls = Vec::new();
    let mut urls_inicializar = Vec::new();
    let mut em_inicializar = false;
    let mut chamado = chamado_no_onload;
    super::guarda_navegacao::visitar_codigo(texto, |indice, atual, profundidade, ultimo| {
        if atual == '}' && profundidade == 1 {
            em_inicializar = false;
        }
        if !matches!(atual, 'f' | 'i' | 'o')
            || !texto[..indice].rsplit('\n').next()?.trim().is_empty()
            || !matches!(ultimo, ' ' | ';' | '}' | '{')
        {
            return Some(());
        }
        let linha = texto[indice..].lines().next()?.trim_end();
        if profundidade == 0 {
            if matches!(linha, "function inicializar(){" | "function inicializar() {") {
                em_inicializar = true;
            } else if linha == "inicializar();" {
                chamado = true;
            }
        }
        if let Some(url) = linha
            .strip_prefix(PREFIXO)
            .and_then(|resto| resto.strip_suffix("');"))
        {
            if profundidade == 0 {
                urls.push(url.to_owned());
            } else if profundidade == 1 && em_inicializar {
                urls_inicializar.push(url.to_owned());
            }
        }
        Some(())
    })?;
    if chamado {
        urls.extend(urls_inicializar);
    }
    Some(urls)
}

/// Nome exato, não busca parcial; mantém o ID e a descrição da mesma opção.
fn selecionar(xml: &str, nome: &str) -> Result<TextoPadrao, ErroTextoPadrao> {
    let resposta = RespostaAjax::ler(xml)?;
    let raizes = resposta.elementos("itens");
    let [raiz] = raizes.as_slice() else {
        return Err(ErroTextoPadrao::RespostaNaoReconhecida);
    };
    let mut selecionado = None;
    for item in raiz.descendentes("item") {
        let (Some(id), Some(descricao)) = (item.atributo("id"), item.atributo("descricao")) else {
            return Err(ErroTextoPadrao::RespostaNaoReconhecida);
        };
        if id.is_empty() || descricao.is_empty() {
            return Err(ErroTextoPadrao::RespostaNaoReconhecida);
        }
        if descricao == nome {
            if selecionado.is_some() {
                return Err(ErroTextoPadrao::Ambiguo);
            }
            selecionado = Some(TextoPadrao {
                id: id.to_owned(),
                nome: descricao.to_owned(),
            });
        }
    }
    selecionado.ok_or(ErroTextoPadrao::NaoOferecido)
}

pub(super) fn resolver<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    cookie: &str,
    nome: &str,
) -> Result<TextoPadrao, ErroTextoPadrao> {
    let url = url_publicada(tela.html)?;
    // Hipótese WIP: mantém o charset da tela, como as demais consultas.
    // A captura JS não prova os bytes efetivamente recebidos pelo SEI.
    let consulta = derivar_consulta_ajax(
        &CONSULTA,
        instalacao,
        tela,
        &url,
        &[("palavras_pesquisa", nome)],
    )?;
    let resposta = envio.post(consulta, cookie)?.resposta;
    if resposta.status_code != 200 {
        return Err(crate::transporte::recusar_status(resposta).into());
    }
    if !matches!(
        resposta.mime.split(';').next().unwrap_or_default().trim(),
        "application/xml" | "text/xml"
    ) {
        return Err(ErroTextoPadrao::RespostaNaoReconhecida);
    }
    let bytes = ler_limitado(resposta.corpo, crate::MAX_HTML_BYTES)?;
    let xml = HtmlServido::do_corpo(&bytes, &resposta.mime)?;
    selecionar(&xml, nome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_exato_preserva_id_opaco_e_entidades() {
        let xml =
            "<itens><item id='007' descricao='A &amp; B'/><item id='8' descricao='A'/></itens>";
        let texto = selecionar(xml, "A & B").unwrap();
        assert_eq!(texto.id, "007");
        assert_eq!(texto.nome, "A & B");
        assert!(matches!(
            selecionar(xml, "B"),
            Err(ErroTextoPadrao::NaoOferecido)
        ));
    }

    #[test]
    fn homonimos_e_resposta_sem_id_sao_recusados() {
        let xml = "<itens><item id='1' descricao='A'/><item id='2' descricao='A'/></itens>";
        assert!(matches!(
            selecionar(xml, "A"),
            Err(ErroTextoPadrao::Ambiguo)
        ));
        assert!(matches!(
            selecionar("<itens><item descricao='A'/></itens>", "A"),
            Err(ErroTextoPadrao::RespostaNaoReconhecida)
        ));
    }
    #[test]
    fn declaracao_de_autocomplete_inerte_nao_publica_url() {
        let url = "controlador_ajax.php?acao_ajax=texto_padrao_auto_completar";
        let declaracao = format!("{PREFIXO}{url}');");
        let cadastro = |script: &str| {
            HtmlServido::de_teste(&format!(
                r#"<form id="frmDocumentoCadastro">
                <input name="rdoTextoInicial" type="radio" value="T">
                <input name="txtTextoPadrao" type="text">
                <input name="hdnIdTextoPadrao" type="hidden">
                </form><script>{script}</script>"#
            ))
        };
        assert_eq!(url_publicada(&cadastro(&declaracao)).unwrap(), url);
        // Sintético: uma declaração literal dentro de um callback continua
        // candidata; o replay original é o oráculo de compatibilidade.
        let callback = format!("function inicializar() {{\n{declaracao}\n}}\ninicializar();");
        assert_eq!(url_publicada(&cadastro(&callback)).unwrap(), url);
        for inerte in [
            format!("/*\n{declaracao}\n*/"),
            format!("const exemplo = `\n{declaracao}\n`;"),
            format!("if (false) {{\n{declaracao}\n}}"),
            format!("function inicializar() {{\n{declaracao}\n}}"),
            format!("function inicializar() {{\nif (false) {{\n{declaracao}\n}}\n}}\ninicializar();"),
        ] {
            assert!(matches!(
                url_publicada(&cadastro(&inerte)),
                Err(ErroTextoPadrao::AutocompletarNaoDeclarado)
            ));
        }
    }
}
