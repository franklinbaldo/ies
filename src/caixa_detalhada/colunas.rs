//! Cabeçalho e células configuráveis da tabela D, lidos pelas chaves
//! estáveis de [`ColunaDoDetalhe`]. Cabeçalho desconhecido é recusa: uma
//! mudança de layout não pode embaralhar campos.
use super::ErroDetalhada;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_por_elemento_e_classe, valor_do_atributo};
use crate::nivel_de_detalhe::{CabecalhoDaColuna, ColunaDoDetalhe, texto_colapsado};
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};

/// Papel de uma coluna da tabela D, pelo rótulo do cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PapelDaColuna {
    Processo,
    Configurada(ColunaDoDetalhe),
    /// Seleção, ícones de estado e ações: cabeçalho sem rótulo.
    SemRotulo,
}

/// Valor observado numa célula configurável.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(crate) enum ValorDaColuna {
    /// Texto com espaços colapsados; `None` se a célula está vazia (a chave sai do JSON).
    Texto(Option<String>),
    /// Itens separados pelo HTML (`div.divRotuloItemCelula`).
    Itens(Vec<String>),
    /// Um objeto por `a.ancMarcador`; célula vazia é `[]`.
    Marcadores(Vec<super::celula_marcadores::MarcadorDaCelula>),
}

/// Colunas configuradas observadas numa linha, na ordem do cabeçalho.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ColunasDaLinha(pub(crate) Vec<(ColunaDoDetalhe, ValorDaColuna)>);

impl Serialize for ColunasDaLinha {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // A célula de texto vazia é omitida: a saída não traz null.
        let lidas: Vec<_> = self
            .0
            .iter()
            .filter(|(_, valor)| !matches!(valor, ValorDaColuna::Texto(None)))
            .collect();
        let mut mapa = serializer.serialize_map(Some(lidas.len()))?;
        for (coluna, valor) in lidas {
            mapa.serialize_entry(coluna.chave(), valor)?;
        }
        mapa.end()
    }
}

/// Texto visível com entidades conhecidas decodificadas e espaços colapsados.
fn texto(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    let bruto = tag.inner_text(parser);
    let decodificado = decodificar_atributo(&bruto).unwrap_or_else(|| bruto.into_owned());
    texto_colapsado(&decodificado)
}

fn tags<'d>(
    dom: &'d tl::VDom<'d>,
    raiz: &tl::HTMLTag<'_>,
    elemento: &str,
    classe: &str,
) -> Result<Vec<&'d tl::HTMLTag<'d>>, ErroDetalhada> {
    let parser = dom.parser();
    Ok(nos_por_elemento_e_classe(dom, Some(raiz), elemento, classe)
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?
        .into_iter()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
        .collect())
}

