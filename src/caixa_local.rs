//! Releitura offline de exportação privada da caixa; nunca infere sessão viva.
use crate::caixa_detalhada::ProcessoDetalhado;
use crate::dominio::{CaixaDeEntrada, Categoria, IdProcedimento, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use crate::processo::NumeroProcesso;
use crate::transporte::{self, FalhaTransporte};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ErroCaixaLocal {
    Arquivo(FalhaTransporte),
    FormatoInvalido,
    OrigemNaoWeb,
    UnidadeDiferenteDaSolicitada,
    /// O array exportado não é exatamente `[recebidos, gerados]`.
    ListasExportadasNaoSaoRecebidosEGerados,
    /// As duas listas exportadas declaram rótulos de unidade diferentes.
    UnidadeDasListasExportadasDiferente,
    /// As duas listas exportadas declaram origens diferentes.
    OrigemDasListasExportadasDiferente,
    /// `categoria: None` é a lista única exportada da visão detalhada.
    TotalNaoComprovado {
        categoria: Option<Categoria>,
    },
    IdentificadorInvalido {
        categoria: Option<Categoria>,
    },
    NumeroInvalido {
        categoria: Option<Categoria>,
    },
    IdentificadorRepetido {
        categoria: Option<Categoria>,
    },
}

impl ErroCaixaLocal {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Arquivo(falha) => falha.subcodigo(),
            Self::FormatoInvalido => "formato_do_arquivo_da_caixa_invalido",
            Self::OrigemNaoWeb => "origem_do_arquivo_da_caixa_nao_web",
            Self::UnidadeDiferenteDaSolicitada => "unidade_do_arquivo_da_caixa_diferente",
            Self::ListasExportadasNaoSaoRecebidosEGerados => {
                "listas_do_arquivo_da_caixa_nao_sao_recebidos_e_gerados"
            }
            Self::UnidadeDasListasExportadasDiferente => {
                "unidade_das_listas_do_arquivo_da_caixa_diferente"
            }
            Self::OrigemDasListasExportadasDiferente => {
                "origem_das_listas_do_arquivo_da_caixa_diferente"
            }
            Self::TotalNaoComprovado {
                categoria: Some(Categoria::Recebidos),
            } => "total_recebidos_do_arquivo_nao_comprovado",
            Self::TotalNaoComprovado {
                categoria: Some(Categoria::Gerados),
            } => "total_gerados_do_arquivo_nao_comprovado",
            Self::TotalNaoComprovado { categoria: None } => {
                "total_da_lista_sem_categoria_do_arquivo_nao_comprovado"
            }
            Self::IdentificadorInvalido {
                categoria: Some(Categoria::Recebidos),
            } => "id_recebidos_do_arquivo_invalido",
            Self::IdentificadorInvalido {
                categoria: Some(Categoria::Gerados),
            } => "id_gerados_do_arquivo_invalido",
            Self::IdentificadorInvalido { categoria: None } => {
                "id_da_lista_sem_categoria_do_arquivo_invalido"
            }
            Self::NumeroInvalido {
                categoria: Some(Categoria::Recebidos),
            } => "numero_recebidos_do_arquivo_invalido",
            Self::NumeroInvalido {
                categoria: Some(Categoria::Gerados),
            } => "numero_gerados_do_arquivo_invalido",
            Self::NumeroInvalido { categoria: None } => {
                "numero_da_lista_sem_categoria_do_arquivo_invalido"
            }
            Self::IdentificadorRepetido {
                categoria: Some(Categoria::Recebidos),
            } => "id_recebidos_do_arquivo_repetido",
            Self::IdentificadorRepetido {
                categoria: Some(Categoria::Gerados),
            } => "id_gerados_do_arquivo_repetido",
            Self::IdentificadorRepetido { categoria: None } => {
                "id_da_lista_sem_categoria_do_arquivo_repetido"
            }
        }
    }
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Arquivo(falha) => falha.mensagem(),
            Self::FormatoInvalido => "arquivo da caixa não tem a estrutura JSON esperada",
            Self::OrigemNaoWeb => "arquivo não declara a origem de exportação Web",
            Self::UnidadeDiferenteDaSolicitada => "unidade do arquivo da caixa não é a solicitada",
            Self::ListasExportadasNaoSaoRecebidosEGerados => {
                "arquivo da caixa não traz as listas recebidos e gerados, nessa ordem"
            }
            Self::UnidadeDasListasExportadasDiferente => {
                "as listas do arquivo da caixa declaram unidades diferentes"
            }
            Self::OrigemDasListasExportadasDiferente => {
                "as listas do arquivo da caixa declaram origens diferentes"
            }
            Self::TotalNaoComprovado { .. } => {
                "total da categoria não confere com as linhas exportadas"
            }
            Self::IdentificadorInvalido { .. } => "identificador de processo inválido no arquivo",
            Self::NumeroInvalido { .. } => "número de processo inválido no arquivo",
            Self::IdentificadorRepetido { .. } => "identificador de processo repetido no arquivo",
        }
    }
}

