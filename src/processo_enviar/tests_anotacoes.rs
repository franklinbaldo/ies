//! BDD da prévia opt-in com leitura independente da anotação do processo.
//! URLs, ids e textos sintéticos; sem acesso ao SEI.
use super::tests_apoio::*;
use super::{PedidoDoEnvio, enviar_processo};
use crate::arvore_processo::tests_concluir::absoluta;
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_anotacao() -> String {
    format!(
        "controlador.php?acao=anotacao_registrar&acao_origem=arvore_visualizar&id_procedimento={ID}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_ENVIAR}"
    )
}

fn anotacao(texto: &str, id: &str) -> String {
    format!(
        r#"<html><form id="frmAnotacaoCadastro" method="post" action="controlador.php?acao=anotacao_registrar">
          <input name="hdnIdProtocolo" type="hidden" value="{id}">
          <textarea id="txaDescricao" name="txaDescricao">{texto}</textarea></form></html>"#
    )
}

fn previa_com_anotacao(
    texto: &str,
    id: &str,
) -> (
    Result<super::ProcessoEnviado, super::ErroEnviar>,
    Vec<Pedido>,
) {
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao(texto, id)),
            ok(um_destino()),
        ],
    );
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: true,
            confirmar: false,
        };
        enviar_processo(sessao, &pedido, indice, rede)
    })
}

#[test]
fn bdd_previa_remocao_observa_anotacao_sem_escrever() {
    // Dado link assinado de anotações na barra e texto servido no cadastro,
    // quando executo ies processo enviar --remover-anotacoes,
    // então a prévia registra a presença e só faz GET de leitura.
    let (resultado, pedidos) = previa_com_anotacao("Minha anotação", ID);
    let feito = resultado.expect("a anotação foi lida");
    assert_eq!(feito.anotacao_antes, Some(true));
    assert_eq!(feito.requisicoes, 6);
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    esperado.push(get(&absoluta(&link_anotacao())));
    esperado.push(post(
        &absoluta(&autocompletar()),
        &corpo_da_consulta(Versao::V40),
    ));
    assert_eq!(pedidos, esperado);
}

#[test]
fn bdd_anotacao_de_outro_processo_recusa_antes_de_consultar_destino() {
    // Dado formulario de anotações de outro processo,
    // quando solicito prévia com remoção, então nenhuma consulta à unidade
    // de destino nem escrita é executada.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao("Outra anotação", "999")),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: true,
            confirmar: false,
        };
        enviar_processo(sessao, &pedido, indice, rede)
    });
    assert!(resultado.is_err());
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    esperado.push(get(&absoluta(&link_anotacao())));
    assert_eq!(pedidos, esperado);
}

#[test]
fn bdd_sem_checkbox_mas_com_anotacao_planeja_edicao_na_previa() {
    // Dado a tela de envio sem o checkbox e o formulario da anotacao servido,
    // quando executo ies processo enviar --remover-anotacoes sem --confirmar,
    // entao o plano escolhe limpar a anotacao e nao faz escrita.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let tela =
        tela_de_envio(Versao::V41, ID).replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela),
            ok(anotacao("Anotação de teste", ID).replace(
                "</form>",
                r#"<button type="submit" name="sbmRegistrarAnotacao" value="Salvar">Salvar</button></form>"#,
            )),
            ok(um_destino()),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: true,
            confirmar: false,
        };
        enviar_processo(sessao, &pedido, indice, rede)
    });
    let feito = resultado.expect("a edicao da anotacao foi oferecida");
    assert_eq!(feito.anotacao_antes, Some(true));
    assert_eq!(
        feito.metodo_remocao,
        Some(super::MetodoRemocao::LimparAnotacao)
    );
    assert_eq!(feito.requisicoes, 6);
    let json = crate::processo_enviar_cli::json_de_sucesso(&feito);
    assert_eq!(json["metodo_remocao"], "limpar_anotacao");
    assert!(crate::processo_enviar_cli::humano(&feito).contains("limpar formulário"));
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    esperado.push(get(&absoluta(&link_anotacao())));
    esperado.push(post(
        &absoluta(&autocompletar()),
        &corpo_da_consulta(Versao::V41),
    ));
    assert_eq!(pedidos, esperado);
}

