//! Limite inviolável do AGENTS.md para escrita: só instalações públicas de
//! treinamento ou homologação, nunca produção. Fonte única da lista usada
//! por todo `--confirmar`; conferida antes da primeira requisição.

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OrigemForaDoTreinamento;

impl OrigemForaDoTreinamento {
    pub(crate) const SUBCODIGO: &'static str = "origem_fora_do_treinamento";
    pub(crate) const MENSAGEM: &'static str = "a origem da sessão não é instalação pública de treinamento; nenhuma requisição foi enviada";
}

/// Único ponto de construção de [`OrigemForaDoTreinamento`].
pub(crate) fn exigir(origem: &str) -> Result<(), OrigemForaDoTreinamento> {
    let aceita = crate::instalacao::Instalacao::interpretar(origem)
        .is_some_and(|instalacao| ORIGENS.contains(&instalacao.url_base().as_str()));
    if aceita {
        Ok(())
    } else {
        Err(OrigemForaDoTreinamento)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_so_instalacoes_de_treinamento_e_a_bancada() {
        for origem in ORIGENS {
            assert!(exigir(origem).is_ok(), "{origem}");
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
                matches!(exigir(producao_ou_parecida), Err(OrigemForaDoTreinamento)),
                "{producao_ou_parecida}"
            );
        }
    }
}
