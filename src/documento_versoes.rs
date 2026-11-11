//! Histórico Web observado: nunca constrói links nem abre editor.
use crate::acao_do_sei::AcaoDoSei;
use crate::documento::NumeroDocumento;
use crate::html_atributos::{nos_por_elemento_e_classe, valor_do_atributo};
use std::collections::HashSet;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersaoDocumento(NonZeroU32);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroVersao {
    LinhaIncompleta,
    /// O número da versão não cabe em u32; guarda o tipo do erro de conversão.
    NumeroDaVersaoIlegivel(std::num::IntErrorKind),
    LinkInseguro,
    DocumentoDivergente,
    ListaIncompleta,
    VersaoPedidaAusenteNoHistorico,
    ColunasHistoricoInesperadas,
    HistoricoDocumentoAusente,
    HistoricoDocumentoNaoInterpretado(#[source] tl::ParseError),
    HistoricoDocumentoComAtributoIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    HistoricosDocumentoDuplicados,
    LegendasHistoricoVersoesRepetidas,
    LinkVersaoAusenteOuAmbiguo,
    LinkVersaoComAtributoIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    VersaoDocumentoDuplicada,
    VersaoDocumentoInvalida,
}

impl VersaoDocumento {
    pub fn nova(numero: u32) -> Result<Self, ErroVersao> {
        NonZeroU32::new(numero)
            .map(Self)
            .ok_or(ErroVersao::VersaoDocumentoInvalida)
    }
    pub fn interpretar(texto: &str) -> Result<Self, ErroVersao> {
        if texto.is_empty() || !texto.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroVersao::VersaoDocumentoInvalida);
        }
        Self::nova(
            texto
                .parse::<u32>()
                .map_err(|erro| ErroVersao::NumeroDaVersaoIlegivel(*erro.kind()))?,
        )
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
            .ok_or(ErroVersao::VersaoPedidaAusenteNoHistorico)
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
// o PHP interpreta essas formas de modo potencialmente ambígua.
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
    if acao.and_then(AcaoDoSei::ler) != Some(AcaoDoSei::DocumentoVisualizar)
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
                return Err(ErroVersao::ColunasHistoricoInesperadas);
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
        let (true, [_, versao, usuario, unidade, modificacao, acoes]) =
            (cabecalho, celulas.as_slice())
        else {
            return Err(ErroVersao::LinhaIncompleta);
        };
        let numero = VersaoDocumento::interpretar(&texto(versao, parser))?;
        if !encontrados.insert(numero) {
            return Err(ErroVersao::VersaoDocumentoDuplicada);
        }
        let td = acoes.as_tag().ok_or(ErroVersao::LinhaIncompleta)?;
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
            let href =
                valor_do_atributo(a, "href").map_err(ErroVersao::LinkVersaoComAtributoIlegivel)?;
            links.extend(href.filter(|h| !h.starts_with('#')));
        }
        let Ok([link]) = <[String; 1]>::try_from(links) else {
            return Err(ErroVersao::LinkVersaoAusenteOuAmbiguo);
        };
        conferir_link(&link, id_interno, numero)?;
        let [usuario, unidade, modificacao] =
            [usuario, unidade, modificacao].map(|n| texto(n, parser));
        if [&usuario, &unidade, &modificacao]
            .iter()
            .any(|s| s.is_empty())
        {
            return Err(ErroVersao::LinhaIncompleta);
        }
        entradas.push(EntradaVersao {
            versao: numero,
            usuario,
            unidade,
            modificacao,
            link,
        });
        if entradas.len() > 10_000 {
            return Err(ErroVersao::ListaIncompleta);
        }
    }
    if !cabecalho {
        return Err(ErroVersao::ColunasHistoricoInesperadas);
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
        return Err(ErroVersao::LegendasHistoricoVersoesRepetidas);
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
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroVersao::HistoricoDocumentoNaoInterpretado)?;
    let parser = dom.parser();
    let mut tabela = None;
    for handle in nos_por_elemento_e_classe(&dom, None, "table", "infraTable")
        .map_err(ErroVersao::HistoricoDocumentoComAtributoIlegivel)?
    {
        let no = handle
            .get(parser)
            .ok_or(ErroVersao::HistoricoDocumentoAusente)?;
        let tag = no.as_tag().ok_or(ErroVersao::HistoricoDocumentoAusente)?;
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
                return Err(ErroVersao::HistoricosDocumentoDuplicados);
            }
            tabela = Some(tag);
        }
    }
    let tabela = tabela.ok_or(ErroVersao::HistoricoDocumentoAusente)?;
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
#[path = "documento_versoes/tests_anti_ies.rs"]
mod testes_anti_ies;

