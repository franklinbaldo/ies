use super::nome_cenario_aceito;

    #[test]
    fn replay_pesquisa_aceita_filtro_nup_ja_previsto_para_replay() {
        use super::{Cli, Domain};
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "ies",
            "--json",
            "replay",
            "processo-busca-unb--67c8f2ed35b6",
            "--resultado-pesquisa",
            "--passo",
            "2",
            "--numero-processo",
            "00001.123456/2026-07",
        ])
        .unwrap();
        let Domain::Replay(replay) = cli.command else {
            panic!("comando diferente de replay");
        };
        assert!(replay.resultado_pesquisa);
        assert_eq!(replay.passo, vec![2]);
        assert_eq!(
            replay.numero_processo.as_deref(),
            Some("00001.123456/2026-07")
        );
    }

    #[test]
    fn aceita_apenas_nome_de_cenario_sem_caminho() {
        assert!(nome_cenario_aceito("processo-busca-unb--67c8f2ed35b6"));
        for nome in ["", "../processo", "a/b", "a\\b", "a..b", "cenário", "a b"] {
            assert!(!nome_cenario_aceito(nome), "não aceito: {nome}");
        }
    }
