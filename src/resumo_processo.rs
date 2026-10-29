//! Consulta mínima de identidade: só reutiliza os dados da Caixa de Entrada.
use crate::dominio::{CaixaDeEntrada, Processo};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Ocorrencia {
    categoria: &'static str,
    processo: Processo,
}

pub fn ocorrencias(caixa: &CaixaDeEntrada, numero: &str) -> Vec<Ocorrencia> {
    let mut achados = Vec::new();
    for (categoria, tabela) in [("recebidos", &caixa.recebidos), ("gerados", &caixa.gerados)] {
        for processo in &tabela.processos {
            if processo.numero == numero {
                achados.push(Ocorrencia {
                    categoria,
                    processo: processo.clone(),
                });
            }
        }
    }
    achados
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
        };
        let caixa = CaixaDeEntrada {
            recebidos: Tabela::nova(Categoria::Recebidos, Some(1), vec![processo]).unwrap(),
            gerados: Tabela::nova(Categoria::Gerados, Some(0), Vec::new()).unwrap(),
        };
        assert_eq!(ocorrencias(&caixa, "000123/2026").len(), 1);
        assert_eq!(ocorrencias(&caixa, "123/2026"), []);
    }
}
