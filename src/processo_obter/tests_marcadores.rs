//! RED/GREEN dos marcadores depois da árvore: sequências exatas de
//! requisições (frio, quente, `--sem-marcadores` e as combinações com
//! `--sem-andamentos`/`--sem-propriedades`), índice e recusas. Nenhum POST:
//! a tela tem escrita (remover, adicionar) que nunca é tocada.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;
use crate::marcadores_processo::ErroMarcadores;
use crate::marcadores_processo::tests_apoio::{
    HASH_MARCADORES, link_dos_marcadores, tela_com, tela_dos_marcadores,
};
use crate::propriedades_processo::tests_apoio::{
    acao_da_barra, barra, com_barra, link_das_propriedades,
};

const TODOS: (bool, bool) = (true, true);
const SO_MARCADORES: (bool, bool) = (false, true);

/// Árvore com o histórico (forma GO) e, na barra do processo, os links das
/// propriedades e dos marcadores.
fn arvore_completa(id: &str) -> String {
    let acoes = [
        acao_da_barra(&link_das_propriedades(
            AcaoDoSei::ProcedimentoAlterar.como_na_url(),
            id,
        )),
        acao_da_barra(&link_dos_marcadores(id)),
    ];
    com_barra(&arvore_com_historico(id, NUP), 0, &barra(&acoes))
}

fn url_marcadores() -> String {
    absoluta(&link_dos_marcadores(ID))
}

fn url_guardada() -> String {
    url_marcadores().replace(HASH_MARCADORES, &hash('8'))
}

fn marcadores() -> String {
    tela_dos_marcadores(NUP, ID)
}

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

fn guardado(amb: &Ambiente) -> Option<String> {
    amb.indice
        .link(
            &instalacao(),
            chave(&geracao(), AcaoDoSei::AndamentoMarcadorGerenciar, ID),
        )
        .map(|u| u.url().to_owned())
}

fn nomes(r: &ProcessoAberto) -> Vec<&str> {
    match &r.marcadores {
        Marcadores::Lidos(lidos) => lidos.itens.iter().map(|m| m.nome.as_str()).collect(),
        outro @ Marcadores::Omitidos => panic!("marcadores não lidos: {outro:?}"),
    }
}

#[test]
fn frio_padrao_le_historico_propriedades_e_os_dois_marcadores_com_seis_get() {
    let amb = ambiente();
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::VisaoServida,
        TODOS,
        frio(
            arvore_completa(ID),
            vec![
                ok(historico('R', 1, 1)),
                ok(propriedades()),
                ok(marcadores()),
            ],
        ),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 6);
    assert_eq!(
        pedidos,
        pedidos_frio(&[
            url_historico(ID),
            url_propriedades(AcaoDoSei::ProcedimentoAlterar.como_na_url(), ID),
            url_marcadores()
        ])
    );
    assert_eq!(nomes(&r), ["Revis\u{e3}o", "Pendente"]);
    assert_eq!(r.marcadores.motivo(), None);
    assert_eq!(guardado(&amb), Some(url_marcadores()));
    let debug = format!("{r:?}");
    assert!(!debug.contains(HASH_MARCADORES), "{debug}");
}

#[test]
fn sem_marcadores_nao_pede_a_tela() {
    let amb = ambiente();
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::VisaoServida,
        (true, false),
        frio(
            arvore_completa(ID),
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
    assert_eq!(r.marcadores, Marcadores::Omitidos);
    assert_eq!(r.marcadores.motivo(), Some("omitidos_por_sem_marcadores"));
    assert_eq!(guardado(&amb), None);
}

/// Andamentos, (propriedades, marcadores), respostas depois da árvore e
/// URLs esperadas depois da árvore.
type Caso = (PedidoDeAndamentos, (bool, bool), Vec<Resposta>, Vec<String>);

#[test]
fn combinacoes_com_sem_andamentos_e_sem_propriedades() {
    let casos: [Caso; 4] = [
        (
            PedidoDeAndamentos::Omitidos,
            TODOS,
            vec![ok(propriedades()), ok(marcadores())],
            vec![
                url_propriedades(AcaoDoSei::ProcedimentoAlterar.como_na_url(), ID),
                url_marcadores(),
            ],
        ),
        (
            PedidoDeAndamentos::VisaoServida,
            SO_MARCADORES,
            vec![ok(historico('R', 1, 1)), ok(marcadores())],
            vec![url_historico(ID), url_marcadores()],
        ),
        (
            PedidoDeAndamentos::Omitidos,
            SO_MARCADORES,
            vec![ok(marcadores())],
            vec![url_marcadores()],
        ),
        (PedidoDeAndamentos::Omitidos, (false, false), vec![], vec![]),
    ];
    for (andamentos, leituras, depois, esperados) in casos {
        let amb = ambiente();
        let n = 3 + esperados.len();
        let (r, pedidos) = rodar_marc(
            &amb,
            andamentos,
            leituras,
            frio(arvore_completa(ID), depois),
        );
        assert_eq!(usize::from(r.unwrap().requisicoes), n);
        assert_eq!(pedidos, pedidos_frio(&esperados));
    }
}

#[test]
fn barra_sem_o_link_recusa_sem_requisicao() {
    let amb = ambiente();
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        frio(arvore_de(ID, NUP), vec![]),
    );
    assert_eq!(
        r.unwrap_err(),
        ErroProcessoObter::MarcadoresNaoPublicadosNaBarra
    );
    assert_eq!(pedidos, pedidos_frio(&[]));
}

