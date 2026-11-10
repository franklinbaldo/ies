//! RED/GREEN dos palpites do índice que falham (esquecer e seguir), da
//! recusa da entrada por id e das falhas tipadas fora de palpite.
use super::ErroProcessoObter as E;
use super::tests_apoio::*;
use crate::arvore_processo::ErroArvore;
use crate::arvore_processo::tests::NUP;
use crate::erro_html::ErroHtml;
use crate::transporte::StatusHttpRecusado;

fn com_link_da_arvore(amb: &Ambiente) {
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, "procedimento_visualizar", ID),
            &url_arvore(ID),
        )
        .unwrap();
}

#[test]
fn link_da_arvore_de_outro_nup_e_esquecido_e_o_controle_resolve() {
    let amb = ambiente();
    com_link_da_arvore(&amb);
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(arvore_de(ID, "OUTRO-NUP")),
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, super::ResolvidoPor::Caixa);
    assert_eq!(r.requisicoes, 4);
    assert_eq!(
        pedidos,
        [
            get(&url_arvore(ID)),
            get(&url_controle()),
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
        ]
    );
    // Reaprendido pela leitura que o servidor confirmou.
    assert_eq!(
        amb.indice.id_por_nup(&instalacao(), NUP).as_deref(),
        Some(ID)
    );
}

#[test]
fn link_da_arvore_que_responde_login_e_esquecido_e_a_entrada_por_id_segue() {
    let amb = ambiente();
    com_link_da_arvore(&amb);
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            r302(LOGIN),
            r302(&trabalhar_entrada(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().resolvido_por, super::ResolvidoPor::IndiceEntrada);
    assert_eq!(pedidos[1], get(&url_entrada(ID)));
    let g = geracao();
    // O link novo da árvore substituiu o esquecido.
    assert!(
        amb.indice
            .link(&instalacao(), chave(&g, "procedimento_visualizar", ID))
            .is_some()
    );
}

#[test]
fn link_expirado_recusado_pelo_servidor_nao_volta_do_cache() {
    let amb = ambiente();
    com_link_da_arvore(&amb);
    // A validade temporal não é inferível do SQLite: o servidor recusa
    // o link uma vez, o palpite cai e a tentativa seguinte não o reutiliza.
    let (resultado, pedidos) = rodar(&amb, None, vec![r302(LOGIN), r302(LOGIN_RECUSA)]);
    assert!(resultado.is_err());
    assert_eq!(pedidos.len(), 2);
    assert_eq!(pedidos[0], get(&url_arvore(ID)));
    assert_eq!(pedidos[1], get(&url_entrada(ID)));
    let geracao_atual = geracao();
    assert_eq!(
        amb.indice.link(
            &instalacao(),
            chave(&geracao_atual, "procedimento_visualizar", ID)
        ),
        None
    );
}

#[test]
fn recusa_da_entrada_por_id_e_registrada_e_para_sem_outra_requisicao() {
    let amb = ambiente();
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    let (r, pedidos) = rodar(&amb, None, vec![r302(LOGIN_RECUSA)]);
    assert_eq!(r.unwrap_err(), E::EntradaPorIdRecusada);
    assert_eq!(pedidos, [get(&url_entrada(ID))]);
    assert_eq!(amb.indice.entrada_por_id(&inst), Some(false));
}

#[test]
fn entrada_recusada_antes_nao_e_tentada() {
    let amb = ambiente();
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice.registrar_entrada_por_id(&inst, false);
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert!(r.is_ok());
    assert_eq!(pedidos[0], get(&url_controle()));
}

#[test]
fn entrada_por_id_com_sessao_vencida_e_erro_sem_recusa_registrada() {
    let amb = ambiente();
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    let (r, pedidos) = rodar(&amb, None, vec![r302(LOGIN)]);
    assert_eq!(
        r.unwrap_err(),
        E::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
    assert_eq!(pedidos.len(), 1);
    assert_eq!(amb.indice.entrada_por_id(&inst), None);
}

#[test]
fn entrada_por_id_para_outro_id_esquece_o_processo_e_segue() {
    let amb = ambiente();
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            r302(&trabalhar_entrada("11")),
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(pedidos[1], get(&url_controle()));
}

#[test]
fn pesquisa_rapida_guardada_que_responde_login_e_esquecida() {
    let amb = ambiente();
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, "protocolo_pesquisa_rapida", ""),
            &absoluta(&action_rapida()),
        )
        .unwrap();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            r302(LOGIN),
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().resolvido_por, super::ResolvidoPor::Caixa);
    assert_eq!(pedidos[1], get(&url_controle()));
}

#[test]
fn pesquisa_rapida_sem_302_validacao_ou_status_sobem_tipados() {
    let amb = ambiente();
    let pagina =
        "<html><title>SEI - Pesquisa</title><form id=\"frmPesquisaProtocolo\"></form></html>";
    let (r, pedidos) = rodar(&amb, None, vec![ok(controle(&[])), ok(pagina.to_owned())]);
    assert_eq!(r.unwrap_err(), E::PesquisaRapidaSemUrlDeTrabalho);
    assert_eq!(pedidos.len(), 2);
    let validacao =
        "<html><textarea id=\"txaInfraValidacao\">Protocolo nao encontrado.</textarea></html>";
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![ok(controle(&[])), ok(validacao.to_owned())],
    );
    assert_eq!(
        r.unwrap_err(),
        E::Formulario(
            crate::formulario_web::ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(
                StatusHttpRecusado {
                    status: 200,
                    mensagem_do_servidor: Some("Protocolo nao encontrado.".into()),
                }
            )
        )
    );
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![ok(controle(&[])), (500, String::new(), None)],
    );
    assert!(matches!(
        r.unwrap_err(),
        E::Transporte(crate::transporte::FalhaTransporte::Status(
            StatusHttpRecusado { status: 500, .. }
        ))
    ));
}

#[test]
fn falhas_fora_de_palpite() {
    // Tela do processo responde login: sessão encerrada.
    let amb = ambiente();
    let (r, _) = rodar(&amb, None, vec![ok(controle(&[(ID, NUP)])), r302(LOGIN)]);
    assert_eq!(
        r.unwrap_err(),
        E::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
    // Controle com a tela de login.
    let login = "<form action='/sip/login.php'><input id='txtUsuario'></form>";
    let amb = ambiente();
    let (r, _) = rodar(&amb, None, vec![ok(login.to_owned())]);
    assert_eq!(
        r.unwrap_err(),
        E::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
    // Árvore de outro processo num caminho que não é palpite.
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, "OUTRO")),
        ],
    );
    assert_eq!(
        r.unwrap_err(),
        E::Arvore(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );
    assert_eq!(amb.indice.id_por_nup(&instalacao(), NUP), None);
    // NUP repetido no Controle.
    let amb = ambiente();
    let (r, pedidos) = rodar(&amb, None, vec![ok(controle(&[(ID, NUP), ("11", NUP)]))]);
    assert_eq!(
        r.unwrap_err(),
        E::Html(ErroHtml::repetido("processo_repetido_no_controle"))
    );
    assert_eq!(pedidos.len(), 1);
    // `--unidade` diferente: para no Controle.
    let amb = ambiente();
    let (_r, pedidos) = rodar(&amb, Some("OUTRA"), vec![ok(controle(&[(ID, NUP)]))]);
    assert_eq!(pedidos.len(), 1);
    // 302 da tela do processo para outro lugar.
    let amb = ambiente();
    let (_r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            r302("https://sei.exemplo.gov.br/sei/inicio.php"),
        ],
    );
}
