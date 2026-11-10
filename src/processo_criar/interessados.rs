//! Interessados do cadastro de processo: autocomplete que a tela publicou.
//! Cada pedido faz uma consulta sem efeito de escrita, por URL assinada
//! observada, e exige identidade unívoca; nenhum nome vira ID inventado.
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, TelaServida, decodificar_atributo, derivar_consulta_ajax,
    localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::transporte::{corpo::decode_http_body, ler_limitado};
use crate::visao_do_controle::EnvioDoControle;

const PREFIXO: &str = "objAutoCompletarInteressadoRI1225 = new infraAjaxAutoCompletar('hdnIdInteressadoProcedimento','txtInteressadoProcedimento','";
const AUTOCOMPLETAR: ConsultaAjax = ConsultaAjax {
    acao_ajax: "contato_auto_completar_contexto_RI1225",
    campos: &["palavras_pesquisa"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/inicio-do-processo.md: autocomplete do contato",
    },
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroInteressado {
    AutocompletarNaoDeclarado,
    DerivacaoRecusada,
    NaoOferecido,
    Ambiguo,
    Html(#[from] ErroHtml),
    /// Falha observada no transporte do AJAX consultivo, antes da escrita.
    Transporte(#[from] crate::transporte::FalhaTransporte),
}

impl ErroInteressado {
    pub(crate) fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        crate::erro_publico::Causa::status_http(self)
    }
}

fn campo_unico(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    elemento: &str,
    nome: &str,
) -> Result<(), ErroInteressado> {
    let mut achados = 0;
    for handle in form.query_selector(parser, elemento).into_iter().flatten() {
        let tag = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::ausente("interessado_controles_ausentes"))?;
        let lido = valor_do_atributo(tag, "name")
            .map_err(|e| ErroHtml::atributo("interessado_controles_ausentes", e))?;
        if lido.as_deref() == Some(nome) {
            if atributo_presente(tag, "disabled")
                .map_err(|e| ErroHtml::atributo("interessado_controles_ausentes", e))?
            {
                return Err(ErroHtml::ausente("interessado_controles_ausentes").into());
            }
            achados += 1;
        }
    }
    if achados == 1 {
        Ok(())
    } else {
        Err(ErroHtml::ausente("interessado_controles_ausentes").into())
    }
}

/// URL declarada na inicialização do autocomplete, sem executar o script.
/// Também confere os campos reais do formulário que receberá os IDs.
fn url_publicada(html: &str) -> Result<String, ErroInteressado> {
    use ErroInteressado as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::ausente("interessado_controles_ausentes"))?;
    let form = localizar_formulario(&dom, "frmProcedimentoCadastro")
        .map_err(|_| ErroHtml::ausente("interessado_controles_ausentes"))?;
    campo_unico(form, dom.parser(), "input", "hdnInteressadosProcedimento")?;
    campo_unico(form, dom.parser(), "input", "txtInteressadoProcedimento")?;
    campo_unico(form, dom.parser(), "select", "selInteressadosProcedimento")?;
    let urls: Vec<&str> = html
        .lines()
        .filter_map(|linha| linha.strip_prefix(PREFIXO))
        .filter_map(|resto| resto.strip_suffix("');"))
        .collect();
    match urls.as_slice() {
        [url] if url.starts_with("controlador_ajax.php?") => Ok((*url).to_owned()),
        _ => Err(E::AutocompletarNaoDeclarado),
    }
}

fn itens_da_resposta(xml: &str) -> Result<Vec<(String, String)>, ErroInteressado> {
    use ErroInteressado as E;
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::valor("interessado_resposta_nao_reconhecida"))?;
    let parser = dom.parser();
    let roots: Vec<_> = dom.query_selector("itens").into_iter().flatten().collect();
    let [root] = roots.as_slice() else {
        return Err(ErroHtml::valor("interessado_resposta_nao_reconhecida").into());
    };
    let root = root
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroHtml::valor("interessado_resposta_nao_reconhecida"))?;
    let mut itens = Vec::new();
    // Só itens subordinados à raiz oferecida no XML são selecionáveis;
    // outros elementos do documento não são candidatos de interessado.
    for handle in root.query_selector(parser, "item").into_iter().flatten() {
        let tag = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::valor("interessado_resposta_nao_reconhecida"))?;
        let ler = |nome| -> Result<String, E> {
            valor_do_atributo(tag, nome)
                .map_err(|e| {
                    E::Html(ErroHtml::atributo(
                        "interessado_resposta_nao_reconhecida",
                        e,
                    ))
                })?
                .and_then(|s| decodificar_atributo(&s))
                .filter(|s| !s.is_empty())
                .ok_or(E::Html(ErroHtml::valor(
                    "interessado_resposta_nao_reconhecida",
                )))
        };
        itens.push((ler("id")?, ler("descricao")?));
    }
    Ok(itens)
}

