//! Evidência da observação positiva numa página sem inferir cobertura global.
use crate::paginas::Leitura;
use crate::resumo_processo::Ocorrencia;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalhaResumoPagina {
    PaginasDoControleNaoObtidas,
    NumeroNaoObservadoNaPagina,
    NumeroNaoObservadoNaCaixa,
}
impl FalhaResumoPagina {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::PaginasDoControleNaoObtidas => "paginas_do_controle_nao_obtidas",
            Self::NumeroNaoObservadoNaPagina => "numero_nao_observado_na_pagina",
            Self::NumeroNaoObservadoNaCaixa => "numero_nao_observado_na_caixa",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::PaginasDoControleNaoObtidas => "a caixa exige páginas que não foram obtidas",
            Self::NumeroNaoObservadoNaPagina => {
                "número não observado na página recebida; outras páginas não foram consultadas"
            }
            Self::NumeroNaoObservadoNaCaixa => {
                "número não observado na caixa verificada; isso não prova inexistência no SEI"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservacaoPagina {
    pub(crate) ocorrencias: Vec<Ocorrencia>,
    pub(crate) pagina_parcial: bool,
}

/// Só uma ocorrência positiva permite responder com parte da caixa observada.
pub(crate) fn observar(
    leitura: &Leitura,
    numero: &str,
    aceitar_pagina_parcial: bool,
) -> Result<ObservacaoPagina, FalhaResumoPagina> {
    use crate::resumo_processo;
    let pagina_parcial = !leitura.recebidos.completa(&leitura.caixa.recebidos)
        || !leitura.gerados.completa(&leitura.caixa.gerados);
    if pagina_parcial && !aceitar_pagina_parcial {
        return Err(FalhaResumoPagina::PaginasDoControleNaoObtidas);
    }
    let ocorrencias = resumo_processo::ocorrencias(&leitura.caixa, numero);
    if ocorrencias.is_empty() {
        return Err(if pagina_parcial {
            FalhaResumoPagina::NumeroNaoObservadoNaPagina
        } else {
            FalhaResumoPagina::NumeroNaoObservadoNaCaixa
        });
    }
    Ok(ObservacaoPagina {
        ocorrencias,
        pagina_parcial,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas;

    fn observada(total: usize) -> Leitura {
        let html = format!(
            "<table id='tblProcessosRecebidos'>\
             <caption>{total} registros - 1 a 1):</caption>\
             <tr id='P42'><td><a class='processoVisualizado'>000123/2026</a></td></tr>\
             </table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>"
        );
        paginas::juntar(&[html], None).unwrap()
    }

    #[test]
    fn nup_presente_na_primeira_pagina_de_caixa_grande_e_observavel_com_opt_in() {
        let resultado = observar(&observada(120), "000123/2026", true).unwrap();
        assert!(resultado.pagina_parcial);
        assert_eq!(resultado.ocorrencias.len(), 1);
    }

    #[test]
    fn sem_opt_in_pagina_parcial_continua_recusada_mesmo_com_numero_visivel() {
        assert_eq!(
            observar(&observada(120), "000123/2026", false),
            Err(FalhaResumoPagina::PaginasDoControleNaoObtidas)
        );
    }

    #[test]
    fn ausencia_na_primeira_pagina_nao_vira_inexistencia_do_processo() {
        assert_eq!(
            observar(&observada(120), "000124/2026", true),
            Err(FalhaResumoPagina::NumeroNaoObservadoNaPagina)
        );
    }

    #[test]
    fn caixa_inteira_mantem_escopo_e_subcodigo_anteriores() {
        let leitura = observada(1);
        let presente = observar(&leitura, "000123/2026", true).unwrap();
        assert!(!presente.pagina_parcial);
        assert_eq!(
            observar(&leitura, "000124/2026", true),
            Err(FalhaResumoPagina::NumeroNaoObservadoNaCaixa)
        );
    }
}
