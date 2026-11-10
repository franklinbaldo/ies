//! Campos `relogin` e `requisicoes` da saída e a ligação só nas leituras.
use super::*;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

const RELOGOU: NotaDoRelogin = NotaDoRelogin {
    relogin: true,
    antes: 6,
    releitura: 3,
};
const OUTRO_RELOGOU: NotaDoRelogin = NotaDoRelogin {
    relogin: false,
    antes: 1,
    releitura: 3,
};

#[test]
fn sem_recomeco_a_saida_nao_muda() {
    let original = json!({"unidade": "U", "requisicoes": 3});
    let mut valor = original.clone();
    anotar(&mut valor, None);
    anotar_com_total(&mut valor, None);
    assert_eq!(valor, original);
    assert_eq!(somar(3, None), 3);
}

#[test]
fn relogin_publica_relogin_true_e_soma_as_requisicoes_do_login() {
    let mut valor = json!({"unidade": "U", "requisicoes": 3});
    anotar(&mut valor, Some(&RELOGOU));
    assert_eq!(valor["relogin"], true);
    assert_eq!(somar(3, Some(&RELOGOU)), 9);
    assert_eq!(somar(250, Some(&RELOGOU)), u8::MAX);
    let mut unidade = json!({"unidade_exibida": "U"});
    anotar_com_total(&mut unidade, Some(&RELOGOU));
    assert_eq!(
        unidade,
        json!({"unidade_exibida": "U", "relogin": true, "requisicoes": 9})
    );
}

#[test]
fn geracao_trocada_nao_publica_relogin_mas_conta_a_primeira_tentativa() {
    let mut valor = json!({"unidade": "U"});
    anotar_com_total(&mut valor, Some(&OUTRO_RELOGOU));
    assert_eq!(valor, json!({"unidade": "U", "requisicoes": 4}));
    assert_eq!(somar(3, Some(&OUTRO_RELOGOU)), 4);
}

#[test]
fn lista_detalhada_anota_cada_objeto_do_array() {
    let mut valor = json!([{"unidade": "U", "processos": []}]);
    anotar_com_total(&mut valor, Some(&RELOGOU));
    assert_eq!(valor[0]["relogin"], true);
    assert_eq!(valor[0]["requisicoes"], 9);
}

/// Leituras ligadas ao relogin; escritas e prévias, nunca.
#[test]
fn so_os_comandos_de_leitura_chamam_o_relogin() {
    let leituras = [
        ("processo NUP", include_str!("../processo_obter_cli.rs")),
        ("processo buscar", include_str!("../processo_buscar_cli.rs")),
        ("documento DOC", include_str!("../documento_obter_cli.rs")),
        (
            "documento buscar",
            include_str!("../documento_buscar_cli.rs"),
        ),
        ("unidade atual", include_str!("../unidade_atual_cli.rs")),
        ("unidade colunas", include_str!("../unidade_colunas_cli.rs")),
        ("listar-processos", include_str!("../unidade_lista_cli.rs")),
        ("recebidos/gerados", include_str!("../unidade_categoria.rs")),
    ];
    for (nome, fonte) in leituras {
        assert!(fonte.contains("relogin_cli::ler("), "{nome}");
    }
    let escritas = [
        ("concluir", include_str!("../processo_concluir_cli.rs")),
        ("reabrir", include_str!("../processo_reabrir_cli.rs")),
        ("enviar", include_str!("../processo_enviar_cli.rs")),
        ("assinar", include_str!("../documento_assinar_cli.rs")),
        (
            "colunas --definir",
            include_str!("../unidade_colunas_definir_cli.rs"),
        ),
    ];
    for (nome, fonte) in escritas {
        assert!(!fonte.contains("relogin"), "{nome}");
    }
    // `colunas` com `--definir` vai para a escrita, sem passar pela leitura.
    // O adaptador da CLI foi extraído de main.rs: conferir ambos os elos.
    let main = include_str!("../main.rs");
    assert!(main.contains("cli_unidade_aux::executar(cli, acao)"));
    let adaptador = include_str!("../cli_unidade_aux.rs");
    assert!(
        adaptador.contains("Some(definir) => crate::unidade_colunas_definir_cli::definir_colunas(")
    );
    assert!(adaptador.contains("None => crate::unidade_colunas_cli::mostrar_colunas("));
}

#[test]
fn subcodigos_publicos_de_sessao_separam_vencimento_de_recusas_sem_relogin() {
    assert!(FalhaTransporte::PaginaDeLoginRecebidaNoControle.sessao_encerrada());
    for falha in [
        FalhaTransporte::CookieInvalido,
        FalhaTransporte::OrigemNaoAutorizada,
        FalhaTransporte::Status(StatusHttpRecusado::sem_mensagem(429)),
        FalhaTransporte::Status(StatusHttpRecusado::sem_mensagem(301)),
        // #222: a sessão regravada duas vezes durante a leitura não é
        // sessão vencida; nenhum relogin implícito.
        FalhaTransporte::SessaoTrocadaDuranteALeitura,
    ] {
        assert!(!falha.sessao_encerrada(), "{falha:?}");
    }
}

#[test]
fn falha_que_sobe_preserva_o_erro_e_a_nota_do_relogin() {
    let direta = subir(FalhaDaLeitura::Leitura {
        erro: FalhaTransporte::Rede,
        nota: None,
    });
    assert_eq!(
        direta.downcast_ref::<FalhaTransporte>(),
        Some(&FalhaTransporte::Rede)
    );
    let refeita = subir(FalhaDaLeitura::Leitura {
        erro: FalhaTransporte::Rede,
        nota: Some(RELOGOU),
    });
    assert_eq!(
        refeita
            .downcast_ref::<LeituraRefeitaAposRelogin>()
            .map(|contexto| contexto.0),
        Some(RELOGOU)
    );
    assert_eq!(
        refeita.root_cause().downcast_ref::<FalhaTransporte>(),
        Some(&FalhaTransporte::Rede)
    );
    let login = subir::<FalhaTransporte>(FalhaDaLeitura::Relogin(
        crate::sessao_login::ErroLogin::CaptchaExigido,
    ));
    assert_eq!(
        login.downcast_ref::<crate::sessao_login::ErroLogin>(),
        Some(&crate::sessao_login::ErroLogin::CaptchaExigido)
    );
}
