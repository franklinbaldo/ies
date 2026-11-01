//! Releitura offline de exportação privada da caixa; nunca infere sessão viva.
use crate::dominio::{CaixaDeEntrada, Categoria, IdProcedimento, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use crate::processo::NumeroProcesso;
use crate::transporte::{self, FalhaTransporte};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroCaixaLocal {
    Arquivo(FalhaTransporte),
    FormatoInvalido,
    OrigemNaoWeb,
    UnidadeDiferenteDaSolicitada,
    TotalNaoComprovado { categoria: Categoria },
    IdentificadorInvalido { categoria: Categoria },
    NumeroInvalido { categoria: Categoria },
    IdentificadorRepetido { categoria: Categoria },
}

impl ErroCaixaLocal {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Arquivo(falha) => falha.subcodigo(),
            Self::FormatoInvalido => "formato_do_arquivo_da_caixa_invalido",
            Self::OrigemNaoWeb => "origem_do_arquivo_da_caixa_nao_web",
            Self::UnidadeDiferenteDaSolicitada => "unidade_do_arquivo_da_caixa_diferente",
            Self::TotalNaoComprovado {
                categoria: Categoria::Recebidos,
            } => "total_recebidos_do_arquivo_nao_comprovado",
            Self::TotalNaoComprovado {
                categoria: Categoria::Gerados,
            } => "total_gerados_do_arquivo_nao_comprovado",
            Self::IdentificadorInvalido {
                categoria: Categoria::Recebidos,
            } => "id_recebidos_do_arquivo_invalido",
            Self::IdentificadorInvalido {
                categoria: Categoria::Gerados,
            } => "id_gerados_do_arquivo_invalido",
            Self::NumeroInvalido {
                categoria: Categoria::Recebidos,
            } => "numero_recebidos_do_arquivo_invalido",
            Self::NumeroInvalido {
                categoria: Categoria::Gerados,
            } => "numero_gerados_do_arquivo_invalido",
            Self::IdentificadorRepetido {
                categoria: Categoria::Recebidos,
            } => "id_recebidos_do_arquivo_repetido",
            Self::IdentificadorRepetido {
                categoria: Categoria::Gerados,
            } => "id_gerados_do_arquivo_repetido",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::Arquivo(falha) => falha.mensagem(),
            Self::FormatoInvalido => "arquivo da caixa não tem a estrutura JSON esperada",
            Self::OrigemNaoWeb => "arquivo não declara a origem de exportação Web",
            Self::UnidadeDiferenteDaSolicitada => "unidade do arquivo da caixa não é a solicitada",
            Self::TotalNaoComprovado { .. } => {
                "total da categoria não confere com as linhas exportadas"
            }
            Self::IdentificadorInvalido { .. } => "identificador de processo inválido no arquivo",
            Self::NumeroInvalido { .. } => "número de processo inválido no arquivo",
            Self::IdentificadorRepetido { .. } => "identificador de processo repetido no arquivo",
        }
    }
}