#[cfg(test)]
pub(crate) mod testes {
    use super::*;
    fn pagina(numeros: &[u32], ids: &[&str]) -> String {
        use std::fmt::Write as _;
        let mut linhas = String::new();
        for (&n, &id) in numeros.iter().zip(ids) {
            let _ = write!(
                linhas,
                "<tr><td><input type=\"checkbox\" /></td><td>{n}</td><td><a>Ana</a></td><td><a>PPI</a></td><td>31/10/2026</td><td><a href=\"controlador.php?acao=documento_visualizar&amp;id_documento={id}&amp;versao={n}&amp;infra_hash={}\">Ver</a></td></tr>",
                "a".repeat(64)
            );
        }
        format!(
            "<table class=\"infraTable\"><caption>Lista de Versões ({} registros):</caption><tr><th></th><th>Versão</th><th>Usuário</th><th>Unidade</th><th>Última Modificação</th><th>Ações</th></tr>{linhas}</table>",
            numeros.len()
        )
    }
    /// A forma servida nas sete instalações (`liberacoes-e-versoes-leitura-*`):
    /// `id_documento` duas vezes com o mesmo valor, `id_procedimento`, e nas
    /// versões anteriores a âncora "Recuperar Versão" (`href="#ID-N"`, link
    /// no `onclick`, nunca seguido).
    pub(crate) fn pagina_servida(id: &str, outro_id: &str, numeros: &[u32]) -> String {
        use std::fmt::Write as _;
        let hash = "d".repeat(64);
        let atual = numeros.iter().copied().max().unwrap_or(0);
        let mut linhas = String::new();
        for &n in numeros {
            let recuperar = if n == atual {
                String::new()
            } else {
                format!(
                    "<a href=\"#ID-{n}\" onclick=\"acaoRecuperar('{n}','N','controlador.php?acao=documento_versao_recuperar&versao={n}&id_documento={id}&infra_hash={hash}');\" tabindex=\"1006\"><img title=\"Recuperar Versão\" /></a>&nbsp;"
                )
            };
            let _ = write!(
                linhas,
                "<tr class=\"infraTrClara\"><td valign=\"center\"><a id=\"lnkInfraID-{n}\" name=\"ID-{n}\"></a><input class=\"infraCheckbox\" type=\"checkbox\" value=\"{n}\"/></td><td align=\"center\">{n}</td>\n<td align=\"center\"><a alt=\"Usuário Teste\" title=\"Usuário Teste\" class=\"ancoraSigla\">USUARIO</a></td>\n<td align=\"center\"><a alt=\"Unidade de Teste 1\" title=\"Unidade de Teste 1\" class=\"ancoraSigla\">TESTE</a></td><td align=\"center\">20/10/2026 14:07:0{n}</td><td align=\"center\"><a target=\"_blank\" onclick=\"infraLimparFormatarTrAcessada(this.parentNode.parentNode);\" href=\"controlador.php?acao=documento_visualizar&acao_origem=documento_versao_listar&acao_retorno=documento_versao_listar&versao={n}&id_documento={id}&arvore=1&id_procedimento=10&id_documento={outro_id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}\" tabindex=\"1003\"><img src=\"svg/documento_interno.svg?25\" title=\"Visualizar Versão\" /></a>&nbsp;{recuperar}</td></tr>\n"
            );
        }
        format!(
            "<html><body><form><div><table width=\"99%\" class=\"infraTable\" summary=\"Tabela de Versões.\">\n<caption class=\"infraCaption\">Lista de Versões ({} registros):</caption><tr><th class=\"infraTh\" width=\"1%\"><a href=\"javascript:void(0);\" id=\"lnkInfraCheck\"><img title=\"Selecionar Tudo\" alt=\"Selecionar Tudo\" class=\"infraImg\"/></a></th>\n<th class=\"infraTh\">Versão</th>\n<th class=\"infraTh\">Usuário</th>\n<th class=\"infraTh\">Unidade</th>\n<th class=\"infraTh\">Última Modificação</th>\n<th class=\"infraTh\">Ações</th>\n</tr>\n{linhas}</table></div><input type=\"hidden\" id=\"hdnInfraNroItens\" value=\"{}\"/></form></body></html>",
            numeros.len(),
            numeros.len()
        )
    }

