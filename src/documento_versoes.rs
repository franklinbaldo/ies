//! Histórico Web observado: nunca constrói links nem abre editor.
use crate::documento::NumeroDocumento;
use crate::html_atributos::{nos_por_elemento_e_classe, valor_do_atributo};
use std::collections::HashSet;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersaoDocumento(NonZeroU32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum ErroVersao {
    #[causa(nome = "linha_historico_incompleta")]
    LinhaIncompleta,
    #[causa(nome = "link_versao_inseguro")]
    LinkInseguro,
    #[causa(nome = "identidade_documento_divergente")]
    DocumentoDivergente,
    #[causa(nome = "lista_versoes_incompleta")]
    ListaIncompleta,
    VersaoNaoObservada,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}
impl ErroVersao {
    pub fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
}

impl VersaoDocumento {
    pub fn nova(numero: u32) -> Result<Self, ErroVersao> {
        NonZeroU32::new(numero).map(Self).ok_or(ErroVersao::Html(
            crate::erro_html::ErroHtml::valor(
                "versao_documento_invalida",
                "o valor não está na forma observada",
            ),
        ))
    }
    pub fn interpretar(texto: &str) -> Result<Self, ErroVersao> {
        if texto.is_empty() || !texto.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroVersao::Html(crate::erro_html::ErroHtml::valor(
                "versao_documento_invalida",
                "o valor não está na forma observada",
            )));
        }
        Self::nova(texto.parse().map_err(|_| {
            ErroVersao::Html(crate::erro_html::ErroHtml::valor(
                "versao_documento_invalida",
                "o valor não está na forma observada",
            ))
        })?)
    }
    pub fn numero(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaVersao {
    pub versao: VersaoDocumento,
    pub usuario: String,
    pub unidade: String,
    pub modificacao: String,
    // Link opaco: nunca serializar ou logar, nem reconstruir a partir de infra_hash.
    link: String,
}
impl EntradaVersao {
    pub fn link_privado(&self) -> &str {
        &self.link
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricoDocumento {
    entradas: Vec<EntradaVersao>,
}
impl HistoricoDocumento {
    pub fn atual(&self) -> Option<VersaoDocumento> {
        self.entradas.iter().map(|e| e.versao).max()
    }
    pub fn selecionar(&self, numero: VersaoDocumento) -> Result<&EntradaVersao, ErroVersao> {
        self.entradas
            .iter()
            .find(|e| e.versao == numero)
            .ok_or(ErroVersao::VersaoNaoObservada)
    }
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entradas.len()
    }
}

fn texto(no: &tl::Node<'_>, parser: &tl::Parser<'_>) -> String {
    no.inner_text(parser)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Não normalizar chaves percent-encoded, pontos ou campos repetidos:
// o PHP interpreta essas variantes de forma potencialmente ambígua.
fn conferir_link(href: &str, id: &str, versao: VersaoDocumento) -> Result<(), ErroVersao> {
    let query = href
        .strip_prefix("controlador.php?")
        .ok_or(ErroVersao::LinkInseguro)?;
    let mut acao = None;
    let mut id_observado = None;
    let mut versao_observada = None;
    let mut hash = None;
    let parametros = query.replace("&amp;", "&");
    for parte in parametros.split('&') {
        let (chave, valor) = parte.split_once('=').ok_or(ErroVersao::LinkInseguro)?;
        if chave.is_empty()
            || !chave.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
            || valor.bytes().any(|b| b <= 32 || b >= 127)
        {
            return Err(ErroVersao::LinkInseguro);
        }
        let slot = match chave {
            "acao" => &mut acao,
            "id_documento" => &mut id_observado,
            "versao" => &mut versao_observada,
            "infra_hash" => &mut hash,
            _ => continue,
        };
        // `id_documento` servido duas vezes com o mesmo valor (sete
        // instalações); valores diferentes ou outra chave repetida: recusa.
        match slot.replace(valor) {
            Some(anterior) if chave == "id_documento" && anterior == valor => {}
            Some(_) => return Err(ErroVersao::LinkInseguro),
            None => {}
        }
    }
    if acao != Some("documento_visualizar")
        || versao_observada != Some(versao.numero().to_string().as_str())
    {
        return Err(ErroVersao::LinkInseguro);
    }
    if id_observado != Some(id) {
        return Err(ErroVersao::DocumentoDivergente);
    }
    if !hash.is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())) {
        return Err(ErroVersao::LinkInseguro);
    }
    Ok(())
}

fn linhas_do_historico(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id_interno: &str,
) -> Result<Vec<EntradaVersao>, ErroVersao> {
    let colunas_historico_inesperadas = ErroVersao::Html(crate::erro_html::ErroHtml::valor(
        "colunas_historico_inesperadas",
        "o valor não está na forma observada",
    ));
    let mut cabecalho = false;
    let mut entradas = Vec::new();
    let mut encontrados = HashSet::new();
    for tr_handle in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let tr = tr_handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroVersao::LinhaIncompleta)?;
        let cols: Vec<_> = tr
            .query_selector(parser, "th")
            .into_iter()
            .flatten()
            .filter_map(|h| h.get(parser))
            .collect();
        if !cols.is_empty() {
            let titulos: Vec<_> = cols.iter().map(|n| texto(n, parser)).collect();
            if cabecalho
                || titulos
                    != [
                        "",
                        "Versão",
                        "Usuário",
                        "Unidade",
                        "Última Modificação",
                        "Ações",
                    ]
            {
                return Err(colunas_historico_inesperadas);
            }
            cabecalho = true;
            continue;
        }
        let celulas: Vec<_> = tr
            .query_selector(parser, "td")
            .into_iter()
            .flatten()
            .filter_map(|h| h.get(parser))
            .collect();
        if celulas.is_empty() {
            continue;
        }
        if !cabecalho || celulas.len() != 6 {
            return Err(ErroVersao::LinhaIncompleta);
        }
        let numero = VersaoDocumento::interpretar(&texto(celulas[1], parser))?;
        if !encontrados.insert(numero) {
            return Err(ErroVersao::Html(crate::erro_html::ErroHtml::repetido(
                "versao_documento_duplicada",
                "o elemento aparece mais de uma vez",
            )));
        }
        let td = celulas[5].as_tag().ok_or(ErroVersao::LinhaIncompleta)?;
        let mut links = Vec::new();
        for a in td
            .query_selector(parser, "a")
            .into_iter()
            .flatten()
            .filter_map(|h| h.get(parser))
            .filter_map(tl::Node::as_tag)
        {
            // `href` lido da fonte da tag; tag ilegível não some da contagem.
            // Âncora de fragmento (`#ID-N`, "Recuperar Versão" com o link no
            // `onclick`) não é link de versão e nunca é seguida.
            let href = valor_do_atributo(a, "href").map_err(|e| {
                ErroVersao::Html(crate::erro_html::ErroHtml::atributo(
                    "link_versao_ausente_ou_ambiguo",
                    "a tela não tem o elemento observado",
                    e,
                ))
            })?;
            links.extend(href.filter(|h| !h.starts_with('#')));
        }
        if links.len() != 1 {
            return Err(ErroVersao::Html(crate::erro_html::ErroHtml::ausente(
                "link_versao_ausente_ou_ambiguo",
                "a tela não tem o elemento observado",
            )));
        }
        conferir_link(&links[0], id_interno, numero)?;
        let valores: Vec<_> = celulas[2..5].iter().map(|n| texto(n, parser)).collect();
        if valores.iter().any(String::is_empty) {
            return Err(ErroVersao::LinhaIncompleta);
        }
        entradas.push(EntradaVersao {
            versao: numero,
            usuario: valores[0].clone(),
            unidade: valores[1].clone(),
            modificacao: valores[2].clone(),
            link: links.remove(0),
        });
        if entradas.len() > 10_000 {
            return Err(ErroVersao::ListaIncompleta);
        }
    }
    if !cabecalho {
        return Err(colunas_historico_inesperadas);
    }
    Ok(entradas)
}

