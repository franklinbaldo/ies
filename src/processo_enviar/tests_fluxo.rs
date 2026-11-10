//! RED/GREEN da prévia e das recusas antes da escrita de `processo enviar`:
//! sequências exatas de requisições (orçamento do contrato).
use super::erro::{ErroEnviar, ErroTelaDeEnvio, FalhaDaConsulta};
use super::tests_apoio::*;
use crate::arvore_processo::{ErroEnviarNaBarra, MotivoAbertoEm};
use crate::erro_html::ErroHtml;
use crate::processo_concluir::tests_apoio::*;
use crate::transporte::FalhaTransporte;

fn previa_respostas(versao: Versao, aberto_em: &[&str], consulta: String) -> Vec<Resposta> {
    com(
        abertura(arvore(aberto_em, &enviar())),
        vec![ok(tela_de_envio(versao, ID)), ok(consulta)],
    )
}

#[test]
fn previa_pede_a_tela_e_a_consulta_e_nao_escreve() {
    for versao in [Versao::V40, Versao::V41] {
        for manter in [false, true] {
            let (r, feitos) = rodar_envio(
                DESTINO,
                manter,
                false,
                previa_respostas(versao, &[UNIDADE], um_destino()),
            );
            let feito = r.unwrap();
            assert_eq!(feito, previa(&[UNIDADE], manter));
            let esperado = if manter {
                siglas(&[UNIDADE, DESTINO])
            } else {
                siglas(&[DESTINO])
            };
            assert_eq!(feito.depois_esperado(), esperado);
            assert_eq!(feitos, pedidos_da_previa(versao));
        }
    }
}

#[test]
fn recusas_pela_situacao_e_pela_barra_so_com_a_abertura() {
    let casos: Vec<(Vec<&str>, Vec<String>, ErroEnviar)> = vec![
        (
            vec!["OUTRA"],
            vazio().to_vec(),
            ErroEnviar::ProcessoNaoAbertoNaUnidadeDaSessao {
                aberto_em: siglas(&["OUTRA"]),
            },
        ),
        (
            vec!["OUTRA"],
            enviar(),
            ErroEnviar::UnidadeDaSessaoAusenteDaSituacao {
                aberto_em: siglas(&["OUTRA"]),
            },
        ),
        (
            vec![UNIDADE],
            vazio().to_vec(),
            ErroEnviar::EnvioNaoPublicadoNaBarra {
                aberto_em: siglas(&[UNIDADE]),
            },
        ),
        (
            vec![UNIDADE],
            vec![pelo_envio(ID), pelo_envio(ID)],
            ErroEnviar::Barra(ErroEnviarNaBarra::LinksDeEnviarRepetidosNaBarra),
        ),
        (
            vec![UNIDADE],
            vec![pelo_envio("11")],
            ErroEnviar::Barra(ErroEnviarNaBarra::EnderecoDoEnvioDeOutroProcesso),
        ),
    ];
    for confirmar in [false, true] {
        for (aberto_em, acoes, erro) in &casos {
            let (r, feitos) = rodar_envio(
                DESTINO,
                false,
                confirmar,
                abertura(arvore(aberto_em, acoes)),
            );
            assert_eq!(r, Err(erro.clone()));
            assert_eq!(feitos, pedidos_da_abertura());
        }
    }
}

#[test]
fn situacao_nao_lida_recusa_antes_da_tela() {
    let sobrestado = arvore(&[UNIDADE], &enviar()).replace(
        "Processo aberto somente na unidade",
        "Processo sobrestado na unidade",
    );
    let (r, feitos) = rodar_envio(DESTINO, false, true, abertura(sobrestado));
    assert!(
        matches!(
            r,
            Err(ErroEnviar::SituacaoDoProcessoNaoLidaAntesDeEnviar { .. })
        ),
        "{r:?}"
    );
    assert_eq!(feitos, pedidos_da_abertura());
    let _ = MotivoAbertoEm::RaizPublicaOSobrestamento;
}

