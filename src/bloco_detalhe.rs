//! Interpretação somente-leitura da tabela tblProtocolosBlocos.
//! Caso derivado de Todos `src/todos/sei/blocos_lista.py` @323eb01.
use crate::bloco::TipoBloco;
use crate::bloco_estado::EstadoDoBloco;
use crate::legenda::{ErroLegenda, Faixa, Legenda, ler_legenda};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct ItemDoBloco {
    pub(crate) processo: String,
    /// Para blocos internos/de reunião, o item é um processo, não documento.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) documento: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tipo: Option<String>,
    pub(crate) anotacoes: String,
}
#[derive(Debug, Serialize)]
pub(crate) struct BlocoDetalhado {
    pub(crate) numero: String,
    pub(crate) tipo: TipoBloco,
    /// O estado foi observado na lista que ofereceu o link do detalhe.
    pub(crate) estado: EstadoDoBloco,
    pub(crate) total_declarado: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) faixa_observada: Option<Faixa>,
    pub(crate) itens: Vec<ItemDoBloco>,
}
fn texto(tag: crate::tela_html::Elemento<'_>) -> String {
    tag.texto()
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// A tela de detalhe do bloco fora da forma observada: um erro por fato.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBlocoDetalhe {
    CabecalhoDoBlocoInvalido,
    FaixaDoBlocoInvalida,
    LegendaDoBlocoInvalida,
    LinhaDoBlocoInvalida,
    TabelaDoBlocoAusente,
    TabelasDoBlocoRepetidas,
    /// A legenda não está em nenhuma forma observada (leitor `legenda`).
    Legenda(#[from] ErroLegenda),
}

/// A marca que o SEI publica para um bloco sem registros, no lugar da tabela.
const BLOCO_VAZIO: &str = "Nenhum registro encontrado.";

/// Verdadeiro só para a forma observada: a área da tabela tem como único
/// conteúdo um rótulo com a marca de vazio, e a tabela não existe.
fn bloco_vazio(dom: &crate::tela_html::Documento) -> bool {
    let areas = dom.com_id("divInfraAreaTabela").collect::<Vec<_>>();
    let [area] = areas.as_slice() else {
        return false;
    };
    let area = *area;
    let mut rotulos = area.elementos("label");
    let rotulo = rotulos.next();
    rotulos.next().is_none()
        && rotulo.is_some_and(|r| texto(r) == BLOCO_VAZIO)
        && texto(area) == BLOCO_VAZIO
}

fn faixa(legenda: &str, linhas: usize) -> Result<(usize, Option<Faixa>), ErroBlocoDetalhe> {
    let Legenda { total, faixa } = ler_legenda(legenda)?;
    let Some(Faixa {
        inicio: de,
        fim: ate,
    }) = faixa
    else {
        if total != linhas {
            return Err(ErroBlocoDetalhe::FaixaDoBlocoInvalida);
        }
        return Ok((
            total,
            (total > 0).then_some(Faixa {
                inicio: 1,
                fim: total,
            }),
        ));
    };
    if de == 0 || ate < de || ate > total || ate - de + 1 != linhas {
        return Err(ErroBlocoDetalhe::FaixaDoBlocoInvalida);
    }
    Ok((
        total,
        Some(Faixa {
            inicio: de,
            fim: ate,
        }),
    ))
}

fn ler_itens(
    tabela: crate::tela_html::Elemento<'_>,
    headers: &[String],
    tipo: TipoBloco,
) -> Result<Vec<ItemDoBloco>, ErroBlocoDetalhe> {
    let index = |nome: &str| headers.iter().position(|c| c == nome);
    let processo = index("Processo").ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
    let anotacoes = index("Anotações").ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
    let documento = index("Documento");
    let tipo_coluna = index("Tipo");
    let mut itens = Vec::new();
    for no in tabela.elementos("tr") {
        let td: Vec<_> = no.filhos().filter(|t| t.nome() == "td").collect();
        if td.is_empty() {
            continue;
        }
        if td.len() != headers.len() {
            return Err(ErroBlocoDetalhe::LinhaDoBlocoInvalida);
        }
        let celula = |i: usize| {
            td.get(i)
                .copied()
                .ok_or(ErroBlocoDetalhe::LinhaDoBlocoInvalida)
        };
        let processo = texto(celula(processo)?);
        if processo.is_empty() {
            return Err(ErroBlocoDetalhe::LinhaDoBlocoInvalida);
        }
        let doc = documento
            .map(|i| celula(i).map(texto))
            .transpose()?
            .filter(|s| !s.is_empty());
        if tipo == TipoBloco::Assinatura && doc.is_none() {
            return Err(ErroBlocoDetalhe::LinhaDoBlocoInvalida);
        }
        itens.push(ItemDoBloco {
            processo,
            documento: doc,
            tipo: tipo_coluna
                .map(|i| celula(i).map(texto))
                .transpose()?
                .filter(|s| !s.is_empty()),
            anotacoes: texto(celula(anotacoes)?),
        });
    }
    Ok(itens)
}

pub(crate) fn ler_detalhe(
    html: &crate::tela_html::HtmlServido,
    numero: &str,
    tipo: TipoBloco,
    estado: EstadoDoBloco,
) -> Result<BlocoDetalhado, ErroBlocoDetalhe> {
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblProtocolosBlocos").collect::<Vec<_>>();
    if tabelas.is_empty() && bloco_vazio(&dom) {
        return Ok(BlocoDetalhado {
            numero: numero.to_owned(),
            tipo,
            estado,
            total_declarado: 0,
            faixa_observada: None,
            itens: Vec::new(),
        });
    }
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroBlocoDetalhe::TabelaDoBlocoAusente),
        [no] => *no,
        _ => return Err(ErroBlocoDetalhe::TabelasDoBlocoRepetidas),
    };
    if tabela.nome() != "table" {
        return Err(ErroBlocoDetalhe::TabelaDoBlocoAusente);
    }
    let mut captions = tabela.elementos("caption");
    let caption = captions
        .next()
        .ok_or(ErroBlocoDetalhe::LegendaDoBlocoInvalida)?;
    if captions.next().is_some() {
        return Err(ErroBlocoDetalhe::LegendaDoBlocoInvalida);
    }
    let mut headers = None;
    for no in tabela.elementos("tr") {
        let th: Vec<_> = no.filhos().filter(|t| t.nome() == "th").collect();
        if !th.is_empty() {
            if headers.is_some() {
                return Err(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido);
            }
            headers = Some(th.iter().copied().map(texto).collect::<Vec<_>>());
        }
    }
    let headers = headers.ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
    for campo in ["Processo", "Anotações"] {
        if headers.iter().filter(|h| h.as_str() == campo).count() != 1 {
            return Err(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido);
        }
    }
    if tipo == TipoBloco::Assinatura
        && headers.iter().filter(|h| h.as_str() == "Documento").count() != 1
    {
        return Err(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido);
    }
    let itens = ler_itens(tabela, &headers, tipo)?;
    let (total_declarado, faixa_observada) = faixa(&texto(caption), itens.len())?;
    Ok(BlocoDetalhado {
        numero: numero.to_owned(),
        tipo,
        estado,
        total_declarado,
        faixa_observada,
        itens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn documentos_do_bloco_sao_lidos_pelas_colunas_nao_pela_posicao() {
        // Caso de Todos tests/paginas_reais/bloco_com_documentos.html @323eb01.
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProtocolosBlocos'><caption>Lista de Processos/Documentos (1 registros):</caption><tr><th></th><th>Documento</th><th>Processo</th><th>Anotações</th><th>Tipo</th></tr><tr><td><input type='checkbox'/></td><td>0001234</td><td>0000.00/2026-01</td><td>Observação</td><td>Despacho</td></tr></table>",
        );
        let d = ler_detalhe(&html, "91", TipoBloco::Assinatura, EstadoDoBloco::Gerado).unwrap();
        assert_eq!(d.total_declarado, 1);
        assert_eq!(d.itens[0].documento.as_deref(), Some("0001234"));
        assert_eq!(d.itens[0].anotacoes, "Observação");
    }
    #[test]
    fn interno_nao_inventa_documento_ou_tipo() {
        // Caso de Todos tests/test_bloco_de_processos_010.py @323eb01.
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProtocolosBlocos'><caption>Lista (1 registros):</caption><tr><th></th><th>Processo</th><th>Anotações</th></tr><tr><td><input type='checkbox'/></td><td>NUP 001</td><td></td></tr></table>",
        );
        let d = ler_detalhe(&html, "91", TipoBloco::Interno, EstadoDoBloco::Gerado).unwrap();
        assert!(d.itens[0].documento.is_none());
        assert!(d.itens[0].tipo.is_none());
    }
    #[test]
    fn detalhe_truncado_recusado() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProtocolosBlocos'><caption>Lista (9 registros):</caption><tr><th>Processo</th><th>Anotações</th></tr><tr><td>NUP</td><td></td></tr></table>",
        );
        assert_eq!(
            ler_detalhe(&html, "91", TipoBloco::Reuniao, EstadoDoBloco::Gerado).err(),
            Some(ErroBlocoDetalhe::FaixaDoBlocoInvalida)
        );
    }
    #[test]
    fn legenda_no_singular_com_um_item_e_lida() {
        // Forma observada: "(1 registro):" com a única linha da tabela.
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProtocolosBlocos'><caption class='infraCaption'>Lista de Processos/Documentos (1 registro):</caption><tr><th></th><th>Documento</th><th>Processo</th><th>Anotações</th></tr><tr><td><input type='checkbox'/></td><td>0001234</td><td>0000.00/2026-01</td><td></td></tr></table>",
        );
        let d = ler_detalhe(&html, "91", TipoBloco::Assinatura, EstadoDoBloco::Gerado).unwrap();
        assert_eq!(d.total_declarado, 1);
        assert_eq!(d.faixa_observada, Some(Faixa { inicio: 1, fim: 1 }));
        assert_eq!(d.itens.len(), 1);
        assert_eq!(d.itens[0].documento.as_deref(), Some("0001234"));
    }
    #[test]
    fn legenda_no_singular_fora_da_forma_e_recusada() {
        let linha = "<tr><th>Processo</th><th>Anotações</th></tr><tr><td>NUP</td><td></td></tr>";
        let dois = crate::tela_html::HtmlServido::de_teste(&format!(
            "<table id='tblProtocolosBlocos'><caption>Lista (2 registro):</caption>{linha}</table>"
        ));
        assert_eq!(
            ler_detalhe(&dois, "91", TipoBloco::Reuniao, EstadoDoBloco::Gerado).err(),
            Some(ErroBlocoDetalhe::Legenda(
                crate::legenda::ErroLegenda::SingularComTotalDiferenteDeUm { total: 2 }
            ))
        );
        let sem_linha = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProtocolosBlocos'><caption>Lista (1 registro):</caption><tr><th>Processo</th><th>Anotações</th></tr></table>",
        );
        assert_eq!(
            ler_detalhe(&sem_linha, "91", TipoBloco::Reuniao, EstadoDoBloco::Gerado).err(),
            Some(ErroBlocoDetalhe::FaixaDoBlocoInvalida)
        );
    }
    #[test]
    fn bloco_vazio_com_a_marca_observada_e_lido_sem_itens() {
        // Forma observada: a área da tabela traz só o rótulo de vazio.
        let html = crate::tela_html::HtmlServido::de_teste(
            "<div id='divInfraAreaTabela' class='infraAreaTabela'><label class='infraLabelTabelaNenhumRegistro'>Nenhum registro encontrado.</label></div><input type='hidden' id='hdnInfraNroItens' name='hdnInfraNroItens' value='0'/>",
        );
        let d = ler_detalhe(&html, "91", TipoBloco::Assinatura, EstadoDoBloco::Gerado).unwrap();
        assert_eq!(d.total_declarado, 0);
        assert_eq!(d.faixa_observada, None);
        assert!(d.itens.is_empty());
        let sem_classe = crate::tela_html::HtmlServido::de_teste(
            "<div id='divInfraAreaTabela' class='infraAreaTabela'><label>Nenhum registro encontrado.</label></div>",
        );
        let d = ler_detalhe(&sem_classe, "91", TipoBloco::Interno, EstadoDoBloco::Gerado).unwrap();
        assert!(d.itens.is_empty());
    }
    #[test]
    fn ausencia_de_tabela_sem_a_marca_de_vazio_continua_recusada() {
        let outra_mensagem = crate::tela_html::HtmlServido::de_teste(
            "<div id='divInfraAreaTabela' class='infraAreaTabela'><label>Outra mensagem.</label></div>",
        );
        assert_eq!(
            ler_detalhe(
                &outra_mensagem,
                "91",
                TipoBloco::Assinatura,
                EstadoDoBloco::Gerado
            )
            .err(),
            Some(ErroBlocoDetalhe::TabelaDoBlocoAusente)
        );
        let marca_fora_da_area = crate::tela_html::HtmlServido::de_teste(
            "<div><label>Nenhum registro encontrado.</label></div>",
        );
        assert_eq!(
            ler_detalhe(
                &marca_fora_da_area,
                "91",
                TipoBloco::Assinatura,
                EstadoDoBloco::Gerado
            )
            .err(),
            Some(ErroBlocoDetalhe::TabelaDoBlocoAusente)
        );
        let marca_com_conteudo_extra = crate::tela_html::HtmlServido::de_teste(
            "<div id='divInfraAreaTabela'><label>Nenhum registro encontrado.</label><label>Outro</label></div>",
        );
        assert_eq!(
            ler_detalhe(
                &marca_com_conteudo_extra,
                "91",
                TipoBloco::Assinatura,
                EstadoDoBloco::Gerado
            )
            .err(),
            Some(ErroBlocoDetalhe::TabelaDoBlocoAusente)
        );
    }
}
