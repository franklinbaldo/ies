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
    _processos: &[ProcessoDetalhado],
    _numero: &str,
) -> Vec<Ocorrencia> {
    Vec::new()
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
    }
}