#[test]
fn link_invalido_na_barra_so_pesa_quando_os_marcadores_sao_lidos() {
    let arvore = arvore_completa(ID).replace(
        &format!("acao={}&acao_origem=procedimento_visualizar&acao_retorno=arvore_visualizar&id_procedimento={ID}&", AcaoDoSei::AndamentoMarcadorGerenciar.como_na_url()),
        &format!("acao={}&acao_origem=procedimento_visualizar&acao_retorno=arvore_visualizar&id_procedimento=99&", AcaoDoSei::AndamentoMarcadorGerenciar.como_na_url()),
    );
    let amb = ambiente();
    let (r, _) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        (false, false),
        frio(arvore.clone(), vec![]),
    );
    assert!(r.is_ok());
    let amb = ambiente();
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        frio(arvore, vec![]),
    );
    let erro = r.unwrap_err();
    assert_eq!(
        erro,
        super::ErroProcessoObter::Marcadores(ErroMarcadores::EnderecoDosMarcadoresDeOutroProcesso)
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn tela_de_outro_processo_pelo_link_da_barra_e_recusa_e_nao_e_guardada() {
    let amb = ambiente();
    let outra = tela_dos_marcadores("00009.000009/2026-09", ID);
    let (_r, _) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        frio(arvore_completa(ID), vec![ok(outra)]),
    );
    assert_eq!(guardado(&amb), None);
}

#[test]
fn tela_sem_tabela_nao_vira_lista_vazia() {
    let amb = ambiente();
    let (r, _) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        frio(arvore_completa(ID), vec![ok(tela_com(NUP, ID, ""))]),
    );
    assert_eq!(
        r.unwrap_err(),
        ErroProcessoObter::TabelaDeMarcadoresNaoPublicadaNaTela
    );
    assert_eq!(guardado(&amb), Some(url_marcadores()));
}

#[test]
fn login_ou_503_na_tela_falham_sem_retry() {
    for (resposta, esperado) in [
        (
            r302(LOGIN),
            Some(super::ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            )),
        ),
        ((503, String::new(), None), None),
    ] {
        let amb = ambiente();
        let (r, pedidos) = rodar_marc(
            &amb,
            PedidoDeAndamentos::Omitidos,
            SO_MARCADORES,
            frio(arvore_completa(ID), vec![resposta]),
        );
        let erro = r.unwrap_err();
        match esperado {
            Some(e) => assert_eq!(erro, e),
            None => assert!(
                matches!(erro, super::ErroProcessoObter::Transporte(_)),
                "{erro:?}"
            ),
        }
        assert_eq!(pedidos.len(), 4);
    }
}

/// Índice quente: árvore, histórico, propriedades e (se dado) marcadores.
fn quente(amb: &Ambiente, marcadores_guardados: Option<&str>) {
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID);
    let mut links = vec![
        (AcaoDoSei::ProcedimentoVisualizar, url_arvore(ID)),
        (AcaoDoSei::ProcedimentoConsultarHistorico, url_historico(ID)),
        (
            AcaoDoSei::ProcedimentoAlterar,
            url_propriedades(AcaoDoSei::ProcedimentoAlterar.como_na_url(), ID),
        ),
    ];
    if let Some(url) = marcadores_guardados {
        links.push((AcaoDoSei::AndamentoMarcadorGerenciar, url.to_owned()));
    }
    for (acao, url) in links {
        amb.indice
            .registrar_link(&inst, chave(&g, acao, ID), &url)
            .unwrap();
    }
}

