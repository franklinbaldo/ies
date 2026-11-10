//! Resposta 200 do POST da pesquisa por protocolo: tabela com barra de
//! contagem, aviso de "sem resultado" ou nenhuma forma reconhecida.
//! Formas observadas na `UnB` 4.0.9 de treinamento em 03/11/2026 (capturas
//! privadas, só estrutura): `div.pesquisaBarraD` "Exibindo 1 - 10 de 1.234"
//! antes de `table.pesquisaResultado`; `div.pesquisaSemResultado` sozinho.
use super::{ErroResultadoPesquisa, ResultadoPesquisa, ler_resultados};
use crate::dominio::IdProcedimento;
use crate::erro_html::ErroHtml;
use crate::html_atributos::{ErroAtributoHtml, nos_por_elemento_e_classe};
use crate::transporte::texto_de_recusa::{decodificar_entidades, texto_visivel};

/// Posições `inicio..=fim` (base 1) declaradas pela barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FaixaDaPesquisa {
    pub(crate) inicio: u64,
    pub(crate) fim: u64,
}

/// Uma página da lista: só números observados, total e faixa da barra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListaDaPesquisa {
    pub(crate) processos: Vec<ResultadoPesquisa>,
    pub(crate) total_declarado: u64,
    pub(crate) faixa: FaixaDaPesquisa,
    /// Posições da faixa da barra sem linha na tabela (Goiás 5.0.3).
    pub(crate) linhas_ausentes_da_faixa: u64,
}

/// As três respostas observadas do envio da pesquisa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RespostaDaPesquisa {
    /// 302 interpretado (nunca seguido): só o identificador interno.
    ProcessoUnico {
        id: IdProcedimento,
    },
    Lista(ListaDaPesquisa),
    /// `div.pesquisaSemResultado`: nada para esta conta/unidade; não é
    /// prova de inexistência do processo.
    SemResultado,
}

/// Uma variante por fato verificável da resposta 200.
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroRespostaDaPesquisa {
    RespostaDaPesquisaNaoReconhecida,
    TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta,
    /// Aviso de "sem resultado" e barra "Exibindo I - F de N" juntos.
    AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta,
    TextoDaBarraDeContagemForaDoFormato,
    NumeroDaBarraDeContagemForaDoFormato,
    FaixaDaBarraDeContagemImpossivel,
    LinhasDaTabelaDiferentesDaFaixaDaBarra,
    #[causa(interna)]
    Resultado(ErroResultadoPesquisa),
    /// Linha da pesquisa em documentos (`ies documento buscar`).
    #[causa(interna)]
    Documento(super::documentos::ErroDocumentoDaPesquisa),
    #[causa(interna)]
    AtributoHtml(ErroAtributoHtml),
    #[causa(transparente)]
    Html(ErroHtml),
}

/// Número da barra: grupos de 3 dígitos separados por `.` (o primeiro com
/// 1 a 3, sem zero à esquerda); qualquer outra forma é `None`.
pub(crate) fn ler_contagem(texto: &str) -> Option<u64> {
    let digitos = |grupo: &str| !grupo.is_empty() && grupo.bytes().all(|b| b.is_ascii_digit());
    let mut grupos = texto.split('.');
    let primeiro = grupos.next()?;
    if !digitos(primeiro) || primeiro.len() > 3 || (primeiro.len() > 1 && primeiro.starts_with('0'))
    {
        return None;
    }
    let mut valor: u64 = primeiro.parse().ok()?;
    for grupo in grupos {
        if grupo.len() != 3 || !digitos(grupo) {
            return None;
        }
        valor = valor
            .checked_mul(1000)?
            .checked_add(grupo.parse::<u64>().ok()?)?;
    }
    Some(valor)
}

/// "Exibindo I - F de N", com espaços colapsados; nada antes ou depois.
/// "N resultados" (sem faixa) volta como `Ok((None, N))`.
fn ler_barra(texto: &str) -> Result<(Option<FaixaDaPesquisa>, u64), ErroRespostaDaPesquisa> {
    use ErroRespostaDaPesquisa as E;
    let palavras: Vec<&str> = texto.split_whitespace().collect();
    let numero = |t: &str| ler_contagem(t).ok_or(E::NumeroDaBarraDeContagemForaDoFormato);
    // "Exibindo I - F de N" pagina; "N resultados" é a lista que cabe numa
    // página, sem `navegar()` (`UnB`, 2 e 9 resultados), ou a última página
    // na PEN3 5.0.4 (`pesquisa-paginas-pen3`, passo 9). O singular não foi
    // observado e continua fora do formato.
    let (inicio, fim, total) = match palavras.as_slice() {
        ["Exibindo", inicio, "-", fim, "de", total] => {
            (numero(inicio)?, numero(fim)?, numero(total)?)
        }
        [total, "resultados"] => return Ok((None, numero(total)?)),
        _ => return Err(E::TextoDaBarraDeContagemForaDoFormato),
    };
    if inicio == 0 || inicio > fim || fim > total {
        return Err(E::FaixaDaBarraDeContagemImpossivel);
    }
    Ok((Some(FaixaDaPesquisa { inicio, fim }), total))
}

/// A página `numero` marcada no paginador (`div.pesquisaPaginaSelecionada`).
pub(crate) fn pagina_selecionada(
    dom: &tl::VDom<'_>,
) -> Result<Option<u64>, ErroRespostaDaPesquisa> {
    let marcadas = nos_por_elemento_e_classe(dom, None, "div", "pesquisaPaginaSelecionada")?;
    Ok(match marcadas.as_slice() {
        [uma] => ler_contagem(texto_visivel(*uma, dom.parser()).trim()),
        _ => None,
    })
}

