//! RED/GREEN do fluxo de `documento assinar`: sequências exatas de
//! requisições (orçamento do contrato: prévia 5, escrita 7 a frio por
//! `--processo`), decisão antes de escrever, um POST com a senha e a tabela
//! de reconciliação. Telas sintéticas.
use super::erro::{ErroAssinar as A, FalhaDoEnvio, VerificacaoDaAssinatura as V};
use super::tests_apoio::*;
use crate::arvore_processo::{Assinante, EdicaoNaBarra};
use crate::erro_html::ErroHtml;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::FalhaTransporte;

fn com(mut base: Vec<Resposta>, depois: Vec<Resposta>) -> Vec<Resposta> {
    base.extend(depois);
    base
}

fn pedidos(depois: Vec<Pedido>) -> Vec<Pedido> {
    let mut todos = pedidos_da_abertura();
    todos.extend(depois);
    todos
}

/// Abertura (3), tela do documento (5.0) e tela de assinatura.
fn ate_a_tela(arvore: String) -> Vec<Resposta> {
    com(
        abertura(arvore),
        vec![ok(tela_do_documento_50(ID, ID_DOC)), ok(tela_padrao())],
    )
}

fn pedidos_ate_a_tela() -> Vec<Pedido> {
    pedidos(vec![
        get(&url_da_tela_do_documento()),
        get(&url_de_assinar()),
    ])
}

fn pedidos_da_escrita() -> Vec<Pedido> {
    let mut todos = pedidos_ate_a_tela();
    todos.push(post(&absoluta(&action_de_assinar()), CORPO_DA_ASSINATURA));
    todos.push(get(&url_arvore()));
    todos
}

fn da_sessao(cargo: &str) -> Assinante {
    Assinante {
        nome: ASSINANTE.into(),
        cargo: cargo.into(),
        unidade: UNIDADE.into(),
    }
}

#[test]
fn previa_abre_as_telas_e_nao_envia() {
    let (r, feitos) = rodar_assinar(None, false, ate_a_tela(arvore_do_documento('N', &[])));
    let feito = r.unwrap();
    assert_eq!(feitos, pedidos_ate_a_tela());
    assert!(matches!(
        &feito.resultado,
        super::ResultadoDaAssinatura::Previa
    ));
    let saida = crate::documento_assinar_cli::json_de_sucesso(&feito);
    assert_eq!(saida["confirmado"], false);
    assert_eq!(saida["alterado"], false);
    assert_eq!(saida["assinaturas_depois"], serde_json::Value::Null);
    assert_eq!(
        (feito.numero.as_str(), feito.id_documento.as_str()),
        (DOC, ID_DOC)
    );
    assert_eq!(feito.assinante, ASSINANTE);
    assert_eq!(feito.cargo, None);
    assert_eq!(feito.cargo_da_tela.as_deref(), Some(CARGO));
    assert_eq!(
        feito.cargos,
        vec![CARGO.to_owned(), "Presidente".to_owned()]
    );
    assert_eq!(feito.assinaturas, Vec::<Assinante>::new());
    assert_eq!(feito.edicao, EdicaoNaBarra::NaoAssinado);
    assert_eq!(feito.requisicoes, 5);
}

#[test]
fn previa_pela_tela_do_documento_40() {
    let respostas = com(
        abertura(arvore_do_documento('N', &[])),
        vec![ok(tela_do_documento_40(ID, ID_DOC)), ok(tela_padrao())],
    );
    let (r, feitos) = rodar_assinar(Some("Presidente"), false, respostas);
    assert_eq!(r.unwrap().cargo.as_deref(), Some("Presidente"));
    assert_eq!(feitos, pedidos_ate_a_tela());
}

#[test]
fn cargo_fora_das_opcoes_e_recusa_sem_post() {
    for confirmar in [false, true] {
        let (r, feitos) = rodar_assinar(
            Some("Assessor"),
            confirmar,
            ate_a_tela(arvore_do_documento('N', &[])),
        );
        assert_eq!(
            r,
            Err(A::CargoNaoOferecidoPelaTela {
                cargos: vec![CARGO.into(), "Presidente".into()]
            })
        );
        assert_eq!(feitos, pedidos_ate_a_tela());
    }
}

