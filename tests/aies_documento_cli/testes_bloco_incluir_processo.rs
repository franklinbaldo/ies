//! BDD-381-04: prévia processual e releitura independente pela CLI real.
//! Hipótese mínima sobre a cassette PEN1 (41 interações). O confronto
//! semântico original e o ensaio em treinamento permanecem NOT_RUN.
use super::testes_ciencia_processo::executar_ciencia_processo;
use super::*;
use iesei::bloco_detalhe::{self, Detalhe, Item};
use iesei::blocos::{self, Linha as LinhaBloco, Pagina, Tipo};
use iesei::processos::{self, ProcessoSimulado};

const BLOCO_INTERNO: &str = "3102";
const NUP_INTERNO: &str = "00001.000001/2026-01";

fn preparar(pasta: &Pasta) {
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let (mut banco, instalacao) = estado(&caminho, true, 0, false);
    processos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &ProcessoSimulado {
            id: "7".into(),
            numero: NUP_INTERNO.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    blocos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Pagina {
            tipo: Tipo::Interno,
            total: 1,
            faixa: Some((1, 1)),
            linhas: vec![LinhaBloco {
                numero: BLOCO_INTERNO.into(),
                estado: "Gerado".into(),
                descricao: "Processos para análise".into(),
            }],
        },
    )
    .unwrap();
    // Uma outra lista efetivamente observada e completa demonstra que
    // o número do bloco interno não colide com o de reunião.
    blocos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Pagina {
            tipo: Tipo::Reuniao,
            total: 0,
            faixa: None,
            linhas: vec![],
        },
    )
    .unwrap();
    bloco_detalhe::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Detalhe {
            numero: BLOCO_INTERNO.into(),
            tipo: Tipo::Interno,
            total: 1,
            faixa: Some((1, 1)),
            itens: vec![Item {
                processo: "00001.000002/2026-02".into(),
                documento: None,
                tipo: None,
                anotacoes: String::new(),
            }],
        },
    )
    .unwrap();
}

#[test]
fn bdd_381_04_previa_sem_inclusao_e_releitura_do_bloco_interno() {
    let pasta = Pasta::nova();
    preparar(&pasta);
    let (previa, pedidos) = executar_ciencia_processo(
        &pasta,
        &[
            "--json",
            "bloco",
            "incluir",
            BLOCO_INTERNO,
            "--processo",
            NUP_INTERNO,
        ],
    );
    assert_eq!(
        previa.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&previa.stdout),
        String::from_utf8_lossy(&previa.stderr),
    );
    let valor = json(&previa);
    assert_eq!(valor["bloco"], BLOCO_INTERNO);
    assert_eq!(valor["tipo_bloco"], "interno");
    assert_eq!(valor["alvo"]["processo"], NUP_INTERNO);
    assert_eq!(valor["presente_antes"], false);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert!(valor["presente_depois"].is_null());
    assert_eq!(valor["requisicoes"], pedidos.len());
    assert!(
        pedidos.iter().all(|p| p.starts_with("GET ")),
        "prévia deve ser sem POST: {pedidos:?}",
    );
    let (leitura, depois) = executar_ciencia_processo(&pasta, &["--json", "bloco", BLOCO_INTERNO]);
    assert_eq!(
        leitura.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&leitura.stderr),
    );
    let detalhe = json(&leitura);
    assert_eq!(detalhe["bloco"]["tipo"], "interno");
    assert!(
        detalhe["bloco"]["itens"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["processo"] != NUP_INTERNO && item["documento"].is_null())
    );
    assert!(depois.iter().all(|p| p.starts_with("GET ")));
    let (banco, instalacao) = estado(&pasta.0.join("anti-ies.sqlite3"), true, 0, false);
    let persistido = bloco_detalhe::ler(&banco, &instalacao, CENARIO, Tipo::Interno, BLOCO_INTERNO)
        .unwrap()
        .expect("detalhe observado no mesmo SQLite");
    assert_eq!(persistido.total, 1);
    assert_eq!(persistido.itens.len(), 1);
    assert_eq!(persistido.itens[0].documento, None);
    assert_ne!(persistido.itens[0].processo, NUP_INTERNO);
}
