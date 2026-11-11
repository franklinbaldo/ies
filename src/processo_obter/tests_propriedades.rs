//! RED/GREEN das propriedades depois da árvore: sequências exatas de
//! requisições (frio, quente, `--sem-propriedades`, `--sem-andamentos`),
//! fallback `procedimento_consultar`, índice e recusas. Nenhum POST.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;
use crate::propriedades_processo::{ErroPropriedades, FonteDasPropriedades, NivelDeAcesso};

fn frio(arvore: String, depois: Vec<Resposta>) -> Vec<Resposta> {
    let mut respostas = vec![ok(controle(&[(ID, NUP)])), ok(trabalhar(ID)), ok(arvore)];
    respostas.extend(depois);
    respostas
}

fn pedidos_frio(depois: &[String]) -> Vec<Pedido> {
    let mut pedidos = vec![
        get(&url_controle()),
        get(&absoluta(&trabalhar_controle(ID))),
        get(&url_arvore(ID)),
    ];
    pedidos.extend(depois.iter().map(|u| get(u)));
    pedidos
}

fn guardado(amb: &Ambiente, acao: AcaoDoSei) -> Option<String> {
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .link(&inst, chave(&g, acao, ID))
        .map(|u| u.url().to_owned())
}

fn lidas(r: &ProcessoAberto) -> &crate::propriedades_processo::PropriedadesLidas {
    match &r.propriedades {
        Propriedades::Lidas(lidas) => lidas,
        outro => panic!("propriedades não lidas: {outro:?}"),
    }
}

#[test]
fn frio_padrao_le_historico_e_propriedades_com_cinco_get() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::VisaoServida,
        true,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![ok(historico('R', 1, 1)), ok(propriedades())],
        ),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 5);
    assert_eq!(
        pedidos,
        pedidos_frio(&[
            url_historico(ID),
            url_propriedades(AcaoDoSei::ProcedimentoAlterar.como_na_url(), ID)
        ])
    );
    let lidas = lidas(&r);
    assert_eq!(lidas.fonte, FonteDasPropriedades::Alterar);
    assert_eq!(lidas.nivel_de_acesso, NivelDeAcesso::Publico);
    assert_eq!(r.propriedades.motivo(), None);
    assert_eq!(
        guardado(&amb, AcaoDoSei::ProcedimentoAlterar),
        Some(url_propriedades(
            AcaoDoSei::ProcedimentoAlterar.como_na_url(),
            ID
        ))
    );
    let debug = format!("{r:?}");
    assert!(!debug.contains(&"3".repeat(64)), "{debug}");
}

#[test]
fn sem_propriedades_nao_pede_a_tela() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::VisaoServida,
        false,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![ok(historico('R', 1, 1))],
        ),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 4);
    assert_eq!(pedidos, pedidos_frio(&[url_historico(ID)]));
    assert_eq!(r.propriedades, Propriedades::Omitidas);
    assert_eq!(
        r.propriedades.motivo(),
        Some("omitidas_por_sem_propriedades")
    );
    assert_eq!(guardado(&amb, AcaoDoSei::ProcedimentoAlterar), None);
}

#[test]
fn sem_andamentos_le_so_as_propriedades() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![ok(propriedades())],
        ),
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(
        pedidos,
        pedidos_frio(&[url_propriedades(
            AcaoDoSei::ProcedimentoAlterar.como_na_url(),
            ID
        )])
    );
}

#[test]
fn sem_andamentos_e_sem_propriedades_custa_so_a_abertura() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        false,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![],
        ),
    );
    assert_eq!(r.unwrap().requisicoes, 3);
    assert_eq!(pedidos, pedidos_frio(&[]));
}

#[test]
fn sem_alterar_na_barra_le_pela_consulta() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoConsultar.como_na_url()),
            vec![ok(propriedades())],
        ),
    );
    let r = r.unwrap();
    assert_eq!(lidas(&r).fonte, FonteDasPropriedades::Consultar);
    assert_eq!(
        pedidos,
        pedidos_frio(&[url_propriedades(
            AcaoDoSei::ProcedimentoConsultar.como_na_url(),
            ID
        )])
    );
    assert_eq!(
        guardado(&amb, AcaoDoSei::ProcedimentoConsultar),
        Some(url_propriedades(
            AcaoDoSei::ProcedimentoConsultar.como_na_url(),
            ID
        ))
    );
}

#[test]
fn barra_sem_o_link_publica_o_motivo_sem_requisicao() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(arvore_de(ID, NUP), vec![]),
    );
    let r = r.unwrap();
    assert_eq!(r.propriedades, Propriedades::NaoPublicadasNaBarra);
    assert_eq!(
        r.propriedades.motivo(),
        Some("tela_de_propriedades_nao_publicada_na_barra")
    );
    assert_eq!(pedidos, pedidos_frio(&[]));
}

#[test]
fn link_invalido_na_barra_so_pesa_quando_as_propriedades_sao_lidas() {
    let arvore = arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()).replace(
        &format!("acao={}&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={ID}&", AcaoDoSei::ProcedimentoAlterar.como_na_url()),
        &format!("acao={}&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento=99&", AcaoDoSei::ProcedimentoAlterar.como_na_url()),
    );
    let amb = ambiente();
    let (r, _) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        false,
        frio(arvore.clone(), vec![]),
    );
    assert!(r.is_ok());
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(arvore, vec![]),
    );
    let erro = r.unwrap_err();
    assert_eq!(
        erro,
        super::ErroProcessoObter::Propriedades(
            ErroPropriedades::EnderecoDasPropriedadesDeOutroProcesso
        )
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn tela_de_outro_processo_pelo_link_da_arvore_e_recusa() {
    let amb = ambiente();
    let outra = crate::propriedades_processo::tests_apoio::tela_das_propriedades(
        "00009.000009/2026-09",
        ID,
    );
    let (_r, _) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![ok(outra)],
        ),
    );
    assert_eq!(guardado(&amb, AcaoDoSei::ProcedimentoAlterar), None);
}

