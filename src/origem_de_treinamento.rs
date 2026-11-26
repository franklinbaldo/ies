//! Onde cada `--confirmar` pode gravar. Decisões do proprietário
//! (10/11/2026): a escrita aprovada em ensaio ao vivo fica liberada para
//! produção, para uso dos usuários; as demais gravam só nas instalações
//! públicas de treinamento. Conferido antes da primeira requisição.
//!
//! **Aprovação exige ensaio em todas as instalações de treinamento**
//! ([`ORIGENS`]); uma instalação que bloqueie por motivo alheio ao IES
//! (WAF, Cloudflare, conta recusada) fica `NOT_RUN` no relatório e não
//! impede a aprovação.

/// Instalações de `docs/interfaces/credenciais-de-treinamento.md` e a
/// bancada reservada `sei.test` do anti-IES. Comparação exata da URL base:
/// hosts parecidos, outro caminho ou `http` não passam.
const ORIGENS: [&str; 8] = [
    "https://sei.treinamento.unb.br/sei",
    "https://sei.orgao1.tramita.processoeletronico.gov.br/sei",
    "https://sei.orgao2.tramita.processoeletronico.gov.br/sei",
    "https://sei.orgao3.tramita.processoeletronico.gov.br/sei",
    "https://sei.orgao4.tramita.processoeletronico.gov.br/sei",
    "https://teste.sei.tic.ufrj.br/sei",
    "https://seitreinamento.go.gov.br/sei",
    "https://sei.test/sei",
];

/// A origem da sessão não é instalação de treinamento: a escrita é
/// recusada antes de qualquer requisição.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum CausaDaOrigem {
    OrigemForaDoTreinamento,
}

/// As escritas da CLI, cada uma com o seu estado de aprovação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Escrita {
    BlocoIncluirDocumento,
    BlocoCriar,
    DocumentoIncluirExterno,
    DocumentoSalvar,
    ProcessoPrazo,
    ProcessoRelacionar,
    ProcessoAndamento,
    ProcessoAtribuir,
    ProcessoMarcar,
    ProcessoAnexar,
    ProcessoAnotar,
    ProcessoComentar,
    ProcessoAcompanharDeixar,
    ProcessoEnviarRemovendoAnotacoes,
    ProcessoEnviarMultiplos,
}

impl Escrita {
    /// Ensaio ao vivo aprovado nas sete instalações de treinamento:
    /// - prazo: `docs/validacao-live/2026-11-10-processo-prazo-7-instalacoes.md`;
    /// - incluir externo, relacionar, andamento e enviar removendo anotações:
    ///   `docs/validacao-live/2026-11-10-cobertura-7-instalacoes.md` (mais os
    ///   relatórios da `UnB` lá citados);
    /// - anotar, marcar, atribuir, anexar, documento salvar e bloco incluir
    ///   documento: `docs/validacao-live/2026-11-20-promocao-7-instalacoes.md`.
    ///
    /// Comentar e deixar de acompanhar falharam no ensaio e seguem restritas.
    fn aprovada_em_treinamento(self) -> bool {
        matches!(
            self,
            Self::DocumentoIncluirExterno
                | Self::ProcessoPrazo
                | Self::ProcessoRelacionar
                | Self::ProcessoAndamento
                | Self::ProcessoEnviarRemovendoAnotacoes
                | Self::ProcessoAnotar
                | Self::ProcessoMarcar
                | Self::ProcessoAtribuir
                | Self::ProcessoAnexar
                | Self::DocumentoSalvar
                | Self::BlocoIncluirDocumento
        )
    }
}

