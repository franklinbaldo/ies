//! Estado SQLite descartável para o BDD de documento entre processos.
//! Hipóteses sintéticas não são observações das cassettes.
use super::*;

fn semear_documento_sem_src(banco: &Connection, origem: &Instalacao) {
    // Hipotese minima no SQLite: identidade publicada, mas sem link de conteudo.
    use iesei::documentos_simulados::{self as internos, Processo, Tipo};
    internos::registrar_processo(
        banco,
        origem,
        CENARIO,
        &Processo {
            id: "7".into(),
            nup: NUP_A.into(),
            unidade: "TESTE".into(),
        },
    )
    .unwrap();
    internos::registrar_tipo(
        banco,
        origem,
        CENARIO,
        "7",
        &Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        },
    )
    .unwrap();
    let sem_conteudo =
        internos::criar(banco, origem, CENARIO, "7", "291", "Metadados sem conteudo").unwrap();
    assert_eq!(sem_conteudo.numero_sei, "1000001");
}

#[derive(Clone, Copy)]
pub(super) enum EstadoExtra {
    Nenhum,
    SemSrc,
    Cancelado,
    NumeroRepetido,
    RevogadoAoVisualizar,
    TituloTrocadoAoVisualizar,
}

fn semear_numero_repetido(banco: &mut Connection, origem: &Instalacao) {
    // Hipótese mínima para BDD-60-05: dois nós com mesmo número SEI
    // e IDs distintos no mesmo processo, sem qualquer corpo substituto.
    documento_versoes::guardar(
        banco,
        CENARIO,
        &DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "44".into(),
            id_procedimento: "7".into(),
            numero_processo: NUP_A.into(),
            numero: DOC_A.into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "TESTE".into(),
                unidade: "TESTE".into(),
                modificacao: "01/11/2026".into(),
                html_corpo: "<p>Outro corpo que não pode substituir DOC_A</p>".into(),
            }],
        },
    )
    .unwrap();
}

pub(super) fn estado(com_segundo: bool, modo: EstadoExtra) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    // O NUP da recusa precisa ser navegável: sem processo cadastrado o
    // anti-IES devolve 503 na pesquisa e testa apenas transporte, não
    // o BDD-60-05. A árvore mantém somente DOC_A sob NUP_A.
    processos::guardar(
        &mut banco,
        &origem,
        CENARIO,
        &ProcessoSimulado {
            id: "7".into(),
            numero: NUP_A.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    for (id, processo, nup, numero, corpo) in [
        (
            "42",
            "7",
            NUP_A,
            DOC_A,
            "Conteudo exclusivo do primeiro processo",
        ),
        (
            "43",
            "8",
            NUP_B,
            DOC_B,
            "Conteudo exclusivo do segundo processo",
        ),
    ]
    .into_iter()
    .take(if com_segundo { 2 } else { 1 })
    {
        documento_versoes::guardar(
            &mut banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: id.into(),
                id_procedimento: processo.into(),
                numero_processo: nup.into(),
                numero: numero.into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: format!("<p>{corpo}</p>"),
                }],
            },
        )
        .unwrap();
    }

    match modo {
        EstadoExtra::Nenhum
        | EstadoExtra::RevogadoAoVisualizar
        | EstadoExtra::TituloTrocadoAoVisualizar => {}
        EstadoExtra::SemSrc => semear_documento_sem_src(&banco, &origem),
        EstadoExtra::Cancelado => {
            assert_eq!(
                cancelamento_documento::aplicar(
                    &banco,
                    &origem,
                    CENARIO,
                    "42",
                    "7",
                    "Motivo sintético"
                )
                .unwrap(),
                Aplicacao::Aplicada
            );
        }
        EstadoExtra::NumeroRepetido => semear_numero_repetido(&mut banco, &origem),
    }
    (banco, origem)
}