#[test]
fn repeticao_quente_custa_quatro_e_so_marcadores_custa_dois() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::VisaoServida,
        TODOS,
        vec![
            ok(arvore_completa(ID)),
            ok(historico('R', 1, 1)),
            ok(propriedades()),
            ok(marcadores()),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(r.requisicoes, 4);
    assert_eq!(
        pedidos,
        [
            get(&url_arvore(ID)),
            get(&url_historico(ID)),
            get(&url_propriedades(
                AcaoDoSei::ProcedimentoAlterar.como_na_url(),
                ID
            )),
            get(&url_guardada()),
        ]
    );
    assert_eq!(nomes(&r).len(), 2);
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        vec![ok(arvore_completa(ID)), ok(marcadores())],
    );
    assert_eq!(r.unwrap().requisicoes, 2);
    assert_eq!(pedidos, [get(&url_arvore(ID)), get(&url_guardada())]);
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        (false, false),
        vec![ok(arvore_completa(ID))],
    );
    assert_eq!(r.unwrap().requisicoes, 1);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn guardado_velho_e_esquecido_e_o_da_barra_segue() {
    let velhos = [
        r302(LOGIN),
        r302(&absoluta("controlador.php?acao=outra")),
        ok("<html><body>Link sem assinatura</body></html>".to_owned()),
        ok(tela_dos_marcadores("00009.000009/2026-09", ID)),
    ];
    for velho in velhos {
        let amb = ambiente();
        quente(&amb, Some(&url_guardada()));
        let (r, pedidos) = rodar_marc(
            &amb,
            PedidoDeAndamentos::Omitidos,
            SO_MARCADORES,
            vec![ok(arvore_completa(ID)), velho, ok(marcadores())],
        );
        assert_eq!(r.unwrap().requisicoes, 3);
        assert_eq!(
            pedidos,
            [
                get(&url_arvore(ID)),
                get(&url_guardada()),
                get(&url_marcadores()),
            ]
        );
        assert_eq!(guardado(&amb), Some(url_marcadores()));
    }
}

#[test]
fn guardado_nao_e_usado_quando_a_barra_nao_publica_a_tela() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        vec![ok(arvore_de(ID, NUP))],
    );
    assert_eq!(
        r.unwrap_err(),
        ErroProcessoObter::MarcadoresNaoPublicadosNaBarra
    );
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn guardado_que_responde_503_falha_sem_tentar_o_da_barra() {
    let amb = ambiente();
    quente(&amb, Some(&url_guardada()));
    let (r, pedidos) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        vec![ok(arvore_completa(ID)), (503, String::new(), None)],
    );
    assert!(matches!(
        r.unwrap_err(),
        super::ErroProcessoObter::Transporte(_)
    ));
    assert_eq!(pedidos.len(), 2);
}

/// Sem marcador, a tela de marcadores responde 302 para o formulário de
/// adicionar (`andamento_marcador_cadastrar`) do mesmo processo (`UnB`
/// 4.0.9, 04/11/2026). Vira lista vazia com total 0, sem seguir o 302.
fn cadastrar(id: &str) -> String {
    format!(
        "controlador.php?acao=andamento_marcador_cadastrar&acao_origem=andamento_marcador_gerenciar&acao_retorno=arvore_visualizar&id_procedimento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        hash('c')
    )
}

#[test]
fn redirecionamento_para_adicionar_marcador_e_lista_vazia_sem_seguir() {
    for local in [cadastrar(ID), absoluta(&cadastrar(ID))] {
        let amb = ambiente();
        let (r, pedidos) = rodar_marc(
            &amb,
            PedidoDeAndamentos::Omitidos,
            SO_MARCADORES,
            frio(arvore_completa(ID), vec![r302(&local)]),
        );
        let r = r.unwrap();
        assert_eq!(
            r.marcadores,
            Marcadores::Lidos(crate::marcadores_processo::MarcadoresLidos {
                total_declarado: 0,
                itens: Vec::new(),
            })
        );
        assert_eq!(pedidos.len(), 4, "o 302 não é seguido");
        assert_eq!(guardado(&amb), Some(url_marcadores()));
    }
}

#[test]
fn redirecionamento_para_adicionar_marcador_de_outro_processo_e_recusa() {
    let amb = ambiente();
    let (r, _) = rodar_marc(
        &amb,
        PedidoDeAndamentos::Omitidos,
        SO_MARCADORES,
        frio(arvore_completa(ID), vec![r302(&cadastrar("999"))]),
    );
    assert!(
        matches!(
            r.unwrap_err(),
            super::ErroProcessoObter::RedirecionamentoNaoReconhecido
        ),
        "302 de outro processo não prova zero marcadores"
    );
}
