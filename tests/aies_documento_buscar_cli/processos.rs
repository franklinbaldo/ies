//! E2E sintético, sem alegação de paridade externa.
use super::*;
use iesei::documentos_simulados::{Processo, registrar_processo};
use iesei::pesquisa_processo::{
    Criterios, Pagina, registrar_pagina, registrar_sem_resultado, registrar_tipo,
};

pub(super) fn preparar_paginas(banco: &mut Connection, instalacao: &Instalacao) {
    let unidade = UnidadeExibida::nova("TESTE").unwrap();
    registrar_tipo(banco, instalacao, CENARIO, &unidade, "73", "Administrativo").unwrap();
    let criterios = Criterios {
        tipo_id: "73",
        de: "01/01/2026",
        ate: "31/01/2026",
    };
    for n in 1..=3 {
        registrar_processo(
            banco,
            instalacao,
            CENARIO,
            &Processo {
                id: n.to_string(),
                nup: format!("00002.{n:06}/2026-00"),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
    }
    for pagina in [
        Pagina {
            numero: 1,
            inicio: 1,
            fim: 2,
            total: 6,
            ids: vec!["1".into(), "2".into()],
        },
        Pagina {
            numero: 2,
            inicio: 3,
            fim: 4,
            total: 6,
            ids: vec!["3".into()],
        },
    ] {
        registrar_pagina(banco, instalacao, CENARIO, &unidade, criterios, &pagina).unwrap();
    }
}

#[test]
fn processo_por_tipo_e_periodo_lista_pagina_parcial() {
    for (pagina, codigo) in [("2", 0), ("3", 1)] {
        let pasta = Pasta::nova();
        let saida = executar_com_estado(
            &pasta,
            &[
                "--json",
                "processo",
                "buscar",
                "--tipo",
                "Administrativo",
                "--de",
                "01/01/2026",
                "--ate",
                "31/01/2026",
                "--pagina",
                pagina,
            ],
            EstadoDeBusca::PaginasProcessuais,
        );
        assert_eq!(
            saida.status.code(),
            Some(codigo),
            "{}",
            String::from_utf8_lossy(&saida.stderr)
        );
        if codigo == 0 {
            let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
            assert_eq!(json["resultado"], "lista");
            assert_eq!(json["termo"], "");
            assert_eq!(json["total_declarado"], 6);
            assert_eq!(json["faixa_observada"]["inicio"], 3);
            assert_eq!(json["faixa_observada"]["fim"], 4);
            assert_eq!(json["linhas_ausentes_da_faixa"], 1);
            assert_eq!(json["processos"].as_array().unwrap().len(), 1);
            assert_eq!(json["processos"][0]["numero"], "00002.000003/2026-00");
        } else {
            assert_eq!(saida.stdout.as_slice(), b"");
        }
    }
}

pub(super) fn preparar_coorte_contraditoria(
    banco: &mut Connection,
    instalacao: &Instalacao,
    zero_primeiro: bool,
) {
    let unidade = UnidadeExibida::nova("TESTE").unwrap();
    registrar_tipo(banco, instalacao, CENARIO, &unidade, "73", "Administrativo").unwrap();
    let criterios = Criterios {
        tipo_id: "73",
        de: "01/01/2026",
        ate: "31/01/2026",
    };
    if zero_primeiro {
        registrar_sem_resultado(banco, instalacao, CENARIO, &unidade, criterios).unwrap();
    }
    registrar_pagina(
        banco, instalacao, CENARIO, &unidade, criterios,
        &Pagina { numero: 2, inicio: 3, fim: 4, total: 4, ids: vec![] },
    )
    .unwrap();
    if !zero_primeiro {
        registrar_sem_resultado(banco, instalacao, CENARIO, &unidade, criterios).unwrap();
    }
}

#[test]
fn processo_recusa_coorte_contraditoria_antes_de_paginar() {
    for zero_primeiro in [true, false] {
        for pagina in ["1", "2"] {
            let pasta = Pasta::nova();
            let saida = executar_com_estado(
                &pasta,
                &[
                    "--json", "processo", "buscar", "--tipo", "Administrativo",
                    "--de", "01/01/2026", "--ate", "31/01/2026", "--pagina", pagina,
                ],
                EstadoDeBusca::CoorteProcessualContraditoria(zero_primeiro),
            );
            assert_eq!(
                saida.status.code(),
                Some(1),
                "{}",
                String::from_utf8_lossy(&saida.stderr)
            );
            assert_eq!(saida.stdout.as_slice(), b"");
        }
    }
}
