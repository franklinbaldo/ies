//! Releitura offline de exportação privada da caixa; nunca infere sessão viva.
use crate::dominio::{CaixaDeEntrada, Categoria};
use crate::transporte::{self, FalhaTransporte};
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
            Self::TotalNaoComprovado { categoria: Categoria::Recebidos } => "total_recebidos_do_arquivo_nao_comprovado",
            Self::TotalNaoComprovado { categoria: Categoria::Gerados } => "total_gerados_do_arquivo_nao_comprovado",
            Self::IdentificadorInvalido { categoria: Categoria::Recebidos } => "id_recebidos_do_arquivo_invalido",
            Self::IdentificadorInvalido { categoria: Categoria::Gerados } => "id_gerados_do_arquivo_invalido",
            Self::NumeroInvalido { categoria: Categoria::Recebidos } => "numero_recebidos_do_arquivo_invalido",
            Self::NumeroInvalido { categoria: Categoria::Gerados } => "numero_gerados_do_arquivo_invalido",
            Self::IdentificadorRepetido { categoria: Categoria::Recebidos } => "id_recebidos_do_arquivo_repetido",
            Self::IdentificadorRepetido { categoria: Categoria::Gerados } => "id_gerados_do_arquivo_repetido",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::Arquivo(falha) => falha.mensagem(),
            Self::FormatoInvalido => "arquivo da caixa não tem a estrutura JSON esperada",
            Self::OrigemNaoWeb => "arquivo não declara a origem de exportação Web",
            Self::UnidadeDiferenteDaSolicitada => "unidade do arquivo da caixa não é a solicitada",
            Self::TotalNaoComprovado { .. } => "total da categoria não confere com as linhas exportadas",
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

/// Shim de CONTRACT; o RED precisa mostrar que ainda não interpreta a exportação.
pub(crate) fn interpretar(_json: &str, _unidade: Option<&str>) -> Result<CaixaObservadaLocal, ErroCaixaLocal> {
    Err(ErroCaixaLocal::FormatoInvalido)
}

/// Usa a leitura privada já endurecida no transporte, sem abrir socket.
pub(crate) fn ler(caminho: &Path, unidade: Option<&str>) -> Result<CaixaObservadaLocal, ErroCaixaLocal> {
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
        }).to_string()
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
            ErroCaixaLocal::TotalNaoComprovado { categoria: Categoria::Recebidos }
        );
    }
    #[test]
    fn recusa_unidade_trocada_sem_abrir_sessao() {
        assert_eq!(
            interpretar(&caixa(), Some("OUTRA")).unwrap_err(),
            ErroCaixaLocal::UnidadeDiferenteDaSolicitada
        );
    }
}