#[test]
fn bdd_confirmar_checkbox_verifica_anotacao_apos_tramitar() {
    // Dado checkbox oferecido e anotação presente na origem,
    // quando executo ies processo enviar --remover-anotacoes --confirmar,
    // então há um POST de envio e duas leituras independentes da anotação.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao("Texto anterior", ID)),
            ok(um_destino()),
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[DESTINO], &[])),
            ok(anotacao("", ID)),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    });
    let feito = resultado.expect("o envio e a remoção foram relidos");
    assert!(feito.alterado);
    assert_eq!(feito.anotacao_antes, Some(true));
    assert_eq!(feito.anotacao_depois, Some(false));
    assert_eq!(feito.requisicoes, 9);
    let mut esperados = pedidos_da_abertura();
    esperados.push(get(&url_da_tela()));
    esperados.push(get(&absoluta(&link_anotacao())));
    esperados.push(post(
        &absoluta(&autocompletar()),
        &corpo_da_consulta(Versao::V40),
    ));
    let corpo = corpo_do_envio(Versao::V40, false).replacen(
        "txtPrazo=",
        "chkSinRemoverAnotacoes=on&txtPrazo=",
        1,
    );
    esperados.push(post(&absoluta(&action_de_enviar()), &corpo));
    esperados.push(get(&url_arvore()));
    esperados.push(get(&absoluta(&link_anotacao())));
    assert_eq!(pedidos, esperados);
}

#[test]
fn bdd_anotacao_ainda_presente_apos_envio_e_resultado_parcial() {
    // Dado que o SEI tramitou, mas manteve a anotação,
    // quando confirmo a remoção pelo checkbox,
    // então a operação não declara sucesso nem repete o envio.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V41, ID)),
            ok(anotacao("Antes", ID)),
            ok(um_destino()),
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[DESTINO], &[])),
            ok(anotacao("Ainda presente", ID)),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    });
    let erro = resultado.unwrap_err();
    assert!(matches!(
        erro,
        super::ErroEnviar::AnotacoesNaoRemovidasAposEnvio { .. }
    ));
    assert_eq!(pedidos.len(), 9);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.metodo == "POST" && p.url == absoluta(&action_de_enviar()))
            .count(),
        1
    );
}

#[test]
fn bdd_releitura_da_anotacao_falha_apos_envio_sem_retry() {
    // Dado que a árvore comprova a tramitação, mas a anotação não pode ser relida,
    // quando confirmo, então devolvo efeito incerto (saída 3), sem novo POST.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao("Antes", ID)),
            ok(um_destino()),
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[DESTINO], &[])),
            Err(crate::transporte::FalhaTransporte::Rede),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    });
    let erro = resultado.unwrap_err();
    assert!(matches!(
        erro,
        super::ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes { .. }
    ));
    assert_eq!(pedidos.len(), 9);
    assert_eq!(pedidos.last(), Some(&get(&absoluta(&link_anotacao()))));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.metodo == "POST" && p.url == absoluta(&action_de_enviar()))
            .count(),
        1
    );
}

#[test]
fn bdd_333_04_remocao_incerta_preserva_destino_comprovado_na_saida_json() {
    // Dado envio comprovado por releitura, mas anotação indisponível depois,
    // quando executo ies processo enviar com --remover-anotacoes --confirmar,
    // então recebo saída 3 com o destino comprovado, sem segundo envio.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao("Antes", ID)),
            ok(um_destino()),
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[DESTINO], &[])),
            Err(crate::transporte::FalhaTransporte::Rede),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    });
    let erro = resultado.expect_err("a anotação não pôde ser relida");
    assert_eq!(
        erro,
        super::ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes {
            depois: Some(vec![DESTINO.into()])
        }
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.metodo == "POST" && p.url == absoluta(&action_de_enviar()))
            .count(),
        1,
        "a escrita não pode ser repetida"
    );
}

#[test]
fn bdd_fallback_ainda_nao_grava_antes_do_post_proprio() {
    // Dado formulário de anotação mas sem checkbox no envio,
    // quando confirmo, então não há POST até o fallback estar pronto.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let tela =
        tela_de_envio(Versao::V41, ID).replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![ok(tela), ok(anotacao("Antes", ID))],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(super::ErroEnviar::RemocaoDeAnotacoesSemVerificacao)
    ));
    assert_eq!(pedidos.len(), 5);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}

#[test]
fn bdd_333_04_arvore_incerta_mas_anotacao_removida_preserva_a_prova() {
    // Dado um único POST com checkbox e anotação removida pelo simulador,
    // quando a releitura da árvore falha e a da anotação tem sucesso,
    // então a saída 3 distingue a remoção observada da tramitação incerta.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao("Antes", ID)),
            ok(um_destino()),
            r302(&volta_a_arvore_do_envio()),
            Err(crate::transporte::FalhaTransporte::Rede),
            ok(anotacao("", ID)),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: true,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    });
    let erro = resultado.expect_err("árvore incerta não autoriza sucesso");
    assert!(matches!(
        erro,
        super::ErroEnviar::NaoConfirmadoNoEnvio {
            anotacao_depois: Some(false),
            ..
        }
    ));
    assert_eq!(pedidos.len(), 9);
    assert_eq!(pedidos.last(), Some(&get(&absoluta(&link_anotacao()))));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.metodo == "POST" && p.url == absoluta(&action_de_enviar()))
            .count(),
        1
    );
}
