//! RED/GREEN da prévia e das recusas antes da escrita de `processo enviar`:
//! sequências exatas de requisições (orçamento do contrato).
use super::erro::{ErroEnviar as R, ErroTelaDeEnvio, FalhaDaConsulta};
use super::tests_apoio::*;
use crate::arvore_processo::{ErroEnviarNaBarra, MotivoAbertoEm};
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
    let casos: Vec<(Vec<&str>, Vec<String>, R)> = vec![
        (
            vec!["OUTRA"],
            vazio().to_vec(),
            R::ProcessoNaoAbertoNaUnidadeDaSessao {
                aberto_em: siglas(&["OUTRA"]),
            },
        ),
        (
            vec!["OUTRA"],
            enviar(),
            R::UnidadeDaSessaoAusenteDaSituacao {
                aberto_em: siglas(&["OUTRA"]),
            },
        ),
        (
            vec![UNIDADE],
            vazio().to_vec(),
            R::EnvioNaoPublicadoNaBarra {
                aberto_em: siglas(&[UNIDADE]),
            },
        ),
        (
            vec![UNIDADE],
            vec![pelo_envio(ID), pelo_envio(ID)],
            R::Barra(ErroEnviarNaBarra::Html(
                crate::erro_html::ErroHtml::repetido(
                    "links_de_enviar_repetidos_na_barra",
                    "o elemento aparece mais de uma vez",
                ),
            )),
        ),
        (
            vec![UNIDADE],
            vec![pelo_envio("11")],
            R::Barra(ErroEnviarNaBarra::EnderecoDoEnvioDeOutroProcesso),
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
        matches!(r, Err(R::SituacaoDoProcessoNaoLidaAntesDeEnviar { .. })),
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
            R::TelaDeEnvio(ErroTelaDeEnvio::Html(crate::erro_html::ErroHtml::ausente(
                "formulario_de_envio_ausente",
                "a tela não tem o elemento observado",
            ))),
        ),
        (
            r302("controlador.php?acao=principal"),
            R::TelaDaEscritaRedirecionada,
        ),
        (
            Err(FalhaTransporte::Rede),
            R::TelaDaEscrita(FalhaTransporte::Rede),
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
            R::DestinoNaoOferecidoPeloSei {
                oferecidos: vec!["OUTRA - Outra unidade".into()],
            },
        ),
        (
            dois,
            R::VariosDestinosComASigla {
                candidatos: vec!["TESTE_1_1 - Um".into(), "TESTE_1_1 - Dois".into()],
            },
        ),
        (
            "<html><body>Sessão</body></html>".to_owned(),
            R::Consulta(FalhaDaConsulta::RespostaDoAutocompletarNaoXml),
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
        Err(R::ProcessoJaAbertoNoDestino {
            aberto_em: siglas(&[UNIDADE, DESTINO])
        })
    );
    assert_eq!(feitos, pedidos_da_previa(Versao::V40));
}
