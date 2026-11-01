//! Histórico Web observado: nunca constrói links nem abre editor.
use crate::documento::NumeroDocumento;
use std::collections::HashSet;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersaoDocumento(NonZeroU32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroVersao {
    NumeroInvalido,
    HistoricoAusente,
    HistoricosDuplicados,
    LegendasDoHistoricoRepetidas,
    ColunasInesperadas,
    LinhaIncompleta,
    LinkAusenteOuAmbiguo,
    LinkInseguro,
    DocumentoDivergente,
    VersaoDuplicada,
    ListaIncompleta,
    VersaoNaoObservada,
}
impl ErroVersao {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::NumeroInvalido => "versao_documento_invalida",
            Self::HistoricoAusente => "historico_documento_ausente",
            Self::HistoricosDuplicados => "historicos_documento_duplicados",
            Self::LegendasDoHistoricoRepetidas => "legendas_historico_versoes_repetidas",
            Self::ColunasInesperadas => "colunas_historico_inesperadas",
            Self::LinhaIncompleta => "linha_historico_incompleta",
            Self::LinkAusenteOuAmbiguo => "link_versao_ausente_ou_ambiguo",
            Self::LinkInseguro => "link_versao_inseguro",
            Self::DocumentoDivergente => "identidade_documento_divergente",
            Self::VersaoDuplicada => "versao_documento_duplicada",
            Self::ListaIncompleta => "lista_versoes_incompleta",
            Self::VersaoNaoObservada => "versao_nao_observada",
        }
    }
}

impl VersaoDocumento {
    pub fn nova(numero: u32) -> Result<Self, ErroVersao> {
        NonZeroU32::new(numero)
            .map(Self)
            .ok_or(ErroVersao::NumeroInvalido)
    }
    pub fn interpretar(texto: &str) -> Result<Self, ErroVersao> {
        if texto.is_empty() || !texto.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroVersao::NumeroInvalido);
        }
        Self::nova(texto.parse().map_err(|_| ErroVersao::NumeroInvalido)?)
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
        if slot.replace(valor).is_some() {
            return Err(ErroVersao::LinkInseguro);
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
                return Err(ErroVersao::ColunasInesperadas);
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
            return Err(ErroVersao::VersaoDuplicada);
        }
        let td = celulas[5].as_tag().ok_or(ErroVersao::LinhaIncompleta)?;
        let mut links: Vec<_> = td
            .query_selector(parser, "a")
            .into_iter()
            .flatten()
            .filter_map(|h| h.get(parser))
            .filter_map(tl::Node::as_tag)
            .filter_map(|a| a.attributes().get("href").flatten())
            .map(|v| v.as_utf8_str().into_owned())
            .collect();
        if links.len() != 1 {
            return Err(ErroVersao::LinkAusenteOuAmbiguo);
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
        return Err(ErroVersao::ColunasInesperadas);
    }
    Ok(entradas)
}

/// Caption única é requisito de identidade e contagem da tabela Web.
fn legenda_unica(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<String, ErroVersao> {
    let mut legendas = tabela.query_selector(parser, "caption").into_iter().flatten();
    let primeira = legendas.next().ok_or(ErroVersao::ListaIncompleta)?;
    if legendas.next().is_some() {
        return Err(ErroVersao::LegendasDoHistoricoRepetidas);
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
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroVersao::HistoricoAusente)?;
    let parser = dom.parser();
    let mut tabela = None;
    for handle in dom
        .query_selector("table.infraTable")
        .ok_or(ErroVersao::HistoricoAusente)?
    {
        let no = handle.get(parser).ok_or(ErroVersao::HistoricoAusente)?;
        let tag = no.as_tag().ok_or(ErroVersao::HistoricoAusente)?;
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
                return Err(ErroVersao::HistoricosDuplicados);
            }
            tabela = Some(tag);
        }
    }
    let tabela = tabela.ok_or(ErroVersao::HistoricoAusente)?;
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
mod testes {
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
    #[test]
    fn atual_e_nao_contiguas() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let h =
            interpretar_historico(&pagina(&[1, 4], &["42", "42"]), &doc, "42", Some(2)).unwrap();
        assert_eq!(h.atual().unwrap().numero(), 4);
        assert_eq!(
            h.selecionar(VersaoDocumento::nova(2).unwrap()),
            Err(ErroVersao::VersaoNaoObservada)
        );
    }
    #[test]
    fn recusa_duplicada_e_documento_trocado() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(
            interpretar_historico(&pagina(&[1, 1], &["42", "42"]), &doc, "42", None),
            Err(ErroVersao::VersaoDuplicada)
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
        assert_eq!(VersaoDocumento::nova(0), Err(ErroVersao::NumeroInvalido));
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
                Err(ErroVersao::LegendasDoHistoricoRepetidas)
            );
        }
        assert_eq!(
            ErroVersao::LegendasDoHistoricoRepetidas.subcodigo(),
            "legendas_historico_versoes_repetidas"
        );
    }

}
