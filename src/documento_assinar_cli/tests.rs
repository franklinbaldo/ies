//! Goldens históricos das falhas de assinatura, sem acesso ao SEI.
use super::*;
use crate::documento_assinar::VerificacaoDaAssinatura;
use crate::erro_html::ErroHtml;

#[test]
fn falhas_da_assinatura_serializam_direto_com_goldens_fixos() {
    let casos = [
        (
            ErroAssinar::AssinaturaNaoPublicadaNaBarra,
            r#"{"codigo":"fonte_invalida","erro":"a barra do documento não oferece assinar nesta sessão; nada foi assinado","retentavel":false}"#,
        ),
        (
            ErroAssinar::CargoNaoOferecidoPelaTela { cargos: vec![] },
            r#"{"cargos":[],"codigo":"fonte_invalida","erro":"o cargo pedido não está entre as opções da tela (compare com `cargos`); nada foi enviado","retentavel":false}"#,
        ),
        (
            ErroAssinar::DocumentoAssinadoPorOutroUsuario {
                antes: vec![Assinante {
                    nome: "Pessoa sintética".into(),
                    cargo: "Cargo sintético".into(),
                    unidade: "TESTE".into(),
                }],
            },
            r#"{"assinaturas":[{"cargo":"Cargo sintético","nome":"Pessoa sintética","unidade":"TESTE"}],"codigo":"fonte_invalida","erro":"o documento já tem assinatura de outro usuário, e a segunda não poderia ser conferida pela árvore; nada foi enviado","retentavel":false}"#,
        ),
        (
            ErroAssinar::AssinaturaNaoAplicada {
                antes: vec![],
                falha_do_envio: None,
            },
            r#"{"assinaturas":[],"codigo":"escrita_nao_aplicada","erro":"a árvore lida depois de assinar mostra as mesmas assinaturas de antes: o documento não foi assinado","retentavel":false}"#,
        ),
        (
            ErroAssinar::NaoConfirmadoNaAssinatura {
                antes: vec![],
                falha_do_envio: None,
                verificacao: Box::new(VerificacaoDaAssinatura::Lida(vec![])),
            },
            r#"{"assinaturas":[],"assinaturas_depois":[],"codigo":"nao_confirmado","erro":"não se sabe se o documento foi assinado; confira a árvore antes de repetir","retentavel":false}"#,
        ),
        (
            ErroAssinar::NaoConfirmadoNaAssinatura {
                antes: vec![],
                falha_do_envio: Some(FalhaDoEnvio::Redirecionamento),
                verificacao: Box::new(VerificacaoDaAssinatura::ArvoreNaoLida(ErroHtml::ausente(
                    "assinatura_da_arvore_ausente",
                ))),
            },
            r#"{"assinaturas":[],"codigo":"nao_confirmado","erro":"não se sabe se o documento foi assinado; confira a árvore antes de repetir","falha_do_envio":{"status_http":302},"retentavel":false,"status_http":302}"#,
        ),
    ];
    for (erro, esperado) in casos {
        let projecao = json_de_falha(&erro);
        assert_eq!(serde_json::to_string(&projecao).unwrap(), esperado);
        assert_ne!(
            std::any::type_name_of_val(&projecao),
            std::any::type_name::<serde_json::Value>()
        );
    }
}
#[test]
fn apresentacao_humana_parte_do_erro_tipado() {
    let cargos = ErroAssinar::CargoNaoOferecidoPelaTela {
        cargos: vec!["Cargo A".into(), "Cargo B".into()],
    };
    assert_eq!(
        linhas_da_falha(&cargos),
        vec![
            "IES: o cargo pedido não está entre as opções da tela (compare com `cargos`); nada foi enviado",
            "IES: cargos da tela: Cargo A; Cargo B",
        ]
    );
    let _incerto = ErroAssinar::NaoConfirmadoNaAssinatura {
        antes: vec![],
        falha_do_envio: Some(FalhaDoEnvio::Redirecionamento),
        verificacao: Box::new(VerificacaoDaAssinatura::ArvoreNaoLida(ErroHtml::ausente(
            "assinatura_da_arvore_ausente",
        ))),
    };
}

#[test]
fn falhas_http_da_assinatura_preservam_subcodigo_e_mensagem() {
    use crate::formulario_web::ErroSubmissaoFormulario;
    use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

    let casos = [
        (
            ErroAssinar::Formulario(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(
                StatusHttpRecusado {
                    status: 200,
                    mensagem_do_servidor: Some("Mensagem saneada.".into()),
                },
            )),
            r#"{"codigo":"fonte_invalida","erro":"o envio da assinatura não pôde ser montado; nada foi enviado","mensagem_do_servidor":"Mensagem saneada.","retentavel":false}"#,
        ),
        (
            ErroAssinar::TelaDaEscrita(FalhaTransporte::StatusHttpDeErroDoServidor(
                StatusHttpRecusado {
                    status: 500,
                    mensagem_do_servidor: Some("Mensagem saneada.".into()),
                },
            )),
            r#"{"codigo":"fonte_invalida","erro":"a tela da assinatura não foi servida; nada foi assinado","mensagem_do_servidor":"Mensagem saneada.","retentavel":false,"status_http":500}"#,
        ),
    ];
    for (erro, esperado) in casos {
        assert_eq!(
            serde_json::to_string(&json_de_falha(&erro)).unwrap(),
            esperado
        );
    }
}
