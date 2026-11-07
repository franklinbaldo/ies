//! Conferência das pastas para consulta informativa de processos (#342).
//! Somente lacunas de documentos degradam; corrupção e identidade divergente recusam.

use super::pastas::{NoDePasta, conferir_pastas};
use super::{DocumentoLido, DocumentoNaArvore, ErroArvore, PastaNaArvore};

pub(super) struct Conferencia {
    pub(super) pastas: Vec<PastaNaArvore>,
    pub(super) pastas_nao_carregadas: bool,
    pub(super) documentos: Vec<DocumentoNaArvore>,
    pub(super) documentos_incluidos: bool,
    pub(super) aviso_leitura_parcial: Option<ErroArvore>,
}

pub(super) fn conferir(
    id: &str,
    nos: &[NoDePasta],
    protocolos: &[(usize, String)],
    documentos: Vec<DocumentoLido>,
    permitir_parcial: bool,
) -> Result<Conferencia, ErroArvore> {
    let (pastas, pastas_nao_carregadas, documentos_incluidos, aviso_leitura_parcial) =
        match conferir_pastas(id, nos, protocolos, &documentos) {
            Ok((pastas, pastas_nao_carregadas)) => (pastas, pastas_nao_carregadas, true, None),
            Err(erro) if permitir_parcial && erro.admite_leitura_parcial() => {
                (Vec::new(), true, false, Some(erro))
            }
            Err(erro) => return Err(erro),
        };
    Ok(Conferencia {
        pastas,
        pastas_nao_carregadas,
        documentos: if documentos_incluidos {
            documentos.into_iter().map(|d| d.documento).collect()
        } else {
            Vec::new()
        },
        documentos_incluidos,
        aviso_leitura_parcial,
    })
}