/// As listas lidas do arquivo, na forma em que foram exportadas.
#[derive(Debug)]
pub(crate) enum ListasLocais {
    /// `[recebidos, gerados]` da visão resumida.
    Categorias(CaixaDeEntrada),
    /// `[lista]` da visão detalhada: `categoria: null`, sem categoria por linha.
    SemCategoria(Vec<ProcessoDetalhado>),
}

#[derive(Debug)]
pub(crate) struct CaixaObservadaLocal {
    pub(crate) listas: ListasLocais,
    pub(crate) unidade: Option<String>,
}

/// Uma lista do array exportado por `listar-processos --saida`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListaExportada {
    unidade: Option<String>,
    origem: String,
    /// `null` somente na lista única da visão detalhada.
    categoria: Option<String>,
    total_declarado: Option<usize>,
    // A importação comprova cobertura pelo total igual às linhas; a faixa
    // é dado da leitura e não substitui essa prova.
    #[serde(rename = "faixa_observada")]
    _faixa_observada: serde::de::IgnoredAny,
    processos: Vec<Linha>,
}

/// `visualizado`/`sigiloso` são `null` na visão detalhada quando a classe
/// do link não foi reconhecida; na resumida, `visualizado` é obrigatório.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Linha {
    id: String,
    numero: String,
    visualizado: Option<bool>,
    #[serde(default)]
    sigiloso: Option<bool>,
    /// Lista D: categoria observada nas tabelas R do mesmo GET, ou `null`.
    #[serde(default)]
    categoria: Option<String>,
    /// Lista D: colunas configuradas; não participam do resumo.
    #[serde(default, rename = "colunas")]
    _colunas: Option<serde::de::IgnoredAny>,
}

/// Identidade da linha preservada como exportada (NUP opaco, sem aparar).
fn identidade(
    linha: &Linha,
    categoria: Option<Categoria>,
) -> Result<IdProcedimento, ErroCaixaLocal> {
    let id = IdProcedimento::novo(&linha.id)
        .map_err(|_| ErroCaixaLocal::IdentificadorInvalido { categoria })?;
    let numero = NumeroProcesso::novo(&linha.numero)
        .map_err(|_| ErroCaixaLocal::NumeroInvalido { categoria })?;
    if numero.as_str() != linha.numero {
        return Err(ErroCaixaLocal::NumeroInvalido { categoria });
    }
    Ok(id)
}

fn converter(
    secao: ListaExportada,
    categoria: Categoria,
) -> Result<Tabela<Processo>, ErroCaixaLocal> {
    // Arquivo arbitrário não comprova cobertura por si só, mas totais divergentes
    // impedem até a interpretação interna como uma caixa integral exportada.
    if secao.total_declarado != Some(secao.processos.len()) {
        return Err(ErroCaixaLocal::TotalNaoComprovado {
            categoria: Some(categoria),
        });
    }
    let mut processos = Vec::with_capacity(secao.processos.len());
    for linha in secao.processos {
        let id = identidade(&linha, Some(categoria))?;
        processos.push(Processo {
            id,
            numero: linha.numero,
            visualizado: linha.visualizado.ok_or(ErroCaixaLocal::FormatoInvalido)?,
            // Exportações anteriores não tinham o campo: aquelas versões
            // recusavam linhas sem as classes não sigilosas.
            sigiloso: linha.sigiloso.unwrap_or(false),
        });
    }
    Tabela::nova(categoria, secao.total_declarado, processos).map_err(|erro| match erro {
        ErroCaixa::IdRepetido { .. } => ErroCaixaLocal::IdentificadorRepetido {
            categoria: Some(categoria),
        },
        _ => ErroCaixaLocal::TotalNaoComprovado {
            categoria: Some(categoria),
        },
    })
}

/// Lista única da visão detalhada: mesmas provas de total e identidade.
fn converter_sem_categoria(
    secao: ListaExportada,
) -> Result<Vec<ProcessoDetalhado>, ErroCaixaLocal> {
    if secao.total_declarado != Some(secao.processos.len()) {
        return Err(ErroCaixaLocal::TotalNaoComprovado { categoria: None });
    }
    let mut vistos = HashSet::with_capacity(secao.processos.len());
    let mut processos = Vec::with_capacity(secao.processos.len());
    for linha in secao.processos {
        let id = identidade(&linha, None)?;
        if !vistos.insert(id.as_str().to_owned()) {
            return Err(ErroCaixaLocal::IdentificadorRepetido { categoria: None });
        }
        let categoria = match linha.categoria.as_deref() {
            None => None,
            Some("recebidos") => Some(Categoria::Recebidos),
            Some("gerados") => Some(Categoria::Gerados),
            Some(_) => return Err(ErroCaixaLocal::FormatoInvalido),
        };
        processos.push(ProcessoDetalhado {
            id,
            numero: linha.numero,
            visualizado: linha.visualizado,
            sigiloso: linha.sigiloso,
            categoria,
            colunas: crate::caixa_detalhada::ColunasDaLinha::default(),
        });
    }
    Ok(processos)
}

