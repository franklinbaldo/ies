//! BDD-381-01 (#381): prévia de inclusão de documento em bloco de assinatura.
//! CLI real → TLS local → anti-IES → SQLite. Hipótese sintética derivada do
//! card `bloco-incluir-anotar-retirar-unb` (barra do documento com
//! `bloco_escolher`; `frmBlocoEscolher` com `selBloco` e caixas por número
//! SEI); não valida equivalência ao SEI de treinamento.
use super::*;
use iesei::bloco_detalhe::{self, Detalhe, Item};
use iesei::blocos::{self, Linha as LinhaBloco, Pagina, Tipo};

const BLOCO: &str = "2195";

/// Documento 42 (DOC) disponível; bloco 2195 de assinatura sem DOC.
fn preparar(pasta: &Pasta) {
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let (mut banco, instalacao) = estado(&caminho, true, 0, false);
    blocos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Pagina {
            tipo: Tipo::Assinatura,
            total: 1,
            faixa: Some((1, 1)),
            linhas: vec![LinhaBloco {
                numero: BLOCO.into(),
                estado: "Gerado".into(),
                descricao: "Despachos de teste".into(),
            }],
        },
    )
    .unwrap();
    bloco_detalhe::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &Detalhe {
            numero: BLOCO.into(),
            tipo: Tipo::Assinatura,
            total: 1,
            faixa: Some((1, 1)),
            itens: vec![Item {
                processo: "00001.000002/2026-02".into(),
                documento: Some("0011600".into()),
                tipo: Some("Ofício".into()),
                anotacoes: String::new(),
            }],
        },
    )
    .unwrap();
}

#[test]
fn bdd_381_01_previa_documental_sem_incluir() {
    let pasta = Pasta::nova();
    preparar(&pasta);
    // Documento até a árvore, tela de escolha do bloco; Controle, lista e
    // detalhe do bloco. Nenhum POST de inclusão.
    let saida = executar(
        &pasta,
        &["--json", "bloco", "incluir", BLOCO, "--documento", DOC],
        REQUISICOES_DA_PREVIA,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let previa = json(&saida);
    assert_eq!(previa["bloco"], BLOCO);
    assert_eq!(previa["tipo_bloco"], "assinatura");
    assert_eq!(previa["alvo"]["documento"], DOC);
    assert_eq!(previa["presente_antes"], false);
    assert_eq!(previa["confirmado"], false);
    assert_eq!(previa["alterado"], false);
    assert!(previa["presente_depois"].is_null());

    // Nova execução no MESMO SQLite da prévia: o bloco continua sem DOC.
    let lido = executar(&pasta, &["--json", "bloco", BLOCO], 3);
    assert_eq!(lido.status.code(), Some(0));
    let itens = json(&lido)["bloco"]["itens"].clone();
    assert!(
        itens
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["documento"] != DOC),
        "{itens}"
    );

    // Consulta direta ao mesmo arquivo: o detalhe guardado é o semeado.
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let (banco, instalacao) = estado(&caminho, true, 0, false);
    let detalhe = bloco_detalhe::ler(&banco, &instalacao, CENARIO, Tipo::Assinatura, BLOCO)
        .unwrap()
        .expect("bloco semeado");
    assert_eq!(detalhe.total, 1);
    assert_eq!(detalhe.itens.len(), 1);
    assert_eq!(detalhe.itens[0].documento.as_deref(), Some("0011600"));
}

/// Pesquisa, Controle até a árvore do documento, tela de escolha; Controle,
/// lista e detalhe do bloco.
const REQUISICOES_DA_PREVIA: usize = 8;

/// BDD-381-02: única confirmação oferecida, aceita e comprovada no mesmo
/// SQLite; o segundo comando lê o estado produzido pelo primeiro.
#[test]
fn bdd_381_02_inclusao_documental_causal_e_persistente() {
    let pasta = Pasta::nova();
    preparar(&pasta);
    let saida = executar(
        &pasta,
        &[
            "--json",
            "bloco",
            "incluir",
            BLOCO,
            "--documento",
            DOC,
            "--confirmar",
        ],
        REQUISICOES_DA_PREVIA + 4, // um POST e três GETs da releitura independente
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let incluido = json(&saida);
    assert_eq!(incluido["confirmado"], true);
    assert_eq!(incluido["presente_antes"], false);
    assert_eq!(incluido["presente_depois"], true);
    assert_eq!(incluido["alterado"], true);

    let consulta = executar(&pasta, &["--json", "bloco", BLOCO], 3);
    assert_eq!(consulta.status.code(), Some(0));
    let itens = json(&consulta)["bloco"]["itens"].clone();
    assert_eq!(
        itens
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["documento"] == DOC)
            .count(),
        1,
        "documento não persistiu no mesmo SQLite: {itens}"
    );
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let (banco, instalacao) = estado(&caminho, true, 0, false);
    let detalhe = bloco_detalhe::ler(&banco, &instalacao, CENARIO, Tipo::Assinatura, BLOCO)
        .unwrap()
        .expect("bloco publicado");
    assert_eq!(detalhe.total, 2);
    assert_eq!(
        detalhe
            .itens
            .iter()
            .filter(|i| i.documento.as_deref() == Some(DOC))
            .count(),
        1
    );
}

/// O formulário oferece dois documentos: a primeira caixa vem marcada,
/// mas DOC é a segunda. Só DOC pode chegar ao bloco (mesmo SQLite).
#[test]
fn bdd_381_02_caixa_posterior_nao_inclui_documento_vizinho() {
    let pasta = Pasta::nova();
    preparar(&pasta);
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let (mut banco, instalacao) = estado(&caminho, true, 0, false);
    documento_versoes::guardar(
        &mut banco,
        CENARIO,
        &DocumentoSimulado {
            instalacao,
            id_documento: "41".into(),
            id_procedimento: "7".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0011518".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Ana".into(),
                unidade: "TESTE".into(),
                modificacao: "01/11/2026".into(),
                html_corpo: "<p>Documento anterior</p>".into(),
            }],
        },
    )
    .unwrap();
    drop(banco);
    let saida = executar(
        &pasta,
        &[
            "--json",
            "bloco",
            "incluir",
            BLOCO,
            "--documento",
            DOC,
            "--confirmar",
        ],
        REQUISICOES_DA_PREVIA + 4,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    assert_eq!(json(&saida)["alterado"], true);
    let leitura = executar(&pasta, &["--json", "bloco", BLOCO], 3);
    assert_eq!(leitura.status.code(), Some(0));
    let itens = json(&leitura)["bloco"]["itens"]
        .as_array()
        .unwrap()
        .to_vec();
    assert_eq!(itens.iter().filter(|i| i["documento"] == DOC).count(), 1);
    assert_eq!(
        itens.iter().filter(|i| i["documento"] == "0011518").count(),
        0
    );
    let (banco, instalacao) = estado(&caminho, true, 0, false);
    let detalhe = bloco_detalhe::ler(&banco, &instalacao, CENARIO, Tipo::Assinatura, BLOCO)
        .unwrap()
        .unwrap();
    assert_eq!(detalhe.total, 2);
}