/// Único ponto de construção de [`CausaDaOrigem::OrigemForaDoTreinamento`]. Escrita
/// aprovada em treinamento grava em qualquer instalação; as demais, só nas
/// instalações públicas de treinamento.
pub(crate) fn exigir(escrita: Escrita, origem: &str) -> Result<(), CausaDaOrigem> {
    if escrita.aprovada_em_treinamento() {
        return Ok(());
    }
    let aceita = crate::instalacao::Instalacao::interpretar(origem)
        .is_some_and(|instalacao| ORIGENS.contains(&instalacao.url_base().as_str()));
    if aceita {
        Ok(())
    } else {
        Err(CausaDaOrigem::OrigemForaDoTreinamento)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escrita_restrita_recusa_producao_e_origens_imitadoras() {
        // Comentar não foi promovido (ensaio de 20/11/2026).
        assert!(!Escrita::ProcessoComentar.aprovada_em_treinamento());
        for origem in ORIGENS {
            assert!(
                exigir(Escrita::ProcessoComentar, origem).is_ok(),
                "{origem}"
            );
        }
        for origem in [
            "https://sei.unb.br/sei",
            "https://sei.treinamento.unb.br:443/sei",
            "https://sei.treinamento.unb.br.exemplo/sei",
            "https://sei.treinamento.unb.br/outro",
            "https://sei.treinamento.unb.br/sei/extra",
            "http://sei.treinamento.unb.br/sei",
            "https://sei.go.gov.br/sei",
        ] {
            assert_eq!(
                exigir(Escrita::ProcessoComentar, origem),
                Err(CausaDaOrigem::OrigemForaDoTreinamento),
                "{origem}"
            );
        }
    }

    #[test]
    fn criar_bloco_ainda_restrito_ao_treinamento() {
        assert!(exigir(Escrita::BlocoCriar, "https://sei.test/sei").is_ok());
        assert_eq!(
            exigir(Escrita::BlocoCriar, "https://sei.exemplo.gov.br/sei"),
            Err(CausaDaOrigem::OrigemForaDoTreinamento)
        );
    }

    #[test]
    fn retirada_de_acompanhamento_somente_no_treinamento() {
        assert!(exigir(Escrita::ProcessoAcompanharDeixar, "https://sei.test/sei").is_ok());
        assert!(matches!(
            exigir(
                Escrita::ProcessoAcompanharDeixar,
                "https://sei.exemplo.gov.br/sei"
            ),
            Err(CausaDaOrigem::OrigemForaDoTreinamento)
        ));
    }

    #[test]
    fn envio_multiplo_ainda_restrito_ao_treinamento() {
        assert!(exigir(Escrita::ProcessoEnviarMultiplos, "https://sei.test/sei").is_ok());
        assert_eq!(
            exigir(Escrita::ProcessoEnviarMultiplos, "https://sei.unb.br/sei"),
            Err(CausaDaOrigem::OrigemForaDoTreinamento)
        );
    }

    #[test]
    fn comentarios_sao_restritos_a_treinamento() {
        assert!(exigir(Escrita::ProcessoComentar, "https://sei.test/sei").is_ok());
        assert!(matches!(
            exigir(Escrita::ProcessoComentar, "https://sei.exemplo.gov.br/sei"),
            Err(CausaDaOrigem::OrigemForaDoTreinamento)
        ));
    }

    #[test]
    fn aceita_so_instalacoes_de_treinamento_e_a_bancada() {
        for origem in ORIGENS {
            assert!(
                exigir(Escrita::ProcessoAcompanharDeixar, origem).is_ok(),
                "{origem}"
            );
        }
        for producao_ou_parecida in [
            "https://sei.unb.br/sei",
            "https://sei.treinamento.unb.br.exemplo/sei",
            "https://sei.treinamento.unb.br/outro",
            "http://sei.treinamento.unb.br/sei",
            "https://sei.go.gov.br/sei",
            "https://sei.exemplo.gov.br/sei",
        ] {
            assert!(
                matches!(
                    exigir(Escrita::ProcessoAcompanharDeixar, producao_ou_parecida),
                    Err(CausaDaOrigem::OrigemForaDoTreinamento)
                ),
                "{producao_ou_parecida}"
            );
            assert!(exigir(Escrita::ProcessoPrazo, producao_ou_parecida).is_ok());
            assert!(exigir(Escrita::ProcessoRelacionar, producao_ou_parecida).is_ok());
            assert!(exigir(Escrita::ProcessoAndamento, producao_ou_parecida).is_ok());
            assert!(
                exigir(
                    Escrita::ProcessoEnviarRemovendoAnotacoes,
                    producao_ou_parecida
                )
                .is_ok()
            );
            assert!(exigir(Escrita::DocumentoIncluirExterno, producao_ou_parecida).is_ok());
            for promovida in [
                Escrita::ProcessoAnotar,
                Escrita::ProcessoMarcar,
                Escrita::ProcessoAtribuir,
                Escrita::ProcessoAnexar,
                Escrita::DocumentoSalvar,
                Escrita::BlocoIncluirDocumento,
            ] {
                assert!(exigir(promovida, producao_ou_parecida).is_ok());
            }
            assert!(exigir(Escrita::ProcessoComentar, producao_ou_parecida).is_err());
        }
    }
}
