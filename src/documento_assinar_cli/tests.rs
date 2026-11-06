//! Goldens históricos das falhas de assinatura, sem acesso ao SEI.
use super::*;
use crate::documento_assinar::VerificacaoDaAssinatura;
use crate::erro_html::ErroHtml;

#[test]
fn falhas_da_assinatura_serializam_direto_com_goldens_fixos() {
    let casos = [
        (
            ErroAssinar::AssinaturaNaoPublicadaNaBarra,
            r#"{"codigo":"fonte_invalida","erro":"a barra do documento não oferece assinar nesta sessão; nada foi assinado","retentavel":false,"subcodigo":"assinatura_nao_publicada_na_barra"}"#,
        ),
        (
            ErroAssinar::CargoNaoOferecidoPelaTela { cargos: vec![] },
            r#"{"cargos":[],"codigo":"fonte_invalida","erro":"o cargo pedido não está entre as opções da tela (compare com `cargos`); nada foi enviado","retentavel":false,"subcodigo":"cargo_nao_oferecido_pela_tela"}"#,
        ),
        (
            ErroAssinar::DocumentoAssinadoPorOutroUsuario {
                antes: vec![Assinante {
                    nome: "Pessoa sintética".into(),
                    cargo: "Cargo sintético".into(),
                    unidade: "TESTE".into(),
                }],
            },
            r#"{"assinaturas":[{"cargo":"Cargo sintético","nome":"Pessoa sintética","unidade":"TESTE"}],"codigo":"fonte_invalida","erro":"o documento já tem assinatura de outro usuário, e a segunda não poderia ser conferida pela árvore; nada foi enviado","retentavel":false,"subcodigo":"documento_assinado_por_outro_usuario"}"#,
        ),
        (
            ErroAssinar::AssinaturaNaoAplicada {
                antes: vec![],
                falha_do_envio: None,
            },
            r#"{"assinaturas":[],"codigo":"escrita_nao_aplicada","erro":"a árvore lida depois de assinar mostra as mesmas assinaturas de antes: o documento não foi assinado","retentavel":false,"subcodigo":"assinatura_nao_aplicada"}"#,
        ),
        (
            ErroAssinar::EfeitoIncertoNaAssinatura {
                antes: vec![],
                falha_do_envio: None,
                verificacao: Box::new(VerificacaoDaAssinatura::Lida(vec![])),
            },
            r#"{"assinaturas":[],"assinaturas_depois":[],"codigo":"efeito_incerto","erro":"não se sabe se o documento foi assinado; confira a árvore antes de repetir","retentavel":false,"subcodigo":"efeito_incerto_na_assinatura","verificacao":null}"#,
        ),
        (
            ErroAssinar::EfeitoIncertoNaAssinatura {
                antes: vec![],
                falha_do_envio: Some(FalhaDoEnvio::Redirecionamento),
                verificacao: Box::new(VerificacaoDaAssinatura::ArvoreNaoLida(
                    ErroHtml::ausente("assinatura_da_arvore_ausente"),
                )),
            },
            r#"{"assinaturas":[],"codigo":"efeito_incerto","erro":"não se sabe se o documento foi assinado; confira a árvore antes de repetir","falha_do_envio":{"status_http":302,"subcodigo":"redirecionamento_apos_assinar"},"retentavel":false,"status_http":302,"subcodigo":"efeito_incerto_na_assinatura","verificacao":"assinatura_da_arvore_ausente"}"#,
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