#[derive(Debug)]
pub(crate) struct CaixaObservadaLocal {
    pub(crate) caixa: CaixaDeEntrada,
    pub(crate) unidade: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exportacao {
    unidade: Option<String>,
    origem: String,
    recebidos: Secao,
    gerados: Secao,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Secao {
    total_declarado: Option<usize>,
    processos: Vec<Linha>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Linha {
    id: String,
    numero: String,
    visualizado: bool,
}

fn converter(secao: Secao, categoria: Categoria) -> Result<Tabela<Processo>, ErroCaixaLocal> {
    // Arquivo arbitrário não comprova cobertura por si só, mas totais divergentes
    // impedem até a interpretação interna como uma caixa integral exportada.
    if secao.total_declarado != Some(secao.processos.len()) {
        return Err(ErroCaixaLocal::TotalNaoComprovado { categoria });
    }
    let mut processos = Vec::with_capacity(secao.processos.len());
    for linha in secao.processos {
        let id = IdProcedimento::novo(&linha.id)
            .map_err(|_| ErroCaixaLocal::IdentificadorInvalido { categoria })?;
        let numero = NumeroProcesso::novo(&linha.numero)
            .map_err(|_| ErroCaixaLocal::NumeroInvalido { categoria })?;
        // Não reescrever um NUP opaco nem aparar espaços no arquivo escolhido.
        if numero.as_str() != linha.numero {
            return Err(ErroCaixaLocal::NumeroInvalido { categoria });
        }
        processos.push(Processo {
            id,
            numero: linha.numero,
            visualizado: linha.visualizado,
        });
    }
    Tabela::nova(categoria, secao.total_declarado, processos).map_err(|erro| match erro {
        ErroCaixa::IdRepetido { .. } => ErroCaixaLocal::IdentificadorRepetido { categoria },
        _ => ErroCaixaLocal::TotalNaoComprovado { categoria },
    })
}

/// Interpreta somente exportações da CLI; consistência interna não autentica o SEI.
pub(crate) fn interpretar(
    json: &str,
    unidade: Option<&str>,
) -> Result<CaixaObservadaLocal, ErroCaixaLocal> {
    let exportada: Exportacao =
        serde_json::from_str(json).map_err(|_| ErroCaixaLocal::FormatoInvalido)?;
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
    Ok(CaixaObservadaLocal {
        caixa: CaixaDeEntrada {
            recebidos: converter(exportada.recebidos, Categoria::Recebidos)?,
            gerados: converter(exportada.gerados, Categoria::Gerados)?,
        },
        unidade: exportada.unidade,
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
mod tests {
    use super::*;
    use serde_json::json;

    fn caixa() -> String {
        json!({
            "unidade": "PPI", "origem": "web",
            "recebidos": {
                "total_declarado": 1,
                "processos": [{"id":"0042","numero":"000123/2026","visualizado":false}]
            },
            "gerados": {"total_declarado": 0, "processos": []}
        })
        .to_string()
    }
    #[test]
    fn encontra_nup_exato_de_arquivo_de_caixa_sem_rede() {
        let observada = interpretar(&caixa(), Some("ppi")).unwrap();
        assert_eq!(observada.unidade.as_deref(), Some("PPI"));
        assert_eq!(observada.caixa.recebidos.processos[0].numero, "000123/2026");
        assert_eq!(observada.caixa.recebidos.processos[0].id.as_str(), "0042");
    }
    #[test]
    fn recusa_total_que_nao_comprova_todas_as_linhas_da_categoria() {
        let json = caixa().replace("\"total_declarado\":1", "\"total_declarado\":2");
        assert_eq!(
            interpretar(&json, None).unwrap_err(),
            ErroCaixaLocal::TotalNaoComprovado {
                categoria: Categoria::Recebidos
            }
        );
    }
    #[test]
    fn recusa_unidade_trocada_sem_abrir_sessao() {
        assert_eq!(
            interpretar(&caixa(), Some("OUTRA")).unwrap_err(),
            ErroCaixaLocal::UnidadeDiferenteDaSolicitada
        );
    }
    #[test]
    fn recusa_identificador_repetido_sem_selecionar_a_primeira_linha() {
        let mut objeto: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
        objeto["recebidos"]["total_declarado"] = json!(2);
        let repetido = objeto["recebidos"]["processos"][0].clone();
        objeto["recebidos"]["processos"]
            .as_array_mut()
            .unwrap()
            .push(repetido);
        assert_eq!(
            interpretar(&objeto.to_string(), None).unwrap_err(),
            ErroCaixaLocal::IdentificadorRepetido {
                categoria: Categoria::Recebidos
            }
        );
    }
    #[test]
    fn aceita_exportacao_de_caixa_multipagina_gerada_pelo_ies() {
        let mut exportada: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
        exportada["origem"] = json!("web_com_paginas_privadas");
        let observada = interpretar(&exportada.to_string(), Some("PPI")).unwrap();
        assert_eq!(observada.caixa.recebidos.processos[0].numero, "000123/2026");
        assert_eq!(observada.caixa.recebidos.processos[0].id.as_str(), "0042");
    }

    #[test]
    fn exportacao_multipagina_continua_exigindo_totais_comprovados() {
        let mut exportada: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
        exportada["origem"] = json!("web_com_paginas_privadas");
        exportada["recebidos"]["total_declarado"] = json!(2);
        assert_eq!(
            interpretar(&exportada.to_string(), None).unwrap_err(),
            ErroCaixaLocal::TotalNaoComprovado {
                categoria: Categoria::Recebidos
            }
        );
    }

    #[test]
    fn apenas_as_duas_origens_de_exportacao_reais_sao_aceitas() {
        for origem in ["WEB", "web_replay", "web_com_paginas_privadas_fake"] {
            let mut exportada: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
            exportada["origem"] = json!(origem);
            assert_eq!(
                interpretar(&exportada.to_string(), None).unwrap_err(),
                ErroCaixaLocal::OrigemNaoWeb
            );
        }
    }

    #[test]
    fn recusa_origem_de_cassette_ou_json_nao_proveniente_de_exportacao() {
        let json = caixa().replace("\"origem\":\"web\"", "\"origem\":\"replay\"");
        assert_eq!(
            interpretar(&json, None).unwrap_err(),
            ErroCaixaLocal::OrigemNaoWeb
        );
    }
}
