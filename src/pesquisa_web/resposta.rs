//! Resposta 200 do POST da pesquisa por protocolo: tabela com barra de
//! contagem, aviso de "sem resultado" ou nenhuma forma reconhecida.
//! Formas observadas na `UnB` 4.0.9 de treinamento em 03/11/2026 (capturas
//! privadas, só estrutura): `div.pesquisaBarraD` "Exibindo 1 - 10 de 1.234"
//! antes de `table.pesquisaResultado`; `div.pesquisaSemResultado` sozinho.
use super::{ErroResultadoPesquisa, ResultadoPesquisa, ler_resultados};
use crate::dominio::IdProcedimento;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRespostaDaPesquisa {
    HtmlDaRespostaNaoInterpretado,
    RespostaDaPesquisaNaoReconhecida,
    TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta,
    /// Aviso de "sem resultado" e barra "Exibindo I - F de N" juntos.
    AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta,
    BarraDeContagemDaPesquisaAusente,
    BarrasDeContagemDaPesquisaRepetidas,
    TextoDaBarraDeContagemForaDoFormato,
    NumeroDaBarraDeContagemForaDoFormato,
    FaixaDaBarraDeContagemImpossivel,
    LinhasDaTabelaDiferentesDaFaixaDaBarra,
    Resultado(ErroResultadoPesquisa),
    AtributoHtml(ErroAtributoHtml),
}

impl ErroRespostaDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlDaRespostaNaoInterpretado => "html_da_resposta_da_pesquisa_nao_interpretado",
            Self::RespostaDaPesquisaNaoReconhecida => "resposta_da_pesquisa_nao_reconhecida",
            Self::TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta => {
                "tabela_de_resultado_e_aviso_de_sem_resultado_na_mesma_resposta"
            }
            Self::BarraDeContagemDaPesquisaAusente => "barra_de_contagem_da_pesquisa_ausente",
            Self::AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta => {
                "aviso_de_sem_resultado_e_barra_de_contagem_na_mesma_resposta"
            }
            Self::BarrasDeContagemDaPesquisaRepetidas => "barras_de_contagem_da_pesquisa_repetidas",
            Self::TextoDaBarraDeContagemForaDoFormato => {
                "texto_da_barra_de_contagem_fora_do_formato"
            }
            Self::NumeroDaBarraDeContagemForaDoFormato => {
                "numero_da_barra_de_contagem_fora_do_formato"
            }
            Self::FaixaDaBarraDeContagemImpossivel => "faixa_da_barra_de_contagem_impossivel",
            Self::LinhasDaTabelaDiferentesDaFaixaDaBarra => {
                "linhas_da_tabela_diferentes_da_faixa_da_barra"
            }
            Self::Resultado(erro) => erro.subcodigo(),
            Self::AtributoHtml(erro) => erro.subcodigo(),
        }
    }
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
fn ler_barra(texto: &str) -> Result<(FaixaDaPesquisa, u64), ErroRespostaDaPesquisa> {
    use ErroRespostaDaPesquisa as E;
    let palavras: Vec<&str> = texto.split_whitespace().collect();
    let ["Exibindo", inicio, "-", fim, "de", total] = palavras.as_slice() else {
        return Err(E::TextoDaBarraDeContagemForaDoFormato);
    };
    let numero = |t: &str| ler_contagem(t).ok_or(E::NumeroDaBarraDeContagemForaDoFormato);
    let (inicio, fim, total) = (numero(inicio)?, numero(fim)?, numero(total)?);
    if inicio == 0 || inicio > fim || fim > total {
        return Err(E::FaixaDaBarraDeContagemImpossivel);
    }
    Ok((FaixaDaPesquisa { inicio, fim }, total))
}

/// Classifica a resposta 200 (já sem validação da Infra) em lista ou
/// "sem resultado"; o 302 é interpretado antes, em `formulario_pesquisa`.
pub(crate) fn ler_resposta_html(html: &str) -> Result<RespostaDaPesquisa, ErroRespostaDaPesquisa> {
    use ErroRespostaDaPesquisa as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| E::HtmlDaRespostaNaoInterpretado)?;
    let tabelas = nos_por_elemento_e_classe(&dom, None, "table", "pesquisaResultado")
        .map_err(E::AtributoHtml)?;
    let avisos = nos_por_elemento_e_classe(&dom, None, "div", "pesquisaSemResultado")
        .map_err(E::AtributoHtml)?;
    let barras =
        nos_por_elemento_e_classe(&dom, None, "div", "pesquisaBarraD").map_err(E::AtributoHtml)?;
    match (tabelas.is_empty(), avisos.len(), barras.is_empty()) {
        (true, 1, true) => return Ok(RespostaDaPesquisa::SemResultado),
        (true, 1, false) => return Err(E::AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta),
        (true, _, _) => return Err(E::RespostaDaPesquisaNaoReconhecida),
        (false, 0, _) => {}
        (false, _, _) => return Err(E::TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta),
    }
    let barra = match barras.as_slice() {
        [] => return Err(E::BarraDeContagemDaPesquisaAusente),
        [barra] => *barra,
        _ => return Err(E::BarrasDeContagemDaPesquisaRepetidas),
    };
    let texto = decodificar_entidades(&texto_visivel(barra, dom.parser()));
    let (faixa, total_declarado) = ler_barra(&texto)?;
    let processos = ler_resultados(html).map_err(E::Resultado)?;
    if processos.len() as u64 != faixa.fim - faixa.inicio + 1 {
        return Err(E::LinhasDaTabelaDiferentesDaFaixaDaBarra);
    }
    Ok(RespostaDaPesquisa::Lista(ListaDaPesquisa {
        processos,
        total_declarado,
        faixa,
    }))
}

#[cfg(test)]
#[path = "tests_resposta.rs"]
mod tests;
