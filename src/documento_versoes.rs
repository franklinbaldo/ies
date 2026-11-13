//! Histórico Web observado: nunca constrói links nem abre editor.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::legenda::ler_legenda;
use std::collections::HashSet;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersaoDocumento(NonZeroU32);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroVersao {
    Legenda(#[from] crate::legenda::ErroLegenda),
    LinhaIncompleta,
    /// O número da versão não cabe em u32; guarda o tipo do erro de conversão.
    NumeroDaVersaoIlegivel(std::num::IntErrorKind),
    LinkInseguro,
    /// A consulta do link não se deixa ler em pares.
    ConsultaDoLinkIlegivel(#[source] crate::consulta_do_controlador::ErroConsultaDoControlador),
    /// A `acao` do link ausente ou repetida.
    AcaoDoLinkNaoLida(#[source] crate::consulta_do_controlador::ErroAcaoDaUrl),
    /// A `versao` do link não é única.
    VersaoDoLinkNaoLida(#[source] crate::consulta_do_controlador::ErroParametroDaUrl),
    /// O `infra_hash` do link não é único.
    HashDoLinkNaoLido(#[source] crate::consulta_do_controlador::ErroParametroDaUrl),
    DocumentoDivergente,
    ListaIncompleta,
    VersaoPedidaAusenteNoHistorico,
    ColunasHistoricoInesperadas,
    HistoricoDocumentoAusente,
    HistoricosDocumentoDuplicados,
    LegendasHistoricoVersoesRepetidas,
    LinkVersaoAusenteOuAmbiguo,
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
    /// A versão `N + 1`; não existe depois do maior `u32`.
    pub fn seguinte(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
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

fn texto(no: crate::tela_html::Elemento<'_>) -> String {
    no.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

// Não normalizar chaves percent-encoded, pontos ou campos repetidos:
// o PHP interpreta essas formas de modo potencialmente ambígua.
fn conferir_link(href: &str, id: &str, versao: VersaoDocumento) -> Result<(), ErroVersao> {
    let query = href
        .strip_prefix("controlador.php?")
        .ok_or(ErroVersao::LinkInseguro)?;
    if query.bytes().any(|b| b <= 32 || b >= 127) {
        return Err(ErroVersao::LinkInseguro);
    }
    let pares = ConsultaDoControlador::ler(query).map_err(ErroVersao::ConsultaDoLinkIlegivel)?;
    let acao = pares.acao().map_err(ErroVersao::AcaoDoLinkNaoLida)?;
    let versao_observada = pares
        .unico("versao")
        .map_err(ErroVersao::VersaoDoLinkNaoLida)?;
    // `id_documento` servido duas vezes com o mesmo valor (sete
    // instalações); valores diferentes: recusa.
    let ids: Vec<&str> = pares.valores("id_documento").collect();
    let id_observado = match ids.as_slice() {
        [] => None,
        [primeiro, resto @ ..] if resto.iter().all(|v| v == primeiro) => Some(*primeiro),
        _ => return Err(ErroVersao::LinkInseguro),
    };
    let hash = pares
        .unico("infra_hash")
        .map_err(ErroVersao::HashDoLinkNaoLido)?;
    if acao != AcaoDoSei::DocumentoVisualizar.como_na_url()
        || versao_observada != versao.numero().to_string().as_str()
    {
        return Err(ErroVersao::LinkInseguro);
    }
    if id_observado != Some(id) {
        return Err(ErroVersao::DocumentoDivergente);
    }
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ErroVersao::LinkInseguro);
    }
    Ok(())
}

fn linhas_do_historico(
    tabela: crate::tela_html::Elemento<'_>,
    id_interno: &str,
) -> Result<Vec<EntradaVersao>, ErroVersao> {
    let mut cabecalho = false;
    let mut entradas = Vec::new();
    let mut encontrados = HashSet::new();
    for tr_handle in tabela.elementos("tr") {
        let cols: Vec<_> = tr_handle.elementos("th").collect();
        if !cols.is_empty() {
            let titulos: Vec<_> = cols.iter().copied().map(texto).collect();
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
        let celulas: Vec<_> = tr_handle.elementos("td").collect();
        if celulas.is_empty() {
            continue;
        }
        let (true, [_, versao, usuario, unidade, modificacao, acoes]) =
            (cabecalho, celulas.as_slice())
        else {
            return Err(ErroVersao::LinhaIncompleta);
        };
        let numero = VersaoDocumento::interpretar(&texto(*versao))?;
        if !encontrados.insert(numero) {
            return Err(ErroVersao::VersaoDocumentoDuplicada);
        }
        let mut links = Vec::new();
        for a in acoes.elementos("a") {
            // `href` lido da fonte da tag; tag ilegível não some da contagem.
            // Âncora de fragmento (`#ID-N`, "Recuperar Versão" com o link no
            // `onclick`) não é link de versão e nunca é seguida.
            let href = a.atributo("href").map(str::to_owned);
            links.extend(href.filter(|h| !h.starts_with('#')));
        }
        let Ok([link]) = <[String; 1]>::try_from(links) else {
            return Err(ErroVersao::LinkVersaoAusenteOuAmbiguo);
        };
        conferir_link(&link, id_interno, numero)?;
        let [usuario, unidade, modificacao] = [*usuario, *unidade, *modificacao].map(texto);
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
fn legenda_unica(tabela: crate::tela_html::Elemento<'_>) -> Result<String, ErroVersao> {
    let mut legendas = tabela.elementos("caption");
    let primeira = legendas.next().ok_or(ErroVersao::ListaIncompleta)?;
    if legendas.next().is_some() {
        return Err(ErroVersao::LegendasHistoricoVersoesRepetidas);
    }
    Ok(texto(primeira))
}

/// Interpreta uma lista observada. O total declarado é facultativo, mas, quando
/// disponível na tabela de origem, é obrigatório conferir a cobertura.
pub fn interpretar_historico(
    html: &crate::tela_html::HtmlServido,
    id_interno: &str,
    total_declarado: Option<usize>,
) -> Result<HistoricoDocumento, ErroVersao> {
    if id_interno.is_empty() || !id_interno.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroVersao::DocumentoDivergente);
    }
    let dom = crate::tela_html::arvore(html);
    let mut tabela = None;
    for handle in dom
        .elementos("table")
        .filter(|elemento| elemento.tem_classe("infraTable"))
    {
        let captions: Vec<_> = handle.elementos("caption").collect();
        if captions.iter().any(|c| texto(*c).contains("Versões")) {
            if tabela.is_some() {
                return Err(ErroVersao::HistoricosDocumentoDuplicados);
            }
            tabela = Some(handle);
        }
    }
    let tabela = tabela.ok_or(ErroVersao::HistoricoDocumentoAusente)?;
    let legenda = legenda_unica(tabela)?;
    let declaradas = ler_legenda(&legenda)?.total;
    let entradas = linhas_do_historico(tabela, id_interno)?;
    if declaradas != entradas.len() || total_declarado.is_some_and(|n| n != declaradas) {
        return Err(ErroVersao::ListaIncompleta);
    }
    Ok(HistoricoDocumento { entradas })
}

#[cfg(test)]
#[path = "documento_versoes/tests_anti_ies.rs"]
mod testes_anti_ies;

#[cfg(test)]
pub(crate) mod testes {
    use super::*;
    use crate::tela_html::HtmlServido;
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
        let h = interpretar_historico(
            &HtmlServido::de_teste(&pagina_servida("42", "42", &[2, 1])),
            "42",
            None,
        )
        .unwrap();
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
        assert_eq!(
            interpretar_historico(
                &HtmlServido::de_teste(&pagina_servida("42", "43", &[1])),
                "42",
                None
            ),
            Err(ErroVersao::LinkInseguro)
        );
        // Uma âncora a mais que não é fragmento continua ambígua.
        let extra = HtmlServido::de_teste(&pagina_servida("42", "42", &[1]).replace(
            "&nbsp;</td></tr>",
            "<a href=\"controlador.php?acao=outra\">x</a></td></tr>",
        ));
        assert_eq!(
            interpretar_historico(&extra, "42", None),
            Err(ErroVersao::LinkVersaoAusenteOuAmbiguo)
        );
    }

    #[test]
    fn booleano_antes_de_class_e_href_nao_esconde_versoes() {
        let html =
            HtmlServido::de_teste(&crate::tela_html::teste::com_booleano_antes_dos_atributos(
                &pagina(&[1, 4], &["42", "42"]),
            ));
        let h = interpretar_historico(&html, "42", Some(2)).unwrap();
        assert_eq!(h.atual().unwrap().numero(), 4);
    }
    #[test]
    fn atual_e_nao_contiguas() {
        let h = interpretar_historico(
            &HtmlServido::de_teste(&pagina(&[1, 4], &["42", "42"])),
            "42",
            Some(2),
        )
        .unwrap();
        assert_eq!(h.atual().unwrap().numero(), 4);
        assert_eq!(
            h.selecionar(VersaoDocumento::nova(2).unwrap()),
            Err(ErroVersao::VersaoPedidaAusenteNoHistorico)
        );
    }
    #[test]
    fn recusa_duplicada_e_documento_trocado() {
        assert_eq!(
            interpretar_historico(
                &HtmlServido::de_teste(&pagina(&[1, 1], &["42", "42"])),
                "42",
                None
            ),
            Err(ErroVersao::VersaoDocumentoDuplicada)
        );
        assert_eq!(
            interpretar_historico(&HtmlServido::de_teste(&pagina(&[1], &["43"])), "42", None),
            Err(ErroVersao::DocumentoDivergente)
        );
        assert_eq!(
            interpretar_historico(
                &HtmlServido::de_teste(&pagina(&[1], &["42"])),
                "42",
                Some(2)
            ),
            Err(ErroVersao::ListaIncompleta)
        );
    }
    #[test]
    fn numero_zero_e_link_com_acao_repetida_sao_recusados() {
        assert_eq!(
            VersaoDocumento::nova(0),
            Err(ErroVersao::VersaoDocumentoInvalida)
        );
        let html = HtmlServido::de_teste(&pagina(&[1], &["42"]).replace(
            "acao=documento_visualizar",
            "acao=documento_visualizar&amp;acao=editor_montar",
        ));
        assert_eq!(
            interpretar_historico(&html, "42", None),
            Err(ErroVersao::AcaoDoLinkNaoLida(
                crate::consulta_do_controlador::ErroAcaoDaUrl::AcaoRepetida
            ))
        );
    }

    #[test]
    fn par_sem_igual_no_link_da_versao_e_recusado_com_a_causa() {
        let html = HtmlServido::de_teste(&pagina(&[1], &["42"]).replace(
            "acao=documento_visualizar",
            "acao=documento_visualizar&amp;solto",
        ));
        assert_eq!(
            interpretar_historico(&html, "42", None),
            Err(ErroVersao::ConsultaDoLinkIlegivel(
                crate::consulta_do_controlador::ErroConsultaDoControlador::ParSemIgual
            ))
        );
    }

    #[test]
    fn legendas_repetidas_nao_podem_confirmar_cobertura_de_versoes() {
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
                interpretar_historico(&HtmlServido::de_teste(&html), "42", None),
                Err(ErroVersao::LegendasHistoricoVersoesRepetidas)
            );
        }
    }
}