    #[test]
    fn forma_servida_com_id_repetido_e_ancora_de_recuperar_e_lida() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let h =
            interpretar_historico(&pagina_servida("42", "42", &[2, 1]), &doc, "42", None).unwrap();
        assert_eq!(h.atual().unwrap().numero(), 2);
        assert_eq!(h.len(), 2);
        let v1 = h.selecionar(VersaoDocumento::nova(1).unwrap()).unwrap();
        assert!(v1.link_privado().contains("versao=1&"));
        assert!(!v1.link_privado().contains("documento_versao_recuperar"));
        assert_eq!(v1.usuario, "USUARIO");
        assert_eq!(v1.unidade, "TESTE");
    }

    #[test]
    fn id_repetido_com_valores_diferentes_e_recusado() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(
            interpretar_historico(&pagina_servida("42", "43", &[1]), &doc, "42", None),
            Err(ErroVersao::LinkInseguro)
        );
        // Uma âncora a mais que não é fragmento continua ambígua.
        let extra = pagina_servida("42", "42", &[1]).replace(
            "&nbsp;</td></tr>",
            "<a href=\"controlador.php?acao=outra\">x</a></td></tr>",
        );
        assert_eq!(
            interpretar_historico(&extra, &doc, "42", None),
            Err(ErroVersao::LinkVersaoAusenteOuAmbiguo)
        );
    }

    #[test]
    fn booleano_antes_de_class_e_href_nao_esconde_versoes() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let html = crate::html_atributos::com_booleano_antes_dos_atributos(&pagina(
            &[1, 4],
            &["42", "42"],
        ));
        let h = interpretar_historico(&html, &doc, "42", Some(2)).unwrap();
        assert_eq!(h.atual().unwrap().numero(), 4);
    }
    #[test]
    fn atual_e_nao_contiguas() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let h =
            interpretar_historico(&pagina(&[1, 4], &["42", "42"]), &doc, "42", Some(2)).unwrap();
        assert_eq!(h.atual().unwrap().numero(), 4);
        assert_eq!(
            h.selecionar(VersaoDocumento::nova(2).unwrap()),
            Err(ErroVersao::VersaoPedidaAusenteNoHistorico)
        );
    }
    #[test]
    fn recusa_duplicada_e_documento_trocado() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(
            interpretar_historico(&pagina(&[1, 1], &["42", "42"]), &doc, "42", None),
            Err(ErroVersao::VersaoDocumentoDuplicada)
        );
        assert_eq!(
            interpretar_historico(&pagina(&[1], &["43"]), &doc, "42", None),
            Err(ErroVersao::DocumentoDivergente)
        );
        assert_eq!(
            interpretar_historico(&pagina(&[1], &["42"]), &doc, "42", Some(2)),
            Err(ErroVersao::ListaIncompleta)
        );
    }
    #[test]
    fn numero_zero_e_link_com_acao_repetida_sao_recusados() {
        assert_eq!(
            VersaoDocumento::nova(0),
            Err(ErroVersao::VersaoDocumentoInvalida)
        );
        let doc = NumeroDocumento::novo("0007").unwrap();
        let html = pagina(&[1], &["42"]).replace(
            "acao=documento_visualizar",
            "acao=documento_visualizar&amp;acao=editor_montar",
        );
        assert_eq!(
            interpretar_historico(&html, &doc, "42", None),
            Err(ErroVersao::LinkInseguro)
        );
    }

    #[test]
    fn legendas_repetidas_nao_podem_confirmar_cobertura_de_versoes() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let pagina = pagina(&[1], &["42"]);
        let duplicada = pagina.replacen(
            "</caption>",
            "</caption><caption>Lista de Versões (1 registros):</caption>",
            1,
        );
        let extra = pagina.replacen(
            "<caption>Lista de Versões (1 registros):</caption>",
            "<caption>Outro inventário (1 registros):</caption><caption>Lista de Versões (1 registros):</caption>",
            1,
        );
        for html in [duplicada, extra] {
            assert_eq!(
                interpretar_historico(&html, &doc, "42", None),
                Err(ErroVersao::LegendasHistoricoVersoesRepetidas)
            );
        }
    }
}
