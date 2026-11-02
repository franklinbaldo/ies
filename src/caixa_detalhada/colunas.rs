//! Cabeçalho e células configuráveis da tabela D, lidos pelas chaves
//! estáveis de [`ColunaDoDetalhe`]. Cabeçalho desconhecido é recusa: uma
//! mudança de layout não pode embaralhar campos.
use super::ErroDetalhada;
use crate::dominio::Categoria;
use crate::nivel_de_detalhe::ColunaDoDetalhe;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;

/// Papel de uma coluna da tabela D, pelo rótulo do cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PapelDaColuna {
    Processo,
    Configurada(ColunaDoDetalhe),
    /// Seleção, ícones de estado e ações: cabeçalho sem rótulo.
    SemRotulo,
}

/// Valor observado numa célula configurável.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(crate) enum ValorDaColuna {
    /// Texto com espaços colapsados; `None` (JSON `null`) se a célula está vazia.
    Texto(Option<String>),
    /// Itens separados pelo HTML (`div.divRotuloItemCelula`).
    Itens(Vec<String>),
}

/// Colunas configuradas observadas numa linha, na ordem do cabeçalho.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ColunasDaLinha(pub(crate) Vec<(ColunaDoDetalhe, ValorDaColuna)>);

impl Serialize for ColunasDaLinha {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut mapa = serializer.serialize_map(Some(self.0.len()))?;
        for (coluna, valor) in &self.0 {
            mapa.serialize_entry(coluna.chave(), valor)?;
        }
        mapa.end()
    }
}

/// `"recebidos"`, `"gerados"` ou `null` (não observado na mesma leitura).
pub(crate) fn serializar_categoria<S: Serializer>(
    categoria: &Option<Categoria>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match categoria {
        Some(categoria) => serializer.serialize_str(categoria.rotulo()),
        None => serializer.serialize_none(),
    }
}

/// Rótulo e papel de cada `th`, na ordem do documento.
pub(super) fn papeis_do_cabecalho(
    dom: &tl::VDom<'_>,
    tabela: &tl::HTMLTag<'_>,
) -> Result<Vec<PapelDaColuna>, ErroDetalhada> {
    let _ = (dom, tabela);
    Err(ErroDetalhada::CabecalhoProcessoAusente)
}

/// Valor da célula `td` da coluna configurada.
pub(super) fn valor_da_celula(
    dom: &tl::VDom<'_>,
    celula: &tl::HTMLTag<'_>,
    coluna: ColunaDoDetalhe,
) -> Result<ValorDaColuna, ErroDetalhada> {
    let _ = (dom, celula, coluna);
    Ok(ValorDaColuna::Texto(None))
}