fn escolher(pedido: &str, itens: &[(String, String)]) -> Result<(String, String), ErroInteressado> {
    use ErroInteressado as E;
    let normalizar = |texto: &str| {
        texto
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let pedido = normalizar(pedido);
    let encontrados: Vec<_> = itens
        .iter()
        .filter(|(_, nome)| normalizar(nome) == pedido)
        .collect();
    match encontrados.as_slice() {
        [(id, nome)] => Ok((id.clone(), nome.clone())),
        [] => Err(E::NaoOferecido),
        _ => Err(E::Ambiguo),
    }
}

/// Resolve um interessado por POST de consulta, sem retry e sem GET extra.
/// A resposta XML informa o ID, não o texto digitado pelo usuário.
pub(crate) fn resolver<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    cookie: &str,
    pedidos: &[String],
    requisicoes: &mut u32,
) -> Result<Vec<(String, String)>, ErroInteressado> {
    use ErroInteressado as E;
    if pedidos.is_empty() {
        return Ok(Vec::new());
    }
    let url = url_publicada(tela.html)?;
    let mut resultados = Vec::new();
    for pedido in pedidos {
        let consulta = derivar_consulta_ajax(
            &AUTOCOMPLETAR,
            instalacao,
            tela,
            &url,
            &[("palavras_pesquisa", pedido)],
        )
        .map_err(|_| E::DerivacaoRecusada)?;
        *requisicoes += 1;
        let resposta = envio.post(consulta, cookie)?;
        if resposta.resposta.status_code != 200 {
            return Err(E::Transporte(crate::transporte::recusar_status(
                resposta.resposta,
            )));
        }
        if !matches!(
            resposta
                .resposta
                .mime
                .split(';')
                .next()
                .unwrap_or_default()
                .trim(),
            "application/xml" | "text/xml"
        ) {
            return Err(ErroHtml::valor("interessado_resposta_nao_reconhecida").into());
        }
        let mime = resposta.resposta.mime.clone();
        let bytes = ler_limitado(resposta.resposta.corpo, crate::MAX_HTML_BYTES)?;
        let xml = decode_http_body(&bytes, &mime)
            .map_err(|_| ErroHtml::valor("interessado_resposta_nao_reconhecida"))?;
        let escolhido = escolher(pedido, &itens_da_resposta(&xml)?)?;
        if !resultados
            .iter()
            .any(|(id, _): &(String, String)| *id == escolhido.0)
        {
            resultados.push(escolhido);
        }
    }
    Ok(resultados)
}

pub(crate) fn valor_da_lupa(itens: &[(String, String)]) -> String {
    itens
        .iter()
        .map(|(id, nome)| format!("{id}±{nome}"))
        .collect::<Vec<_>>()
        .join("¥")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xml_sem_correspondencia_exata_eh_recusado() {
        let xml =
            "<itens><item id='1' descricao='Pessoa A'/><item id='2' descricao='Pessoa B'/></itens>";
        let itens = itens_da_resposta(xml).unwrap();
        assert_eq!(
            escolher("Pessoa C", &itens),
            Err(ErroInteressado::NaoOferecido)
        );
        assert_eq!(
            escolher("Pessoa A", &itens),
            Ok(("1".into(), "Pessoa A".into()))
        );
    }
    #[test]
    fn duplicados_com_nome_igual_sao_ambiguos() {
        let xml =
            "<itens><item id='1' descricao='Pessoa A'/><item id='2' descricao='Pessoa A'/></itens>";
        assert_eq!(
            escolher("Pessoa A", &itens_da_resposta(xml).unwrap()),
            Err(ErroInteressado::Ambiguo)
        );
    }
    /// Resposta AJAX sintética adversarial: somente `item` sob a única
    /// raiz `itens` representa interessado oferecido pelo SEI. Um item
    /// alheio à raiz não pode virar ID selecionável na criação #245.
    #[test]
    fn autocomplete_nao_aceita_item_fora_da_raiz_xml() {
        // Estrutura dos itens: autocomplete de interessado publicado no
        // cadastro; o segundo `item` é uma mutação sintética negativa.
        let xml = "<itens><item id='1' descricao='Pessoa A'></item></itens><item id='2' descricao='Pessoa B'></item>";
        let itens = itens_da_resposta(xml).unwrap();
        assert_eq!(itens, vec![("1".into(), "Pessoa A".into())]);
        assert_eq!(
            escolher("Pessoa B", &itens),
            Err(ErroInteressado::NaoOferecido)
        );
    }

    #[test]
    fn codificacao_da_lupa_preserva_ids_e_nomes() {
        assert_eq!(
            valor_da_lupa(&[
                ("1".into(), "Pessoa A".into()),
                ("2".into(), "Pessoa B".into())
            ]),
            "1±Pessoa A¥2±Pessoa B"
        );
    }
}
