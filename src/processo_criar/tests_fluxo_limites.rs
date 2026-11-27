//! Recusas da lista e limites da reconciliação na criação Web.
use super::*;
use crate::consulta_do_controlador::ErroParametroDaUrl;

#[test]
fn lista_40_com_identidade_duplicada_recusa_sem_abrir_cadastro() {
    let amb = ambiente();
    let html = lista(false).replace(
        "id_tipo_procedimento=12&amp;infra_hash",
        "id_tipo_procedimento=12&amp;id_tipo_procedimento=invalido&amp;infra_hash",
    );
    let respostas = vec![ok(tela_controle()), ok(html)];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(ErroCriar::Lista(ErroListaDeTipos::ChaveDoCadastroIlegivel(
            ErroParametroDaUrl::ParametroRepetido("id_tipo_procedimento")
        )))
    ));
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// A mesma ação esperada mais uma segunda ação não pode ser aceita em link
/// de escolha; isso é recusado antes de consultar uma URL ambígua.
#[test]
fn lista_40_com_duas_acoes_recusa_sem_abrir_cadastro() {
    let amb = ambiente();
    let html = lista(false).replace(
        "acao=procedimento_gerar&amp;id_tipo_procedimento",
        "acao=procedimento_gerar&amp;acao=procedimento_excluir&amp;id_tipo_procedimento",
    );
    let respostas = vec![ok(tela_controle()), ok(html)];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(ErroCriar::Lista(ErroListaDeTipos::AcaoDoCadastroIlegivel(
            crate::consulta_do_controlador::ErroAcaoDaUrl::AcaoRepetida
        )))
    ));
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// Gerados paginados (conta compartilhada do PEN1, 10/11/2026): sem
/// fotografia completa a escrita segue, e uma resposta incerta ao POST sai
/// com efeito incerto, sem reconciliar nem repetir.
#[test]
fn gerados_paginados_nao_impedem_o_post_mas_tornam_a_resposta_incerta() {
    let amb = ambiente();
    let parcial = tela_controle().replace(
        "<caption>0 registros:</caption></table>",
        "<caption>Processos (5 registros - 1 a 1):</caption>\
         <tr id='P17'><td><a class='processoVisualizado'>00000.000017/2026-17</a></td></tr></table>",
    );
    let respostas = vec![
        ok(parcial),
        ok(lista(false)),
        ok(cadastro()),
        ok("<html><body>resposta inesperada</body></html>".to_owned()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let erro = resultado.unwrap_err();
    let _diagnostico = erro.diagnostico_apos_post().expect("diagnóstico pós-POST");
    assert_eq!(requests.len(), 4);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

/// A restrição à escrita não transforma uma prévia legítima em erro.
/// A fotografia só é exigida quando há intenção de enviar o POST criador.
#[test]
fn previa_nao_exige_fotografia_completa_de_gerados() {
    let amb = ambiente();
    let parcial = tela_controle().replace(
        "<caption>0 registros:</caption></table>",
        "<caption>Processos (5 registros - 1 a 1):</caption>\
         <tr id='P17'><td><a class='processoVisualizado'>00000.000017/2026-17</a></td></tr></table>",
    );
    let respostas = vec![ok(parcial), ok(lista(false)), ok(cadastro())];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(false),
            rede,
        )
    });
    assert!(!resultado.unwrap().confirmado);
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// #245: resposta de POST sem prova exige leitura nova do Controle, não retry.
#[test]
fn falha_do_post_dispara_uma_unica_leitura_nova_dos_gerados() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(tela_controle()),
    ];
    let (_resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[4].metodo, "GET");
}

/// #245: falha da própria leitura nunca autoriza concluir que não criou.
#[test]
fn falha_do_post_e_da_releitura_permanece_incerta_sem_retry() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        Err(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado::sem_mensagem(503),
        )),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    let erro = resultado.unwrap_err();
    let diagnostico = erro.diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.candidatos_observados, None);
}