/// Rótulo de um cabeçalho só de ícone: `aria-label` (SEI 5.0.x) ou o
/// argumento de `return infraTooltipMostrar('…');` (`UnB` 4.0.9); os dois,
/// quando presentes, têm de coincidir.
fn rotulo_do_icone(
    rotulo: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<Option<String>, ErroDetalhada> {
    let imagens: Vec<_> = rotulo
        .query_selector(parser, "img")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
        .collect();
    let [imagem] = imagens.as_slice() else {
        return Ok(None);
    };
    let aria = valor_do_atributo(imagem, "aria-label")
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?
        .map(|v| texto_colapsado(&decodificar_atributo(&v).unwrap_or(v)));
    let dica = valor_do_atributo(imagem, "onmouseover")
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?
        .and_then(|v| decodificar_atributo(&v))
        .and_then(|v| {
            let resto = v.strip_prefix("return infraTooltipMostrar('")?;
            let resto = resto.strip_suffix(';').unwrap_or(resto);
            Some(texto_colapsado(resto.strip_suffix("')")?))
        });
    Ok(match (aria, dica) {
        (Some(a), Some(d)) if a != d => None,
        (Some(a), _) => Some(a),
        (None, d) => d,
    })
}

fn papel_do_rotulo(rotulo: &str) -> Result<PapelDaColuna, ErroDetalhada> {
    if rotulo == "Processo" {
        return Ok(PapelDaColuna::Processo);
    }
    ColunaDoDetalhe::pelo_cabecalho(rotulo)
        .map(PapelDaColuna::Configurada)
        .ok_or(ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido)
}

fn papel(dom: &tl::VDom<'_>, th: &tl::HTMLTag<'_>) -> Result<PapelDaColuna, ErroDetalhada> {
    let parser = dom.parser();
    match tags(dom, th, "div", "infraDivRotuloOrdenacao")?.as_slice() {
        [] => {
            let rotulo = texto(th, parser);
            if rotulo.is_empty() {
                Ok(PapelDaColuna::SemRotulo)
            } else {
                papel_do_rotulo(&rotulo)
            }
        }
        [div] => {
            let rotulo = texto(div, parser);
            if !rotulo.is_empty() {
                return papel_do_rotulo(&rotulo);
            }
            rotulo_do_icone(div, parser)?
                .ok_or(ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido)
                .and_then(|rotulo| papel_do_rotulo(&rotulo))
        }
        _ => Err(ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido),
    }
}

/// Rótulo e papel de cada `th`, na ordem do documento: Processo único,
/// cada coluna configurada no máximo uma vez.
pub(super) fn papeis_do_cabecalho(
    dom: &tl::VDom<'_>,
    tabela: &tl::HTMLTag<'_>,
) -> Result<Vec<PapelDaColuna>, ErroDetalhada> {
    let parser = dom.parser();
    let mut papeis = Vec::new();
    for th in tabela
        .query_selector(parser, "th")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
    {
        let novo = papel(dom, th)?;
        if novo != PapelDaColuna::SemRotulo && papeis.contains(&novo) {
            return Err(if novo == PapelDaColuna::Processo {
                ErroDetalhada::CabecalhoProcessoDetalhadoRepetido
            } else {
                ErroDetalhada::ColunaDaVisaoDetalhadaRepetida
            });
        }
        papeis.push(novo);
    }
    if !papeis.contains(&PapelDaColuna::Processo) {
        return Err(ErroDetalhada::CabecalhoProcessoDetalhadoAusente);
    }
    Ok(papeis)
}

/// Valor da célula `td` da coluna configurada; `data-label`, quando
/// publicado (SEI 5.0.x), precisa ser o cabeçalho da coluna.
pub(super) fn valor_da_celula(
    dom: &tl::VDom<'_>,
    celula: &tl::HTMLTag<'_>,
    coluna: ColunaDoDetalhe,
    id_procedimento: &str,
) -> Result<ValorDaColuna, ErroDetalhada> {
    let parser = dom.parser();
    let rotulo = valor_do_atributo(celula, "data-label")
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?
        .map(|v| texto_colapsado(&decodificar_atributo(&v).unwrap_or(v)))
        .filter(|v| !v.is_empty());
    let (CabecalhoDaColuna::Texto(cabecalho) | CabecalhoDaColuna::Icone(cabecalho)) =
        coluna.cabecalho();
    if rotulo.is_some_and(|r| r != cabecalho) {
        return Err(ErroDetalhada::RotuloDaCelulaDiferenteDoCabecalho);
    }
    if coluna == ColunaDoDetalhe::Marcadores {
        return super::celula_marcadores::marcadores_da_celula(celula, parser, id_procedimento)
            .map(ValorDaColuna::Marcadores);
    }
    let completo = texto(celula, parser);
    if coluna.tem_itens() {
        let itens: Vec<String> = tags(dom, celula, "div", "divRotuloItemCelula")?
            .into_iter()
            .map(|div| texto(div, parser))
            .filter(|item| !item.is_empty())
            .collect();
        // Os itens podem vir colados (sem espaço entre os `div`): compara sem espaços.
        let sem_espacos = |texto: &str| texto.split_whitespace().collect::<String>();
        if sem_espacos(&itens.concat()) != sem_espacos(&completo) {
            return Err(ErroDetalhada::ItensDaCelulaNaoSeparados);
        }
        return Ok(ValorDaColuna::Itens(itens));
    }
    if !tags(dom, celula, "div", "divItemCelula")?.is_empty() {
        return Err(ErroDetalhada::ItensNaoPrevistosNaCelulaDetalhada);
    }
    Ok(ValorDaColuna::Texto(
        (!completo.is_empty()).then_some(completo),
    ))
}