#[test]
fn login_ou_503_na_tela_falham_sem_retry() {
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![r302(LOGIN)],
        ),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
    assert_eq!(pedidos.len(), 4);
    let amb = ambiente();
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        frio(
            arvore_com_propriedades(ID, NUP, AcaoDoSei::ProcedimentoAlterar.como_na_url()),
            vec![(503, String::new(), None)],
        ),
    );
    assert!(matches!(
        r.unwrap_err(),
        super::ErroProcessoObter::Transporte(_)
    ));
    assert_eq!(pedidos.len(), 4);
}

/// Índice quente: árvore, histórico e (se pedido) propriedades guardados.
fn quente(amb: &Ambiente, propriedades_guardadas: Option<&str>) {
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    let mut links = vec![
        (AcaoDoSei::ProcedimentoVisualizar, url_arvore(ID)),
        (AcaoDoSei::ProcedimentoConsultarHistorico, url_historico(ID)),
    ];
    if let Some(url) = propriedades_guardadas {
        links.push((AcaoDoSei::ProcedimentoAlterar, url.to_owned()));
    }
    for (acao, url) in links {
        amb.indice
            .registrar_link(&inst, chave(&g, acao, ID), &url)
            .unwrap();
    }
}

fn url_guardada() -> String {
    url_propriedades(AcaoDoSei::ProcedimentoAlterar.como_na_url(), ID)
        .replace(&"3".repeat(64), &hash('8'))
}

#[test]
fn repeticao_quente_custa_arvore_historico_e_propriedades_guardadas() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::VisaoServida,
        true,
        vec![
            ok(arvore_com_propriedades(
                ID,
                NUP,
                AcaoDoSei::ProcedimentoAlterar.como_na_url(),
            )),
            ok(historico('R', 1, 1)),
            ok(propriedades()),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(r.requisicoes, 3);
    assert_eq!(
        pedidos,
        [
            get(&url_arvore(ID)),
            get(&url_historico(ID)),
            get(&url_guardada()),
        ]
    );
}

#[test]
fn repeticao_quente_sem_andamentos_custa_dois_e_sem_os_dois_custa_um() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        vec![
            ok(arvore_com_propriedades(
                ID,
                NUP,
                AcaoDoSei::ProcedimentoAlterar.como_na_url(),
            )),
            ok(propriedades()),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 2);
    assert_eq!(pedidos, [get(&url_arvore(ID)), get(&url_guardada())]);
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        false,
        vec![ok(arvore_com_propriedades(
            ID,
            NUP,
            AcaoDoSei::ProcedimentoAlterar.como_na_url(),
        ))],
    );
    assert_eq!(r.unwrap().requisicoes, 1);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn guardado_velho_e_esquecido_e_o_da_arvore_segue() {
    let velhos = [
        r302(LOGIN),
        r302(&absoluta("controlador.php?acao=outra")),
        ok("<html><body>Link sem assinatura</body></html>".to_owned()),
        ok(
            crate::propriedades_processo::tests_apoio::tela_das_propriedades(
                "00009.000009/2026-09",
                ID,
            ),
        ),
    ];
    for velho in velhos {
        let amb = ambiente();
        quente(&amb, Some(&url_guardada()));
        let (r, pedidos) = rodar_props(
            &amb,
            PedidoDeAndamentos::Omitidos,
            true,
            vec![
                ok(arvore_com_propriedades(
                    ID,
                    NUP,
                    AcaoDoSei::ProcedimentoAlterar.como_na_url(),
                )),
                velho,
                ok(propriedades()),
            ],
        );
        assert_eq!(r.unwrap().requisicoes, 3);
        assert_eq!(
            pedidos,
            [
                get(&url_arvore(ID)),
                get(&url_guardada()),
                get(&url_propriedades(
                    AcaoDoSei::ProcedimentoAlterar.como_na_url(),
                    ID
                )),
            ]
        );
        assert_eq!(
            guardado(&amb, AcaoDoSei::ProcedimentoAlterar),
            Some(url_propriedades(
                AcaoDoSei::ProcedimentoAlterar.como_na_url(),
                ID
            ))
        );
    }
}

#[test]
fn guardado_nao_e_usado_quando_a_barra_nao_publica_a_tela() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        vec![ok(arvore_de(ID, NUP))],
    );
    assert_eq!(r.unwrap().propriedades, Propriedades::NaoPublicadasNaBarra);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn guardado_que_responde_503_falha_sem_tentar_o_da_arvore() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_props(
        &amb,
        PedidoDeAndamentos::Omitidos,
        true,
        vec![
            ok(arvore_com_propriedades(
                ID,
                NUP,
                AcaoDoSei::ProcedimentoAlterar.como_na_url(),
            )),
            (503, String::new(), None),
        ],
    );
    assert!(matches!(
        r.unwrap_err(),
        super::ErroProcessoObter::Transporte(_)
    ));
    assert_eq!(pedidos.len(), 2);
}