/// A resposta 200 classificada, com as linhas lidas pelo leitor da operação.
pub(crate) enum Classificada<T> {
    SemResultado,
    Lista {
        itens: Vec<T>,
        total_declarado: u64,
        faixa: FaixaDaPesquisa,
        linhas_ausentes_da_faixa: u64,
    },
}

/// Classifica a resposta 200 (já sem validação da Infra) em lista ou
/// "sem resultado"; o 302 é interpretado antes, em `formulario_pesquisa`.
/// `ler` lê as linhas (processos ou documentos); a barra e a contagem são
/// as mesmas.
///
/// `na_pagina` é a faixa esperada e o número da página pedida por
/// `--pagina N`: na última página a PEN3 5.0.4 mostra "N resultados", que
/// só vale essa faixa quando ela termina em N e o paginador marca a página.
pub(crate) fn classificar_resposta<T>(
    html: &str,
    na_pagina: Option<(FaixaDaPesquisa, u64)>,
    ler: impl FnOnce(&str) -> Result<Vec<T>, ErroRespostaDaPesquisa>,
) -> Result<Classificada<T>, ErroRespostaDaPesquisa> {
    use ErroRespostaDaPesquisa as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_da_resposta_da_pesquisa_nao_interpretado"))?;
    let tabelas = nos_por_elemento_e_classe(&dom, None, "table", "pesquisaResultado")?;
    let avisos = nos_por_elemento_e_classe(&dom, None, "div", "pesquisaSemResultado")?;
    let barras = nos_por_elemento_e_classe(&dom, None, "div", "pesquisaBarraD")?;
    match (tabelas.is_empty(), avisos.len(), barras.is_empty()) {
        (true, 1, true) => return Ok(Classificada::SemResultado),
        (true, 1, false) => return Err(E::AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta),
        (true, _, _) => return Err(E::RespostaDaPesquisaNaoReconhecida),
        (false, 0, _) => {}
        (false, _, _) => return Err(E::TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta),
    }
    let barra = match barras.as_slice() {
        [] => return Err(ErroHtml::ausente("barra_de_contagem_da_pesquisa_ausente").into()),
        [barra] => *barra,
        _ => return Err(ErroHtml::repetido("barras_de_contagem_da_pesquisa_repetidas").into()),
    };
    let texto = decodificar_entidades(&texto_visivel(barra, dom.parser()));
    // `escrita`: a barra escreveu "Exibindo I - F de N"; "N resultados" só
    // implica a faixa e não admite linha ausente.
    let (faixa, total_declarado, escrita) = match ler_barra(&texto)? {
        (Some(faixa), total) => (faixa, total, true),
        (None, total) => match na_pagina {
            Some((esperada, numero))
                if esperada.inicio > 1
                    && esperada.fim == total
                    && pagina_selecionada(&dom)? == Some(numero) =>
            {
                (esperada, total, false)
            }
            _ => (
                FaixaDaPesquisa {
                    inicio: 1.min(total),
                    fim: total,
                },
                total,
                false,
            ),
        },
    };
    if faixa.inicio == 0 {
        return Err(E::FaixaDaBarraDeContagemImpossivel);
    }
    let itens = ler(html)?;
    // Goiás 5.0.3 publica menos registros que a faixa escrita na barra (9
    // de "1 - 10", tabela vazia em "1 - 10 de 3.300"): o HTML não tem as
    // linhas, `tr` = 3 por registro lido. Publica-se o que veio e quantas
    // posições faltam; nunca mais linhas que a faixa.
    let na_faixa = faixa.fim - faixa.inicio + 1;
    let lidas = itens.len() as u64;
    if lidas > na_faixa || (lidas < na_faixa && !escrita) {
        return Err(E::LinhasDaTabelaDiferentesDaFaixaDaBarra);
    }
    Ok(Classificada::Lista {
        itens,
        total_declarado,
        faixa,
        linhas_ausentes_da_faixa: na_faixa - lidas,
    })
}

/// A resposta 200 da pesquisa em processos.
pub(crate) fn ler_resposta_html(html: &str) -> Result<RespostaDaPesquisa, ErroRespostaDaPesquisa> {
    ler_resposta_na_pagina(html, None)
}

/// [`ler_resposta_html`] da resposta de `--pagina N` (faixa esperada e N).
pub(crate) fn ler_resposta_na_pagina(
    html: &str,
    na_pagina: Option<(FaixaDaPesquisa, u64)>,
) -> Result<RespostaDaPesquisa, ErroRespostaDaPesquisa> {
    let ler = |html: &str| ler_resultados(html).map_err(ErroRespostaDaPesquisa::Resultado);
    Ok(match classificar_resposta(html, na_pagina, ler)? {
        Classificada::SemResultado => RespostaDaPesquisa::SemResultado,
        Classificada::Lista {
            itens,
            total_declarado,
            faixa,
            linhas_ausentes_da_faixa,
        } => RespostaDaPesquisa::Lista(ListaDaPesquisa {
            processos: itens,
            total_declarado,
            faixa,
            linhas_ausentes_da_faixa,
        }),
    })
}

#[cfg(test)]
#[path = "tests_resposta.rs"]
mod tests;
