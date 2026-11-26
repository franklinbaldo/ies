//! Envio plural pela CLI real, usando a mesma bancada TLS e o SQLite do anti-IES.
//! Hipótese sintética do efeito plural; não é replay do POST original do PEN1.
use super::{CENARIO, ORIGEM, executar_em_bancada};
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::processos::{self, Destino, ProcessoSimulado};
use rusqlite::Connection;
use serde_json::Value;

const NUP: &str = "00001.000001/2026-01";

fn preparar() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::nova(
                Some(1),
                FaixaObservada::nova(1, 1),
                vec![Linha {
                    id: "10".into(),
                    numero: NUP.into(),
                    visualizado: true,
                    sigiloso: false,
                }],
            )
            .unwrap(),
            gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    processos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: NUP.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: Some("Nota de ensaio".into()),
            destinos: vec![
                Destino {
                    id: "2".into(),
                    sigla: "DESTINO".into(),
                    descricao: "DESTINO - Unidade de testes".into(),
                },
                Destino {
                    id: "3".into(),
                    sigla: "ARQUIVO".into(),
                    descricao: "ARQUIVO - Seção".into(),
                },
            ],
        },
    )
    .unwrap();
    (banco, instalacao)
}

fn observar(banco: &Connection, instalacao: &Instalacao) -> ProcessoSimulado {
    processos::ler(banco, instalacao, CENARIO, "10")
        .unwrap()
        .unwrap()
}

#[test]
fn envio_plural_confirma_uma_submissao_e_estado_sqlite() {
    for manter in [false, true] {
        let mut args = vec![
            "--json",
            "processo",
            "enviar",
            NUP,
            "--para",
            "DESTINO",
            "--para",
            "ARQUIVO",
            "--para",
            "DESTINO",
            "--confirmar",
        ];
        if manter {
            args.push("--manter-aberto");
        }
        let (saida, _, pedidos, antes, depois) =
            executar_em_bancada(&args, false, false, preparar, observar);
        assert_eq!(
            saida.status.code(),
            Some(0),
            "stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(json["confirmado"], true);
        assert!(json.get("destino").is_none());
        let destinos = json["destinos"].as_array().unwrap();
        assert_eq!(destinos.len(), 2);
        assert_eq!(destinos[0]["id_unidade"], "2");
        assert_eq!(destinos[1]["id_unidade"], "3");
        let esperado = if manter {
            vec!["TESTE", "DESTINO", "ARQUIVO"]
        } else {
            vec!["DESTINO", "ARQUIVO"]
        };
        assert_eq!(antes.aberto_em, ["TESTE"]);
        assert_eq!(depois.aberto_em, esperado);
        assert_eq!(json["depois"], serde_json::json!(esperado));
        assert_eq!(depois.anotacao_origem, antes.anotacao_origem);
        assert_eq!(
            pedidos
                .iter()
                .filter(|p| p.starts_with("POST procedimento_enviar "))
                .count(),
            1
        );
        let post = pedidos
            .iter()
            .position(|p| p == "POST procedimento_enviar 302")
            .unwrap();
        assert!(
            pedidos[post + 1..]
                .iter()
                .any(|p| p == "GET procedimento_visualizar 200")
        );
    }
}

#[test]
fn destino_inexistente_recusa_antes_de_qualquer_escrita() {
    let (saida, _, pedidos, antes, depois) = executar_em_bancada(
        &[
            "--json",
            "processo",
            "enviar",
            NUP,
            "--para",
            "DESTINO",
            "--para",
            "INEXISTENTE",
            "--remover-anotacoes",
            "--confirmar",
        ],
        false,
        false,
        preparar,
        observar,
    );
    assert_eq!(saida.status.code(), Some(1), "pedidos={pedidos:?}");
    assert_eq!(saida.stdout, [] as [u8; 0]);
    assert!(
        String::from_utf8_lossy(&saida.stderr).contains("DestinoNaoOferecidoPeloSei"),
        "stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    assert_eq!(antes, depois);
    assert!(
        pedidos
            .iter()
            .all(|p| !p.starts_with("POST procedimento_enviar ")
                && !p.starts_with("POST anotacao_registrar "))
    );
}

fn preparar_efeito_parcial() -> (Connection, Instalacao) {
    let (banco, instalacao) = preparar();
    // Contraprova sintética: o servidor aceita o POST, mas não persiste
    // o segundo destino. A releitura HTTP deve revelar o efeito parcial.
    banco
        .execute_batch(
            "CREATE TEMP TRIGGER omitir_segundo_destino
             BEFORE INSERT ON processo_abertura
             WHEN NEW.unidade = 'ARQUIVO'
             BEGIN SELECT RAISE(IGNORE); END;",
        )
        .unwrap();
    (banco, instalacao)
}

#[test]
fn aceite_http_com_efeito_parcial_nao_confirma_nem_repete_envio() {
    let (saida, _, pedidos, antes, depois) = executar_em_bancada(
        &[
            "--json",
            "processo",
            "enviar",
            NUP,
            "--para",
            "DESTINO",
            "--para",
            "ARQUIVO",
            "--confirmar",
        ],
        false,
        false,
        preparar_efeito_parcial,
        observar,
    );
    assert_eq!(saida.status.code(), Some(1), "pedidos={pedidos:?}");
    assert_eq!(saida.stdout, [] as [u8; 0]);
    assert!(
        String::from_utf8_lossy(&saida.stderr).contains("EnvioNaoAplicado"),
        "stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    assert_eq!(antes.aberto_em, ["TESTE"]);
    assert_eq!(depois.aberto_em, ["DESTINO"]);
    assert_eq!(depois.anotacao_origem, antes.anotacao_origem);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.starts_with("POST procedimento_enviar "))
            .count(),
        1
    );
    let post = pedidos
        .iter()
        .position(|p| p == "POST procedimento_enviar 302")
        .unwrap();
    assert!(
        pedidos[post + 1..]
            .iter()
            .any(|p| p == "GET procedimento_visualizar 200")
    );
}