#[test]
fn barra_sem_assinar_para_na_abertura() {
    let arvore = arvore_assinavel(ID, ID_DOC, DOC, "", &[]);
    let (r, feitos) = rodar_assinar(Some(CARGO), true, abertura(arvore));
    assert_eq!(r, Err(A::AssinaturaNaoPublicadaNaBarra));
    assert_eq!(feitos, pedidos_da_abertura());
}

#[test]
fn caneta_fora_da_forma_para_na_abertura() {
    let dupla = caneta_da_sessao(CARGO).replace("\\nTESTE\",", "\\nTESTE\\nOutro\\nX\\nY\",");
    let (r, feitos) = rodar_assinar(None, false, abertura(arvore_do_documento('S', &[dupla])));
    assert_eq!(
        r,
        Err(A::Arvore(ErroHtml::valor(
            "assinaturas_do_documento_fora_da_forma_observada"
        )))
    );
    assert_eq!(feitos, pedidos_da_abertura());
}

#[test]
fn ja_assinado_pela_sessao_nao_envia_nem_com_confirmar() {
    let arvore = arvore_do_documento('S', &[caneta_da_sessao("Presidente")]);
    let (r, feitos) = rodar_assinar(Some(CARGO), true, ate_a_tela(arvore));
    let feito = r.unwrap();
    assert_eq!(
        feito.resultado,
        super::ResultadoDaAssinatura::JaAssinadoPelaSessao {
            confirmacao_pedida: true
        }
    );
    let saida = crate::documento_assinar_cli::json_de_sucesso(&feito);
    assert_eq!(saida["confirmado"], true);
    assert_eq!(saida["alterado"], false);
    assert_eq!(saida["ja_assinado_pela_sessao"], true);
    assert_eq!(feito.assinaturas, vec![da_sessao("Presidente")]);
    assert_eq!(feito.edicao, EdicaoNaBarra::Assinado);
    assert_eq!(feitos, pedidos_ate_a_tela());
}

#[test]
fn assinado_por_outro_usuario_e_recusa() {
    let outra = caneta(0, ID_DOC, "Outra Pessoa", "Presidente", "TESTE", true);
    let (r, feitos) = rodar_assinar(
        Some(CARGO),
        true,
        ate_a_tela(arvore_do_documento('S', &[outra])),
    );
    assert!(matches!(r, Err(A::DocumentoAssinadoPorOutroUsuario { .. })));
    assert_eq!(feitos, pedidos_ate_a_tela());
}

#[test]
fn confirmar_envia_um_post_com_a_senha_e_confere_a_caneta() {
    let respostas = com(
        ate_a_tela(arvore_do_documento('N', &[])),
        vec![
            ok(resposta_aceita()),
            ok(arvore_do_documento('S', &[caneta_da_sessao(CARGO)])),
        ],
    );
    let (r, feitos) = rodar_assinar(Some(CARGO), true, respostas);
    let feito = r.unwrap();
    assert_eq!(feitos, pedidos_da_escrita());
    assert_eq!(
        feito.resultado,
        super::ResultadoDaAssinatura::Comprovada {
            assinatura_depois: da_sessao(CARGO),
            edicao_depois: EdicaoNaBarra::Assinado,
            reconciliado_por: None,
        }
    );
    assert_eq!(feito.cargo.as_deref(), Some(CARGO));
    let saida = crate::documento_assinar_cli::json_de_sucesso(&feito);
    assert_eq!(saida["confirmado"], true);
    assert_eq!(saida["alterado"], true);
    assert_eq!(saida["edicao_depois"], "assinado");
    assert_eq!(saida["reconciliado_por_leitura"], false);
    assert_eq!(feito.requisicoes, 7);
    assert!(!format!("{feito:?}").contains(SENHA));
}