#[test]
fn tela_recusada_custa_um_get_sem_consulta() {
    let mut pedidos = pedidos_da_abertura();
    pedidos.push(get(&url_da_tela()));
    let fora = tela_de_envio(Versao::V41, ID).replace("frmAtividadeListar", "frmOutro");
    for (resposta, erro) in [
        (
            ok(fora),
            ErroEnviar::TelaDeEnvio(ErroTelaDeEnvio::Html(ErroHtml::ausente(
                "formulario_de_envio_ausente",
            ))),
        ),
        (
            r302("controlador.php?acao=principal"),
            ErroEnviar::TelaDaEscritaRedirecionada,
        ),
        (
            Err(FalhaTransporte::Rede),
            ErroEnviar::TelaDaEscrita(FalhaTransporte::Rede),
        ),
    ] {
        let respostas = com(abertura(arvore(&[UNIDADE], &enviar())), vec![resposta]);
        let (r, feitos) = rodar_envio(DESTINO, false, true, respostas);
        assert_eq!(r, Err(erro));
        assert_eq!(feitos, pedidos);
    }
}

#[test]
fn consulta_ou_destino_recusados_nao_escrevem() {
    let outro = resposta_xml(&[("9", "OUTRA - Outra unidade")]);
    let dois = resposta_xml(&[("1", "TESTE_1_1 - Um"), ("2", "TESTE_1_1 - Dois")]);
    let casos = [
        (
            outro,
            ErroEnviar::DestinoNaoOferecidoPeloSei {
                oferecidos: vec!["OUTRA - Outra unidade".into()],
            },
        ),
        (
            dois,
            ErroEnviar::VariosDestinosComASigla {
                candidatos: vec!["TESTE_1_1 - Um".into(), "TESTE_1_1 - Dois".into()],
            },
        ),
        (
            "<html><body>Sessão</body></html>".to_owned(),
            ErroEnviar::Consulta(FalhaDaConsulta::RespostaDoAutocompletarNaoXml),
        ),
    ];
    for confirmar in [false, true] {
        for (consulta, erro) in &casos {
            let (r, feitos) = rodar_envio(
                DESTINO,
                false,
                confirmar,
                previa_respostas(Versao::V41, &[UNIDADE], consulta.clone()),
            );
            assert_eq!(r, Err(erro.clone()));
            assert_eq!(feitos, pedidos_da_previa(Versao::V41));
        }
    }
}

#[test]
fn destino_ja_aberto_e_recusa_sem_escrita() {
    let (r, feitos) = rodar_envio(
        DESTINO,
        true,
        true,
        previa_respostas(Versao::V40, &[UNIDADE, DESTINO], um_destino()),
    );
    assert_eq!(
        r,
        Err(ErroEnviar::ProcessoJaAbertoNoDestino {
            aberto_em: siglas(&[UNIDADE, DESTINO])
        })
    );
    assert_eq!(feitos, pedidos_da_previa(Versao::V40));
}

#[test]
fn bdd_previa_remover_anotacoes_exige_checkbox_e_nao_escreve() {
    // Dado checkbox oferecido pela tela; quando peço a opção,
    // então a prévia apresenta a escolha com a mesma consulta de destino.
    for versao in [Versao::V40, Versao::V41] {
        let amb = ambiente();
        let (resultado, pedidos) = rodar_com(
            &amb,
            previa_respostas(versao, &[UNIDADE], um_destino()),
            |sessao, nup, indice, rede| {
                let pedido = super::PedidoDoEnvio {
                    nup,
                    unidade: None,
                    para: DESTINO,
                    manter_aberto: false,
                    remover_anotacoes: true,
                    confirmar: false,
                };
                super::enviar_processo(sessao, &pedido, indice, rede)
            },
        );
        assert!(resultado.unwrap().remover_anotacoes);
        assert_eq!(pedidos, pedidos_da_previa(versao));
    }
}

#[test]
fn bdd_sem_checkbox_recusa_antes_da_consulta_autocomplete() {
    // Dado uma tela sem a opção, quando solicito remoção,
    // então não há sequer o POST de consulta, muito menos escrita.
    let tela =
        tela_de_envio(Versao::V41, ID).replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
    let amb = ambiente();
    let (resultado, feitos) = rodar_com(
        &amb,
        com(abertura(arvore(&[UNIDADE], &enviar())), vec![ok(tela)]),
        |sessao, nup, indice, rede| {
            let pedido = super::PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: false,
            };
            super::enviar_processo(sessao, &pedido, indice, rede)
        },
    );
    assert_eq!(resultado, Err(ErroEnviar::RemocaoDeAnotacoesNaoOferecida));
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    assert_eq!(feitos, esperado);
}
