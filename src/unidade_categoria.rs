//! Projeção Web de uma categoria integralmente observada, sem POST.
use crate::paginas::Leitura;
use clap::ValueEnum;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CategoriaLista {
    Recebidos,
    Gerados,
}

impl CategoriaLista {
    pub(crate) fn rotulo(self) -> &'static str {
        match self {
            Self::Recebidos => "recebidos",
            Self::Gerados => "gerados",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalhaCategoria {
    PaginasNaoObtidas(CategoriaLista),
}
impl FalhaCategoria {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::PaginasNaoObtidas(CategoriaLista::Recebidos) => "paginas_recebidos_nao_obtidas",
            Self::PaginasNaoObtidas(CategoriaLista::Gerados) => "paginas_gerados_nao_obtidas",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        "a categoria selecionada exige páginas que não foram obtidas"
    }
}

/// Shim do contrato. Testes RED evidenciam rejeição indevida de categoria já coberta.
pub(crate) fn selecionar(
    _leitura: &Leitura,
    categoria: CategoriaLista,
    _origem: &str,
) -> Result<Value, FalhaCategoria> {
    Err(FalhaCategoria::PaginasNaoObtidas(categoria))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas;

    fn leitura(recebidos_total: usize) -> Leitura {
        let html = format!(
            "<table id='tblProcessosRecebidos'>\
             <caption>{recebidos_total} registros - 1 a 1):</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>1 registros - 1 a 1):</caption>\
             <tr id='P2'><td><a class='processoVisualizado'>G-0002</a></td></tr></table>"
        );
        paginas::juntar(&[html], None).unwrap()
    }

    #[test]
    fn gerados_inteiros_sao_consultaveis_mesmo_se_recebidos_paginam() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Gerados, "web").unwrap();
        assert_eq!(modelo["categoria"], "gerados");
        assert_eq!(modelo["total_declarado"], 1);
        assert_eq!(modelo["processos"][0]["numero"], "G-0002");
        assert!(modelo.get("recebidos").is_none());
        assert!(modelo.get("completa").is_none());
    }

    #[test]
    fn categoria_requerida_ainda_incompleta_falha_sem_publicar_resultado() {
        assert_eq!(
            selecionar(&leitura(200), CategoriaLista::Recebidos, "web"),
            Err(FalhaCategoria::PaginasNaoObtidas(CategoriaLista::Recebidos))
        );
    }

    #[test]
    fn categoria_unica_com_todas_posicoes_preserva_numeros() {
        let modelo = selecionar(&leitura(1), CategoriaLista::Recebidos, "web").unwrap();
        assert_eq!(modelo["processos"][0]["numero"], "R-0001");
        assert_eq!(modelo["total_declarado"], 1);
        assert!(modelo.get("gerados").is_none());
    }
}