#[test]
fn indice_quente_nao_confirma_assinatura_ausente_na_arvore_relida() {
    let amb = ambiente();
    let inst = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice.registrar_entrada_por_id(&inst, false);
    let credenciais = credenciais();
    let respostas = com(
        ate_a_tela(arvore_do_documento('N', &[])),
        vec![ok(resposta_aceita()), ok(arvore_do_documento('N', &[]))],
    );
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, envio| {
        let numero = crate::documento::NumeroDocumento::novo(DOC).unwrap();
        let confirmar = super::Confirmacao {
            cargo: CARGO,
            credenciais: &credenciais,
        };
        let pedido = super::PedidoDaAssinatura {
            numero: &numero,
            unidade: None,
            processo: Some(nup),
            cargo: Some(CARGO),
            confirmacao: Some(confirmar),
        };
        super::assinar_documento(sessao, &pedido, indice, envio)
    });
    assert!(
        resultado.is_err(),
        "sem caneta relida não há assinatura provada"
    );
    assert_eq!(pedidos, pedidos_da_escrita());
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn resposta_sem_finalizar_reconciliada_pela_leitura() {
    let respostas = com(
        ate_a_tela(arvore_do_documento('N', &[])),
        vec![
            ok(tela_padrao()),
            ok(arvore_do_documento('S', &[caneta_da_sessao(CARGO)])),
        ],
    );
    let (r, feitos) = rodar_assinar(Some(CARGO), true, respostas);
    let feito = r.unwrap();
    assert_eq!(
        feito.resultado,
        super::ResultadoDaAssinatura::Comprovada {
            assinatura_depois: da_sessao(CARGO),
            edicao_depois: EdicaoNaBarra::Assinado,
            reconciliado_por: Some(FalhaDoEnvio::RespostaSemFinalizacao),
        }
    );
    let saida = crate::documento_assinar_cli::json_de_sucesso(&feito);
    assert_eq!(saida["reconciliado_por_leitura"], true);
    assert!(saida["falha_do_envio"].is_object());
    assert_eq!(feitos, pedidos_da_escrita());
}

#[test]
fn leitura_sem_caneta_e_assinatura_nao_aplicada() {
    let respostas = com(
        ate_a_tela(arvore_do_documento('N', &[])),
        vec![ok(tela_padrao()), ok(arvore_do_documento('N', &[]))],
    );
    let (r, feitos) = rodar_assinar(Some(CARGO), true, respostas);
    let erro = r.unwrap_err();
    assert_eq!(
        erro,
        A::AssinaturaNaoAplicada {
            antes: vec![],
            falha_do_envio: Some(FalhaDoEnvio::RespostaSemFinalizacao),
        }
    );
    assert_eq!(feitos, pedidos_da_escrita());
}

#[test]
fn falha_depois_do_envio_e_leitura_falha_sao_nao_confirmado_sem_retry() {
    let respostas = com(
        ate_a_tela(arvore_do_documento('N', &[])),
        vec![Err(FalhaTransporte::Rede), Err(FalhaTransporte::Rede)],
    );
    let (r, feitos) = rodar_assinar(Some(CARGO), true, respostas);
    let erro = r.unwrap_err();
    let A::NaoConfirmadoNaAssinatura {
        falha_do_envio,
        verificacao,
        ..
    } = &erro
    else {
        panic!("esperado efeito incerto: {erro:?}");
    };
    assert_eq!(
        falha_do_envio,
        &Some(FalhaDoEnvio::Envio(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::Rede
        )))
    );
    assert!(matches!(verificacao.as_ref(), V::LeituraFalhou(_)));
    assert_eq!(feitos, pedidos_da_escrita());
    assert!(!format!("{erro:?}").contains(SENHA));
}

#[test]
fn caneta_com_outro_cargo_e_nao_confirmado() {
    let respostas = com(
        ate_a_tela(arvore_do_documento('N', &[])),
        vec![
            ok(resposta_aceita()),
            ok(arvore_do_documento('S', &[caneta_da_sessao("Presidente")])),
        ],
    );
    let (r, _) = rodar_assinar(Some(CARGO), true, respostas);
    let _erro = r.unwrap_err();
}
