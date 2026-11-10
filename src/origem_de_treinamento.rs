//! Onde cada `--confirmar` pode gravar. Decisão do proprietário
//! (10/11/2026): a escrita que passou em ensaio ao vivo em treinamento fica
//! aprovada para produção, para uso dos usuários; as demais gravam só nas
//! instalações públicas de treinamento. Conferido antes da primeira requisição.

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CausaDaOrigem {
    OrigemForaDoTreinamento,
}

const MENSAGEM: &str =
    "a origem da sessão não é instalação pública de treinamento; nenhuma requisição foi enviada";

/// Saída pública da recusa tipada, antes de qualquer requisição.
pub(crate) fn recusa(json_mode: bool, causa: &CausaDaOrigem) -> std::process::ExitCode {
    crate::saida_erro::erro_fonte_tipado(json_mode, causa, MENSAGEM)
}

/// As escritas da CLI, cada uma com o seu estado de aprovação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Escrita {
    DocumentoIncluirExterno,
    ProcessoPrazo,
    ProcessoRelacionar,
    ProcessoAndamento,
    ProcessoAtribuir,
    ProcessoEnviarRemovendoAnotacoes,
}

impl Escrita {
    /// Ensaio ao vivo aprovado em treinamento, com relatório:
    /// - incluir externo: `docs/validacao-live/2026-11-09-documento-incluir-externo-unb.md`;
    /// - prazo: `docs/validacao-live/2026-11-10-processo-prazo-unb.md`.
    fn aprovada_em_treinamento(self) -> bool {
        matches!(self, Self::DocumentoIncluirExterno | Self::ProcessoPrazo)
    }
}

/// Único ponto de construção de [`OrigemForaDoTreinamento`]. Escrita
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

    // Aceite técnico da #620: a causa publicada vem do tipo via Serde.
    // Este teste entra em RED antes da implementação.
    #[test]
    fn causa_de_origem_serde_direto_preserva_identidade_legacy() {
        let erro = exigir(
            Escrita::ProcessoAndamento,
            "https://sei.exemplo.gov.br/sei",
        )
        .unwrap_err();
        assert_eq!(
            serde_json::to_value(erro).unwrap(),
            serde_json::json!("origem_fora_do_treinamento")
        );
    }

    #[test]
    fn aceita_so_instalacoes_de_treinamento_e_a_bancada() {
        for origem in ORIGENS {
            assert!(
                exigir(Escrita::ProcessoAndamento, origem).is_ok(),
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
                    exigir(Escrita::ProcessoAndamento, producao_ou_parecida),
                    Err(CausaDaOrigem::OrigemForaDoTreinamento)
                ),
                "{producao_ou_parecida}"
            );
            assert!(exigir(Escrita::ProcessoPrazo, producao_ou_parecida).is_ok());
            assert!(exigir(Escrita::DocumentoIncluirExterno, producao_ou_parecida).is_ok());
        }
    }
}
