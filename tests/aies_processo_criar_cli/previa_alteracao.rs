//! #877: prévia CLI real, anti-IES e SQLite; nenhuma transição externa.
use super::*;

const NUP: &str = "00000.000001/2026-01";
const ANTERIOR: &str = "Cadastro anterior";

fn banco_com_processo() -> (Connection, Instalacao) {
    let (banco, instalacao) = banco();
    let criado =
        criacao_processo::criar(&banco, &instalacao, CENARIO, "12", ANTERIOR, "TESTE").unwrap();
    assert_eq!(criado.nup, NUP);
    assert_eq!(criado.id_procedimento, "91");
    (banco, instalacao)
}

fn observar(banco: &Connection, instalacao: &Instalacao) -> criacao_processo::Criado {
    criacao_processo::ler_criado(banco, instalacao, CENARIO, "91")
        .unwrap()
        .unwrap()
}

#[test]
fn previa_e_noop_preservam_cadastro_e_identidade() {
    for pedida in ["Nova descrição", ANTERIOR] {
        let (saida, _, pedidos, antes, depois) = executar_em_bancada(
            &[
                "--json",
                "processo",
                "alterar",
                NUP,
                "--especificacao",
                pedida,
            ],
            false,
            false,
            banco_com_processo,
            observar,
        );
        assert_eq!(
            saida.status.code(),
            Some(0),
            "stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(valor["numero"], NUP);
        assert_eq!(valor["id_procedimento"], "91");
        assert_eq!(valor["especificacao_anterior"], ANTERIOR);
        assert_eq!(valor["especificacao_pedida"], pedida);
        assert_eq!(valor["alteracao_necessaria"], pedida != ANTERIOR);
        assert_eq!(valor["alterado"], false);
        assert_eq!(valor["confirmado"], false);
        assert!(pedidos.iter().any(|p| p == "GET procedimento_alterar 200"));
        assert!(pedidos.iter().all(|p| !p.starts_with("POST ")));
        assert_eq!(antes, depois);
    }
}

#[test]
fn confirmar_recusa_antes_de_abrir_sessao() {
    for pedida in ["Nova descrição", ANTERIOR] {
        let (saida, _, pedidos, antes, depois) = executar_em_bancada(
            &[
                "--json",
                "processo",
                "alterar",
                NUP,
                "--especificacao",
                pedida,
                "--confirmar",
            ],
            false,
            false,
            banco_com_processo,
            observar,
        );
        assert_eq!(saida.status.code(), Some(1));
        assert!(saida.stdout.is_empty());
        assert!(String::from_utf8_lossy(&saida.stderr).contains("EscritaAindaNaoValidada"));
        assert!(pedidos.is_empty());
        assert_eq!(antes, depois);
    }
}