/// Caption única é requisito de identidade e contagem da tabela Web.
fn legenda_unica(tabela: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> Result<String, ErroVersao> {
    let mut legendas = tabela
        .query_selector(parser, "caption")
        .into_iter()
        .flatten();
    let primeira = legendas.next().ok_or(ErroVersao::ListaIncompleta)?;
    if legendas.next().is_some() {
        return Err(ErroVersao::Html(crate::erro_html::ErroHtml::repetido(
            "legendas_historico_versoes_repetidas",
            "o elemento aparece mais de uma vez",
        )));
    }
    primeira
        .get(parser)
        .map(|no| texto(no, parser))
        .ok_or(ErroVersao::ListaIncompleta)
}

/// Interpreta uma lista observada. O total declarado é facultativo, mas, quando
/// disponível na tabela de origem, é obrigatório conferir a cobertura.
pub fn interpretar_historico(
    html: &str,
    documento: &NumeroDocumento,
    id_interno: &str,
    total_declarado: Option<usize>,
) -> Result<HistoricoDocumento, ErroVersao> {
    if id_interno.is_empty() || !id_interno.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroVersao::DocumentoDivergente);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        ErroVersao::Html(crate::erro_html::ErroHtml::ausente(
            "historico_documento_ausente",
            "a tela não tem o elemento observado",
        ))
    })?;
    let parser = dom.parser();
    let mut tabela = None;
    for handle in nos_por_elemento_e_classe(&dom, None, "table", "infraTable").map_err(|e| {
        ErroVersao::Html(crate::erro_html::ErroHtml::atributo(
            "historico_documento_ausente",
            "a tela não tem o elemento observado",
            e,
        ))
    })? {
        let no =
            handle
                .get(parser)
                .ok_or(ErroVersao::Html(crate::erro_html::ErroHtml::ausente(
                    "historico_documento_ausente",
                    "a tela não tem o elemento observado",
                )))?;
        let tag = no
            .as_tag()
            .ok_or(ErroVersao::Html(crate::erro_html::ErroHtml::ausente(
                "historico_documento_ausente",
                "a tela não tem o elemento observado",
            )))?;
        let captions: Vec<_> = tag
            .query_selector(parser, "caption")
            .into_iter()
            .flatten()
            .filter_map(|h| h.get(parser))
            .collect();
        if captions
            .iter()
            .any(|c| texto(c, parser).contains("Versões"))
        {
            if tabela.is_some() {
                return Err(ErroVersao::Html(crate::erro_html::ErroHtml::repetido(
                    "historicos_documento_duplicados",
                    "o elemento aparece mais de uma vez",
                )));
            }
            tabela = Some(tag);
        }
    }
    let tabela = tabela.ok_or(ErroVersao::Html(crate::erro_html::ErroHtml::ausente(
        "historico_documento_ausente",
        "a tela não tem o elemento observado",
    )))?;
    let legenda = legenda_unica(tabela, parser)?;
    let declaradas = legenda
        .rsplit_once('(')
        .and_then(|(_, resto)| resto.split_once(" registros"))
        .and_then(|(numero, _)| numero.parse::<usize>().ok())
        .ok_or(ErroVersao::ListaIncompleta)?;
    let entradas = linhas_do_historico(tabela, parser, id_interno)?;
    if declaradas != entradas.len() || total_declarado.is_some_and(|n| n != declaradas) {
        return Err(ErroVersao::ListaIncompleta);
    }
    let _ = documento; // Número SEI é opaco: ID interno vem do nó do documento.
    Ok(HistoricoDocumento { entradas })
}

#[cfg(test)]
#[path = "documento_versoes/testes.rs"]
pub(crate) mod testes;
