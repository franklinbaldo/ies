//! Lote da leitura Web: omissão tipada sem esconder falhas
//! de identidade, sessão ou documento cancelado (Refs #60).
use super::FalhaDoDocumento;
use crate::documento::NumeroDocumento;
use crate::processo_obter::{DocumentoObtido, ErroDocumentoObter};

pub(super) struct LoteLido {
    pub(super) documentos: Vec<DocumentoObtido>,
    pub(super) omitidos: usize,
    pub(super) requisicoes_omitidas: u32,
}

/// Apenas a ausência observada do .src é omitível. A contagem vem
/// da rede, não de estimativa por caminho.
pub(super) fn ler_lote(
    numeros: &[NumeroDocumento],
    ocultar_inacessiveis: bool,
    mut ler: impl FnMut(&NumeroDocumento) -> Result<DocumentoObtido, ErroDocumentoObter>,
) -> Result<LoteLido, FalhaDoDocumento> {
    let mut documentos = Vec::new();
    let mut omitidos = 0usize;
    let mut requisicoes_omitidas = 0u32;
    for numero in numeros {
        match ler(numero) {
            Ok(documento) => documentos.push(documento),
            Err(ErroDocumentoObter::ConteudoNaoPublicadoNaArvore { requisicoes })
                if ocultar_inacessiveis =>
            {
                omitidos += 1;
                requisicoes_omitidas = requisicoes_omitidas.saturating_add(u32::from(requisicoes));
            }
            Err(erro) => {
                return Err(FalhaDoDocumento::Documento {
                    numero: numero.as_str().to_owned(),
                    lidos: documentos.len(),
                    erro,
                });
            }
        }
    }
    Ok(LoteLido {
        documentos,
        omitidos,
        requisicoes_omitidas,
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::documento_versao_cli::testes::com_foto;
    use crate::processo_obter::ErroProcessoObter;

    fn numeros() -> Vec<NumeroDocumento> {
        ["0000101", "0000102", "0000103"]
            .iter()
            .map(|n| NumeroDocumento::novo(n).unwrap())
            .collect()
    }

    fn documento(numero: &str) -> DocumentoObtido {
        let mut d = com_foto(
            numero,
            1,
            "<html><head><title>SEI/X - 0000101 - Despacho</title></head><body>x</body></html>",
        );
        d.requisicoes = 2;
        d
    }

    #[test]
    fn dado_src_vazio_quando_oculto_entao_conta_e_continua_sem_retry() {
        // Dado A legível, B com .src vazio na árvore (UnB) e C legível,
        // quando leio o lote com --ocultar, então só A/C são apresentados.
        let mut vistos = Vec::new();
        let resultado = ler_lote(&numeros(), true, |n| {
            vistos.push(n.as_str().to_owned());
            if n.as_str() == "0000102" {
                Err(ErroDocumentoObter::ConteudoNaoPublicadoNaArvore { requisicoes: 4 })
            } else {
                Ok(documento(n.as_str()))
            }
        });
        let Ok(lido) = resultado else {
            panic!("o lote deveria continuar");
        };
        assert_eq!(vistos, ["0000101", "0000102", "0000103"]);
        assert_eq!(
            lido.documentos
                .iter()
                .map(|d| d.numero.as_str())
                .collect::<Vec<_>>(),
            ["0000101", "0000103"]
        );
        assert_eq!((lido.omitidos, lido.requisicoes_omitidas), (1, 4));
        let requisicoes: u32 = lido
            .documentos
            .iter()
            .map(|d| u32::from(d.requisicoes))
            .sum();
        assert_eq!(requisicoes + lido.requisicoes_omitidas, 8);
    }

    #[test]
    fn dado_flag_ausente_quando_src_vazio_entao_falha_sem_proximo_get() {
        let mut chamadas = 0;
        let resultado = ler_lote(&numeros(), false, |_| {
            chamadas += 1;
            Err(ErroDocumentoObter::ConteudoNaoPublicadoNaArvore { requisicoes: 4 })
        });
        assert_eq!(chamadas, 1);
        assert!(matches!(
            resultado,
            Err(FalhaDoDocumento::Documento {
                erro: ErroDocumentoObter::ConteudoNaoPublicadoNaArvore { requisicoes: 4 },
                lidos: 0,
                ..
            })
        ));
    }

    #[test]
    fn dado_cancelado_ou_sessao_encerrada_quando_oculto_entao_nao_esconde_erro() {
        for erro in [
            ErroDocumentoObter::DocumentoCancelado,
            ErroDocumentoObter::Processo(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            )),
        ] {
            let resultado = ler_lote(&numeros(), true, |_| Err(erro.clone()));
            assert!(matches!(resultado, Err(FalhaDoDocumento::Documento { .. })));
        }
    }
}
