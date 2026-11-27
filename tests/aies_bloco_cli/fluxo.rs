//! Execução CLI/TLS e anti-IES compartilhada pelos testes de bloco.
use super::{
    CENARIO, Command, Connection, ControleResumido, Instalacao, Linha, ORIGEM, Output, Pasta,
    TabelaObservada, TcpListener, UnidadeExibida, aceitar, argumentos, atender_http,
    banco_cassettes, certificado, criar, iniciar_tunel, registrar_blocos, thread,
};

#[derive(Clone, Copy)]
pub(super) enum Cenario {
    Lista,
    ListaCompleta,
    Interno,
    Detalhe,
    Ausente,
    Previa,
    Confirmar,
    RecusarParcial,
}

impl Cenario {
    fn numero(self) -> Option<&'static str> {
        match self {
            Self::Detalhe => Some("2195"),
            Self::Ausente => Some("9999"),
            _ => None,
        }
    }

    fn previa(self) -> bool {
        matches!(self, Self::Previa | Self::Confirmar | Self::RecusarParcial)
    }
}

pub(super) fn executar_fluxo(cenario: Cenario) -> (Output, Vec<String>) {
    let interno = matches!(cenario, Cenario::Interno);
    let detalhe = matches!(cenario, Cenario::Detalhe);
    let previa = cenario.previa();
    let completa = matches!(cenario, Cenario::ListaCompleta | Cenario::Confirmar);
    let confirmar = matches!(cenario, Cenario::Confirmar | Cenario::RecusarParcial);
    let gravar = matches!(cenario, Cenario::Confirmar);
    let esperados = if gravar {
        6
    } else if interno || detalhe || previa {
        3
    } else {
        2
    };
    let pasta = Pasta::nova();
    let (cert, identidade) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let nome_hash = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{nome_hash}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");

    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let simulador = thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
        banco_cassettes::guardar(
            &mut banco,
            CENARIO,
            &ControleResumido {
                instalacao: instalacao.clone(),
                unidade: UnidadeExibida::nova("TESTE").unwrap(),
                recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            },
        )
        .unwrap();
        registrar_blocos(&mut banco, &instalacao, interno, detalhe, completa);
        let antes = banco.total_changes();
        for _ in 0..esperados {
            let mut fluxo = aceitar(&servidor);
            atender_http(&mut fluxo, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        if gravar {
            criar::conferir_sqlite(&banco);
        } else if previa {
            assert_eq!(banco.total_changes(), antes);
        }
    });

    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let tunel = iniciar_tunel(proxy, identidade, endereco, esperados);

    let mut argumentos = argumentos(interno, cenario.numero(), previa);
    if confirmar {
        argumentos.push("--confirmar");
    }
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .expect("binário ies");
    if previa && (!confirmar || completa) {
        assert!(
            saida.status.success(),
            "{}",
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let pedidos = tunel.join().expect("GET HTTPS");
    simulador.join().expect("GET anti-IES");
    conferir_pedidos(&pedidos, cenario, esperados);
    (saida, pedidos)
}

fn conferir_pedidos(pedidos: &[String], cenario: Cenario, esperados: usize) {
    let interno = matches!(cenario, Cenario::Interno);
    let detalhe = matches!(cenario, Cenario::Detalhe);
    let previa = cenario.previa();
    let gravar = matches!(cenario, Cenario::Confirmar);
    assert_eq!(pedidos.len(), esperados);
    assert_eq!(
        pedidos.iter().filter(|p| p.starts_with("GET ")).count(),
        if gravar {
            5
        } else if detalhe || previa {
            3
        } else {
            2
        }
    );
    assert_eq!(
        pedidos.iter().filter(|p| p.starts_with("POST ")).count(),
        usize::from(interno || gravar)
    );
    assert!(pedidos[0].contains("acao=procedimento_controlar"));
    assert!(pedidos[1].contains(if interno {
        "acao=bloco_interno_listar"
    } else {
        "acao=bloco_assinatura_listar"
    }));
    if interno {
        assert!(pedidos[2].contains("acao=bloco_interno_listar"));
    }
    if previa {
        assert!(pedidos[2].contains("acao=bloco_assinatura_cadastrar"));
    }
    if detalhe {
        assert!(pedidos[2].contains("acao=rel_bloco_protocolo_listar"));
        assert!(pedidos[2].contains("id_bloco=2195"));
    }
}
