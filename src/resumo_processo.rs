//! Consulta mínima de identidade: só reutiliza os dados da Caixa de Entrada.
use crate::caixa_detalhada::ProcessoDetalhado;
use crate::dominio::{CaixaDeEntrada, Processo};
use serde::Serialize;

/// O processo como a lista o publicou; mesma forma JSON nas duas visões.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(crate) enum ProcessoObservado {
    Categoria(Processo),
    /// Visão detalhada: `visualizado`/`sigiloso` podem não ser observáveis.
    SemCategoria(ProcessoDetalhado),
}

/// `categoria: None` (JSON `null`) na lista única da visão detalhada.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Ocorrencia {
    categoria: Option<&'static str>,
    processo: ProcessoObservado,
}

pub fn ocorrencias(caixa: &CaixaDeEntrada, numero: &str) -> Vec<Ocorrencia> {
    let mut achados = Vec::new();
    for (categoria, tabela) in [("recebidos", &caixa.recebidos), ("gerados", &caixa.gerados)] {
        for processo in &tabela.processos {
            if processo.numero == numero {
                achados.push(Ocorrencia {
                    categoria: Some(categoria),
                    processo: ProcessoObservado::Categoria(processo.clone()),
                });
            }
        }
    }
    achados
}

/// Ocorrências na lista única da visão detalhada, sem categoria inferida.
pub(crate) fn ocorrencias_sem_categoria(
    processos: &[ProcessoDetalhado],
    numero: &str,
) -> Vec<Ocorrencia> {
    processos
        .iter()
        .filter(|processo| processo.numero == numero)
        .map(|processo| Ocorrencia {
            categoria: None,
            processo: ProcessoObservado::SemCategoria(processo.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::{Categoria, IdProcedimento, Tabela};

    #[test]
    fn numero_e_opaco_e_grupo_nao_e_estado_do_processo() {
        let processo = Processo {
            id: IdProcedimento::novo("42").unwrap(),
            numero: "000123/2026".into(),
            visualizado: false,
            sigiloso: false,
        };
        let caixa = CaixaDeEntrada {
            recebidos: Tabela::nova(Categoria::Recebidos, Some(1), vec![processo]).unwrap(),
            gerados: Tabela::nova(Categoria::Gerados, Some(0), Vec::new()).unwrap(),
        };
        assert_eq!(ocorrencias(&caixa, "000123/2026").len(), 1);
        assert_eq!(ocorrencias(&caixa, "123/2026"), []);
        let json = serde_json::to_value(ocorrencias(&caixa, "000123/2026")).unwrap();
        assert_eq!(json[0]["categoria"], "recebidos");
        assert_eq!(json[0]["processo"]["visualizado"], false);
    }

    #[test]
    fn ocorrencia_na_lista_sem_categoria_publica_categoria_null() {
        let processos = [ProcessoDetalhado {
            id: IdProcedimento::novo("42").unwrap(),
            numero: "000123/2026".into(),
            visualizado: None,
            sigiloso: Some(false),
        }];
        let achados = ocorrencias_sem_categoria(&processos, "000123/2026");
        assert_eq!(achados.len(), 1);
        assert_eq!(ocorrencias_sem_categoria(&processos, "123/2026"), []);
        let json = serde_json::to_value(&achados).unwrap();
        let ocorrencia = json[0].as_object().unwrap();
        assert_eq!(ocorrencia.get("categoria"), Some(&serde_json::Value::Null));
        assert_eq!(json[0]["processo"]["id"], "42");
        assert_eq!(json[0]["processo"]["numero"], "000123/2026");
        assert!(json[0]["processo"]["visualizado"].is_null());
        assert_eq!(json[0]["processo"]["sigiloso"], false);
    }
}
