//! BDD-381-04: prévia processual e releitura independente pela CLI real.
//! Hipótese mínima sobre a cassette PEN1 (41 interações). O confronto
//! semântico original e o ensaio em treinamento permanecem `NOT_RUN`.
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
    assert_eq!(valor["presente_depois"], false);
    assert_eq!(valor["requisicoes"], pedidos.len());
    // A pesquisa rápida usa POST de leitura; a prévia proíbe POST mutador.
    assert!(
        pedidos
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "prévia não pode escrever no SEI: {pedidos:?}",
    );
    assert_eq!(
        pedidos.iter().filter(|p| p.starts_with("POST ")).count(),
        1,
        "apenas o POST da pesquisa de leitura é esperado",
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

#[test]
fn bdd_381_04_colisao_entre_tipos_nao_escolhe_o_primeiro() {
    let pasta = Pasta::nova();
    preparar(&pasta);
    let (mut banco, instalacao) = estado(&pasta.0.join("anti-ies.sqlite3"), true, 0, false);
    blocos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Pagina {
            tipo: Tipo::Reuniao,
            total: 1,
            faixa: Some((1, 1)),
            linhas: vec![LinhaBloco {
                numero: BLOCO_INTERNO.into(),
                estado: "Gerado".into(),
                descricao: "Outro tipo com mesmo número".into(),
            }],
        },
    )
    .unwrap();
    let (resultado, pedidos) =
        executar_ciencia_processo(&pasta, &["--json", "bloco", BLOCO_INTERNO]);
    assert_eq!(resultado.status.code(), Some(1), "colisão precisa recusar");
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")), "{pedidos:?}");
    let (db, origem) = estado(&pasta.0.join("anti-ies.sqlite3"), true, 0, false);
    let detalhe = bloco_detalhe::ler(&db, &origem, CENARIO, Tipo::Interno, BLOCO_INTERNO)
        .unwrap()
        .expect("estado original preservado");
    assert_eq!(detalhe.itens.len(), 1);
}

#[test]
fn bdd_381_04_lista_parcial_nao_prova_ausencia_de_colisao() {
    let pasta = Pasta::nova();
    preparar(&pasta);
    let (mut banco, instalacao) = estado(&pasta.0.join("anti-ies.sqlite3"), true, 0, false);
    blocos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Pagina {
            tipo: Tipo::Reuniao,
            total: 2,
            faixa: Some((1, 1)),
            linhas: vec![LinhaBloco {
                numero: "9999".into(),
                estado: "Gerado".into(),
                descricao: "Primeira página de reunião".into(),
            }],
        },
    )
    .unwrap();
    let (resultado, pedidos) = executar_ciencia_processo(
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
        resultado.status.code(),
        Some(1),
        "lista parcial não exclui colisão"
    );
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")), "{pedidos:?}");
}