/// Origem e unidade valem para todas as listas do arquivo.
fn conferir_contexto(
    exportada: &ListaExportada,
    unidade: Option<&str>,
) -> Result<(), ErroCaixaLocal> {
    if !matches!(
        exportada.origem.as_str(),
        "web" | "web_com_paginas_privadas"
    ) {
        return Err(ErroCaixaLocal::OrigemNaoWeb);
    }
    if exportada
        .unidade
        .as_deref()
        .is_some_and(|valor| valor.trim().is_empty())
    {
        return Err(ErroCaixaLocal::FormatoInvalido);
    }
    if let Some(esperada) = unidade
        && exportada
            .unidade
            .as_deref()
            .is_none_or(|atual| !atual.eq_ignore_ascii_case(esperada.trim()))
    {
        return Err(ErroCaixaLocal::UnidadeDiferenteDaSolicitada);
    }
    Ok(())
}

/// Interpreta somente exportações da CLI; consistência interna não autentica o SEI.
pub(crate) fn interpretar(
    json: &str,
    unidade: Option<&str>,
) -> Result<CaixaObservadaLocal, ErroCaixaLocal> {
    let listas: Vec<ListaExportada> =
        serde_json::from_str(json).map_err(|_| ErroCaixaLocal::FormatoInvalido)?;
    if listas.len() == 1 {
        return interpretar_sem_categoria(listas, unidade);
    }
    let [recebidos, gerados]: [ListaExportada; 2] = listas
        .try_into()
        .map_err(|_| ErroCaixaLocal::ListasExportadasNaoSaoRecebidosEGerados)?;
    if recebidos.categoria.as_deref() != Some(Categoria::Recebidos.rotulo())
        || gerados.categoria.as_deref() != Some(Categoria::Gerados.rotulo())
    {
        return Err(ErroCaixaLocal::ListasExportadasNaoSaoRecebidosEGerados);
    }
    if recebidos.unidade != gerados.unidade {
        return Err(ErroCaixaLocal::UnidadeDasListasExportadasDiferente);
    }
    if recebidos.origem != gerados.origem {
        return Err(ErroCaixaLocal::OrigemDasListasExportadasDiferente);
    }
    conferir_contexto(&recebidos, unidade)?;
    let unidade_exportada = recebidos.unidade.clone();
    Ok(CaixaObservadaLocal {
        listas: ListasLocais::Categorias(CaixaDeEntrada {
            recebidos: converter(recebidos, Categoria::Recebidos)?,
            gerados: converter(gerados, Categoria::Gerados)?,
        }),
        unidade: unidade_exportada,
    })
}

/// `[lista]` exportada da visão detalhada (`categoria: null`).
fn interpretar_sem_categoria(
    listas: Vec<ListaExportada>,
    unidade: Option<&str>,
) -> Result<CaixaObservadaLocal, ErroCaixaLocal> {
    let [lista]: [ListaExportada; 1] = listas
        .try_into()
        .map_err(|_| ErroCaixaLocal::ListasExportadasNaoSaoRecebidosEGerados)?;
    match lista.categoria.as_deref() {
        None => {}
        // Lista única gravada pela #146 (pré-1.0): formato anterior, sem alias.
        Some("detalhada") => return Err(ErroCaixaLocal::FormatoInvalido),
        Some(_) => return Err(ErroCaixaLocal::ListasExportadasNaoSaoRecebidosEGerados),
    }
    conferir_contexto(&lista, unidade)?;
    let unidade_exportada = lista.unidade.clone();
    Ok(CaixaObservadaLocal {
        listas: ListasLocais::SemCategoria(converter_sem_categoria(lista)?),
        unidade: unidade_exportada,
    })
}

/// Reaproveita leitura privada com limite e descritor protegido, sem abrir socket.
pub(crate) fn ler(
    caminho: &Path,
    unidade: Option<&str>,
) -> Result<CaixaObservadaLocal, ErroCaixaLocal> {
    let dados = transporte::arquivo_privado(caminho, 8_000_000).map_err(ErroCaixaLocal::Arquivo)?;
    interpretar(&dados, unidade)
}

#[cfg(test)]
#[path = "caixa_local/tests.rs"]
mod tests;
