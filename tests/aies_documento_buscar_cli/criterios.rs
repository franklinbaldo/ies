use super::*;

// Estado sintético: nenhum resultado textual foi registrado. Os três tipos
// de observação legada (zero, redirecionamento e lista) não provam a consulta
// com q. O túnel exige três requisições reais, inclusive o POST.
#[test]
fn texto_livre_desconhecido_nao_herda_estado_do_protocolo() {
    for (termo, estado) in [
        (None, EstadoDeBusca::SemResultado),
        (Some(TERMO), EstadoDeBusca::SemResultado),
        (Some(TERMO_EXATO), EstadoDeBusca::DocumentoExato),
        (Some(TERMO_PAGINADO), EstadoDeBusca::Paginas),
    ] {
        let pasta = Pasta::nova();
        let mut args = vec!["--json", "documento", "buscar"];
        args.extend(termo);
        args.extend(["--texto", "licitação"]);
        let saida = executar_com_estado(&pasta, &args, estado);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "{args:?}: {}",
            String::from_utf8_lossy(&saida.stderr)
        );
        assert!(saida.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn coorte_contraditoria_recusa_cli_em_ambas_ordens() {
    for negativo_primeiro in [true, false] {
        for pagina in ["1", "2"] {
            let pasta = Pasta::nova();
            let saida = executar_com_estado(
                &pasta,
                &[
                    "--json",
                    "documento",
                    "buscar",
                    TERMO_PAGINADO,
                    "--pagina",
                    pagina,
                ],
                EstadoDeBusca::CoorteContraditoria { negativo_primeiro },
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

#[test]
fn processo_por_periodo_distingue_zero_observado_de_periodo_desconhecido() {
    for (ate, esperado) in [("31/01/2026", 0), ("30/01/2026", 1)] {
        let pasta = Pasta::nova();
        let saida = executar_com_estado(
            &pasta,
            &[
                "--json",
                "processo",
                "buscar",
                "--de",
                "01/01/2026",
                "--ate",
                ate,
            ],
            EstadoDeBusca::PeriodoProcessual,
        );
        assert_eq!(
            saida.status.code(),
            Some(esperado),
            "{}",
            String::from_utf8_lossy(&saida.stderr)
        );
        if esperado == 0 {
            let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
            assert_eq!(json["resultado"], "sem_resultado");
            assert_eq!(json["termo"], "");
            assert_eq!(json["total_declarado"], 0);
            assert_eq!(json["processos"].as_array().unwrap().len(), 0);
        } else {
            assert_eq!(saida.stdout.as_slice(), b"");
        }
    }
}

#[test]
fn processo_por_tipo_e_periodo_nao_reutiliza_resultado_de_outro_tipo() {
    for (tipo, datas, esperado) in [
        ("Administrativo", false, 0),
        ("Administrativo", true, 0),
        ("Judicial", false, 1),
        ("Judicial", true, 1),
    ] {
        let pasta = Pasta::nova();
        let mut args = vec!["--json", "processo", "buscar", "--tipo", tipo];
        if datas {
            args.extend(["--de", "01/01/2026", "--ate", "31/01/2026"]);
        }
        let saida = executar_com_estado(&pasta, &args, EstadoDeBusca::PeriodoProcessual);
        assert_eq!(
            saida.status.code(),
            Some(esperado),
            "{}",
            String::from_utf8_lossy(&saida.stderr)
        );
        if esperado == 0 {
            let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
            assert_eq!(json["resultado"], "sem_resultado");
            assert_eq!(json["termo"], "");
            assert_eq!(json["total_declarado"], 0);
        } else {
            assert_eq!(saida.stdout.as_slice(), b"");
        }
    }
}
